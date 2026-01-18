mod ws_popup;

use aviutl2::{AnyResult, generic::GenericPlugin, ldbg};
use std::sync::Mutex;
use tap::Pipe;
use ws_popup::WsPopup;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AliasEntry {
    name: String,
    alias: String,
}

#[aviutl2::plugin(GenericPlugin)]
pub struct LocalAliasPlugin {
    webview: wry::WebView,
    window: WsPopup,

    aliases: Vec<AliasEntry>,
}
unsafe impl Send for LocalAliasPlugin {}
unsafe impl Sync for LocalAliasPlugin {}

static WEB_CONTENT: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/page/dist");

pub static CURRENT_ALIAS: Mutex<Option<AliasEntry>> = Mutex::new(None);

impl aviutl2::generic::GenericPlugin for LocalAliasPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Local Alias Plugin...");
        let window = WsPopup::new("Rusty Local Alias Plugin", (800, 600))?;
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("rusty-local-alias-plugin");
        let mut web_context = wry::WebContext::new(Some(cache_dir));
        let webview = wry::WebViewBuilder::new_with_web_context(&mut web_context)
            // JS -> Rust 受信
            .with_ipc_handler(|payload| {
                let message_str = payload.into_body();
                LocalAliasPlugin::ipc_handler(message_str);
            })
            .pipe(|builder| {
                if cfg!(debug_assertions) {
                    log::info!("Running in development mode, loading from localhost:5173");
                    builder.with_url("http://localhost:5173")
                } else {
                    log::info!("Running in production mode, loading from embedded assets");
                    builder
                        .with_custom_protocol("app".to_string(), move |_id, request| {
                            let path = request.uri().path().trim_start_matches('/');
                            ldbg!(path);
                            if let Some(file) = WEB_CONTENT.get_file(path) {
                                let mime = mime_guess::from_path(path).first_or_octet_stream();
                                wry::http::Response::builder()
                                    .header("Content-Type", mime.as_ref())
                                    .body(file.contents().to_vec().into())
                                    .unwrap()
                            } else {
                                wry::http::Response::builder()
                                    .status(404)
                                    .body(Vec::new().into())
                                    .unwrap()
                            }
                        })
                        .with_url("app://./index.html")
                }
            })
            .build(&window)?;

        Ok(LocalAliasPlugin {
            webview,
            window,
            aliases: Vec::new(),
        })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.set_plugin_information(&format!(
            "Project Local Alias for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/local-alias-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        registry.register_menus::<LocalAliasPlugin>();
        registry
            .register_window_client("Rusty Local Alias Plugin", &self.window)
            .unwrap();
    }

    fn on_project_load(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        CURRENT_ALIAS.lock().unwrap().take();
        self.aliases = project.deserialize("alias_entries").unwrap_or_else(|e| {
            log::warn!("Failed to load alias entries from project: {}", e);
            Vec::new()
        });
        self.send_to_webview("update_aliases", &self.aliases);
    }

    fn on_project_save(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        project.clear_params();
        let _ = project.serialize("alias_entries", &self.aliases);
    }
}

impl LocalAliasPlugin {
    fn init_logging() {
        aviutl2::logger::LogBuilder::new()
            .filter_level(if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .init();
    }

    fn send_to_webview<T: serde::Serialize>(&self, name: &str, data: &T) {
        log::debug!("Sending to webview: {}", name);
        match serde_json::to_value(data) {
            Ok(json) => {
                let json = serde_json::json!({ "type": name, "data": json }).to_string();
                let script = format!(
                    "try {{ window.bridge && window.bridge._emit({json}); }} catch(e) {{ console.error(e); }}"
                );
                let _ = self.webview.evaluate_script(&script);
                log::debug!("Sent to webview: {}", name);
            }
            Err(e) => {
                log::error!("Failed to serialize data for webview: {}", e);
            }
        }
    }

    fn ipc_handler(message_str: String) {
        #[derive(serde::Deserialize, Debug)]
        #[serde(tag = "type", content = "data", rename_all = "snake_case")]
        enum IpcMessage {
            GetVersion,
            GetAliases,
            SetAliases(Vec<AliasEntry>),
            SetCurrentAlias(AliasEntry),
        }

        match serde_json::from_str::<IpcMessage>(&message_str) {
            Ok(msg) => {
                log::debug!("IPC message received: {:?}", msg);
                match msg {
                    IpcMessage::GetVersion => {
                        let version = env!("CARGO_PKG_VERSION");
                        let response = serde_json::json!({ "version": version });
                        LocalAliasPlugin::with_instance(|instance| {
                            instance.send_to_webview("version_response", &response);
                        });
                    }
                    IpcMessage::GetAliases => {
                        LocalAliasPlugin::with_instance(|instance| {
                            let aliases = instance.aliases.clone();
                            instance.send_to_webview("aliases_response", &aliases);
                        });
                    }
                    IpcMessage::SetAliases(new_aliases) => {
                        LocalAliasPlugin::with_instance_mut(|instance| {
                            instance.aliases = new_aliases;
                            instance.send_to_webview("update_aliases", &instance.aliases);
                        });
                    }
                    IpcMessage::SetCurrentAlias(entry) => {
                        let mut current = CURRENT_ALIAS.lock().unwrap();
                        *current = Some(entry);
                    }
                }
            }
            Err(error) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&message_str) {
                    if let Some(ty) = value.get("type").and_then(|v| v.as_str()) {
                        match ty {
                            "set_aliases" => {
                                log::error!(
                                    "Failed to parse aliases from IPC message data: {:?}",
                                    value.get("data")
                                );
                            }
                            "set_current_alias" => {
                                log::error!(
                                    "Failed to parse current alias from IPC message data: {:?}",
                                    value.get("data")
                                );
                            }
                            other => {
                                log::warn!("Unknown IPC message type: {}", other);
                            }
                        }
                    } else {
                        log::error!("Failed to parse IPC message: {}", error);
                    }
                } else {
                    log::error!("Failed to parse IPC message: {}", error);
                }
            }
        }
    }
}

#[aviutl2::generic::menus]
impl LocalAliasPlugin {
    #[object(name = "ローカルエイリアスに追加")]
    fn menu_add_alias(
        &mut self,
        edit_section: &mut aviutl2::generic::EditSection,
    ) -> AnyResult<()> {
        let alias = edit_section
            .get_focused_object()?
            .map(|obj| edit_section.get_object_alias(&obj))
            .transpose()?;
        let Some(alias) = alias else {
            anyhow::bail!("オブジェクトが選択されていません。");
        };
        self.aliases.push(AliasEntry {
            name: "New Alias".to_string(),
            alias,
        });
        self.send_to_webview("update_aliases", &self.aliases);
        Ok(())
    }

    #[layer(name = "ローカルエイリアスを配置")]
    fn menu_insert_alias(
        &mut self,
        edit_section: &mut aviutl2::generic::EditSection,
    ) -> AnyResult<()> {
        let current_alias = CURRENT_ALIAS.lock().unwrap().clone();
        let Some(alias) = current_alias else {
            anyhow::bail!("エイリアスが選択されていません。")
        };
        let info = edit_section.info;
        let length = match (info.select_range_start, info.select_range_end) {
            (Some(start), Some(end)) if end >= start => end - start + 1,
            _ => 1,
        };
        edit_section.create_object_from_alias(&alias.alias, info.layer, info.frame, length)?;
        Ok(())
    }
}

aviutl2::register_generic_plugin!(LocalAliasPlugin);
