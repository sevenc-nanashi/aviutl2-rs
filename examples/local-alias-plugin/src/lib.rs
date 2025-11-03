mod entry;
mod ws_popup;
use std::sync::{Arc, Mutex, OnceLock};

use aviutl2::{AnyResult, odbg};
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
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        env_logger::Builder::new()
            .parse_filters("debug")
            .target(aviutl2::utils::debug_logger_target())
            .init();
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
                #[derive(serde::Deserialize)]
                struct IpcMessage {
                    r#type: String,
                    data: serde_json::Value,
                }

                let message_str = payload.into_body();
                if let Ok(msg) = serde_json::from_str::<IpcMessage>(&message_str) {
                    log::debug!("IPC message received: {:?}", msg.r#type);
                    match msg.r#type.as_str() {
                        "get_version" => {
                            let version = env!("CARGO_PKG_VERSION");
                            let response = serde_json::json!({ "version": version });
                            LocalAliasPlugin::with_instance(|instance| {
                                instance.send_to_webview("version_response", &response);
                            });
                        }
                        "get_aliases" => {
                            LocalAliasPlugin::with_instance(|instance| {
                                let aliases = instance.aliases.clone();
                                instance.send_to_webview("aliases_response", &aliases);
                            });
                        }
                        "set_aliases" => {
                            if let Ok(new_aliases) =
                                serde_json::from_value::<Vec<AliasEntry>>(msg.data.clone())
                            {
                                LocalAliasPlugin::with_instance_mut(|instance| {
                                    instance.aliases = new_aliases;
                                });
                            } else {
                                log::error!(
                                    "Failed to parse aliases from IPC message data: {:?}",
                                    msg.data
                                );
                            }
                        }
                        "add_alias" => {
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
                                        instance
                                            .send_to_webview("update_aliases", &instance.aliases);
                                    });
                                }
                                Ok(None) => {
                                    log::warn!(
                                        "No focused object to create alias from in add_alias"
                                    );
                                }
                                Err(e) => {
                                    log::error!("Failed to add alias: {}", e);
                                }
                            }
                        }
                        "set_current_alias" => {
                            if let Ok(entry) =
                                serde_json::from_value::<AliasEntry>(msg.data.clone())
                            {
                                let mut current = CURRENT_ALIAS.lock().unwrap();
                                *current = Some(entry);
                            } else {
                                log::error!(
                                    "Failed to parse current alias from IPC message data: {:?}",
                                    msg.data
                                );
                            }
                        }
                        other => {
                            log::warn!("Unknown IPC message type: {}", other);
                        }
                    }
                } else {
                    log::error!("Failed to parse IPC message: {message_str:?}");
                }
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
        let replace_thread = std::thread::spawn({
            let edit_handle = Arc::clone(&edit_handle);
            move || {
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
                                            "Local Alias (Debug)"
                                        } else {
                                            "Local Alias"
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
            }
        });

        Ok(LocalAliasPlugin {
            webview,
            window,
            edit_handle,

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
        registry.register_filter_plugin::<entry::DummyObject>();
    }

    fn on_project_load(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        self.aliases = project
            .deserialize("alias_entries")
            .unwrap_or_else(|e| {
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
}

aviutl2::register_generic_plugin!(LocalAliasPlugin);
