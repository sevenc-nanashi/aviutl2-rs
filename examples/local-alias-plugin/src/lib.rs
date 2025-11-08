mod entry;
mod ws_popup;

use crate::entry::DummyObject;
use aviutl2::{
    AnyResult,
    generic::{GenericPlugin, SubPlugin},
    odbg,
};
use std::sync::{Arc, Mutex, OnceLock};
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

    dummy: SubPlugin<DummyObject>,

    edit_handle: Arc<OnceLock<aviutl2::generic::EditHandle>>,
    _replace_thread: std::thread::JoinHandle<()>,
    replace_flag: std::sync::mpsc::Sender<()>,

    aliases: Vec<AliasEntry>,
}
unsafe impl Send for LocalAliasPlugin {}
unsafe impl Sync for LocalAliasPlugin {}

static WEB_CONTENT: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/page/dist");

pub static CURRENT_ALIAS: Mutex<Option<AliasEntry>> = Mutex::new(None);

impl aviutl2::generic::GenericPlugin for LocalAliasPlugin {
    fn new(info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Local Alias Plugin...");
        let edit_handle = Arc::new(OnceLock::<aviutl2::generic::EditHandle>::new());

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
                            odbg!(path);
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
                        .with_url("app://index.html")
                }
            })
            .build(&window)?;

        let (replace_flag_tx, replace_flag_rx) = std::sync::mpsc::channel();
        let replace_thread = Self::spawn_replace_thread(Arc::clone(&edit_handle), replace_flag_rx);

        Ok(LocalAliasPlugin {
            webview,
            window,
            edit_handle,

            dummy: SubPlugin::new_filter_plugin(info)?,

            _replace_thread: replace_thread,
            replace_flag: replace_flag_tx,

            aliases: Vec::new(),
        })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.set_plugin_information(&format!(
            "Project Local Alias for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/local-alias-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        let handle = registry.create_edit_handle();
        let _ = self.edit_handle.set(handle);
        registry
            .register_window_client("Rusty Local Alias Plugin", &self.window)
            .unwrap();
        registry.register_filter_plugin(&self.dummy);
    }

    fn on_project_load(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        self.aliases = project.deserialize("alias_entries").unwrap_or_else(|e| {
            log::warn!("Failed to load alias entries from project: {}", e);
            Vec::new()
        });
        self.send_to_webview("update_aliases", &self.aliases);
    }

    fn on_project_save(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        let _ = project.serialize("alias_entries", &self.aliases);
    }
}

impl LocalAliasPlugin {
    fn init_logging() {
        let _ = env_logger::Builder::new()
            .parse_filters("debug")
            .target(aviutl2::utils::debug_logger_target())
            .try_init();
    }

    fn spawn_replace_thread(
        edit_handle: Arc<OnceLock<aviutl2::generic::EditHandle>>,
        replace_flag_rx: std::sync::mpsc::Receiver<()>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            loop {
                // Wait for a replace signal
                if replace_flag_rx.recv().is_err() {
                    break;
                }

                let current_alias = CURRENT_ALIAS.lock().unwrap().clone();
                if let Some(alias) = current_alias {
                    let _ = edit_handle.wait().call_edit_section(move |section| {
                        for layer in section.layers() {
                            for (_, obj) in layer.objects() {
                                let obj = section.object(&obj);
                                let res = obj.get_effect_item(
                                    if cfg!(debug_assertions) {
                                        "Rusty Local Alias (Debug)"
                                    } else {
                                        "Rusty Local Alias"
                                    },
                                    0,
                                    "Marker",
                                );
                                if res.is_ok() {
                                    let position = obj.get_layer_frame()?;
                                    obj.delete_object()?;

                                    section.create_object_from_alias(
                                        &alias.alias,
                                        position.layer,
                                        position.start,
                                        position.end - position.start + 1,
                                    )?;
                                }
                            }
                        }

                        anyhow::Ok(())
                    });
                }
            }
        })
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
        #[serde(tag = "type", content = "data")]
        enum IpcMessage {
            #[serde(rename = "get_version")]
            GetVersion,
            #[serde(rename = "get_aliases")]
            GetAliases,
            #[serde(rename = "set_aliases")]
            SetAliases(Vec<AliasEntry>),
            #[serde(rename = "add_alias")]
            AddAlias,
            #[serde(rename = "set_current_alias")]
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
                    IpcMessage::AddAlias => {
                        let new_alias = LocalAliasPlugin::with_instance(|instance| {
                            let handle = instance.edit_handle.get().unwrap();
                            handle.call_edit_section(|section| {
                                let alias = section
                                    .get_focused_object()?
                                    .map(|obj| section.get_object_alias(&obj))
                                    .transpose()?;
                                let entry = alias.map(|alias| AliasEntry {
                                    name: "New Alias".to_string(),
                                    alias,
                                });
                                anyhow::Ok(entry)
                            })
                        })
                        .flatten();
                        match new_alias {
                            Ok(Some(entry)) => {
                                LocalAliasPlugin::with_instance_mut(|instance| {
                                    instance.aliases.push(entry.clone());
                                    instance.send_to_webview("update_aliases", &instance.aliases);
                                });
                            }
                            Ok(None) => {
                                log::warn!("No focused object to create alias from in add_alias");
                            }
                            Err(e) => {
                                log::error!("Failed to add alias: {}", e);
                            }
                        }
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

aviutl2::register_generic_plugin!(LocalAliasPlugin);
