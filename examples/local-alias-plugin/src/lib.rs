mod ui;
mod window_client;

use aviutl2::{AnyResult, generic::GenericPlugin};
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
use ui::{LocalAliasUiApp, UiState};
use window_client::WindowClient;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AliasEntry {
    name: String,
    alias: String,
}

#[aviutl2::plugin(GenericPlugin)]
pub struct LocalAliasPlugin {
    ui_state: Arc<Mutex<UiState>>,
    ui_repaint: Arc<Mutex<Option<egui::Context>>>,
    _ui_thread: std::thread::JoinHandle<()>,
    window_client: WindowClient,

    edit_handle: Arc<OnceLock<aviutl2::generic::EditHandle>>,

    aliases: Vec<AliasEntry>,
}
unsafe impl Send for LocalAliasPlugin {}
unsafe impl Sync for LocalAliasPlugin {}

pub static CURRENT_ALIAS: Mutex<Option<AliasEntry>> = Mutex::new(None);

impl aviutl2::generic::GenericPlugin for LocalAliasPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Local Alias Plugin...");
        let edit_handle = Arc::new(OnceLock::<aviutl2::generic::EditHandle>::new());
        let window_client = WindowClient::new("Rusty Local Alias Plugin", (800, 600))?;
        let parent_hwnd = window_client.hwnd().0 as isize;

        let ui_state = Arc::new(Mutex::new(UiState::new()));
        let ui_repaint = Arc::new(Mutex::new(None));
        let ui_state_clone = Arc::clone(&ui_state);
        let ui_repaint_clone = Arc::clone(&ui_repaint);
        let ui_thread = std::thread::spawn(move || {
            let mut options = eframe::NativeOptions::default();
            options.viewport = options
                .viewport
                .with_title("Rusty Local Alias Plugin")
                .with_inner_size(egui::vec2(800.0, 600.0));
            #[cfg(windows)]
            {
                options.event_loop_builder = Some(Box::new(|builder| {
                    use winit::platform::windows::EventLoopBuilderExtWindows;
                    builder.with_any_thread(true);
                }));
            }
            log::info!("Starting egui UI thread...");
            if let Err(e) = eframe::run_native(
                "Rusty Local Alias Plugin",
                options,
                Box::new(move |cc| {
                    log::info!("egui context initialized");
                    if !egui::FontDefinitions::default()
                        .font_data
                        .contains_key("M+ 1")
                    {
                        let mut fonts = egui::FontDefinitions::default();
                        fonts.font_data.insert(
                            "M+ 1".to_owned(),
                            std::sync::Arc::new(egui::FontData::from_static(mplus::MPLUS1_REGULAR)),
                        );
                        fonts
                            .families
                            .get_mut(&egui::FontFamily::Proportional)
                            .unwrap()
                            .insert(0, "M+ 1".to_owned());
                        cc.egui_ctx.set_fonts(fonts);
                    }
                    Ok(Box::new(LocalAliasUiApp::new(
                        ui_state_clone,
                        ui_repaint_clone,
                        parent_hwnd,
                    )))
                }),
            ) {
                log::error!("Failed to run egui UI: {}", e);
            }
        });

        Ok(LocalAliasPlugin {
            ui_state,
            ui_repaint,
            _ui_thread: ui_thread,
            window_client,
            edit_handle,

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
        if let Err(e) =
            registry.register_window_client("Rusty Local Alias Plugin", &self.window_client)
        {
            log::error!("Failed to register window client: {e}");
        }
    }

    fn on_project_load(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        self.aliases = project.deserialize("alias_entries").unwrap_or_else(|e| {
            log::warn!("Failed to load alias entries from project: {}", e);
            Vec::new()
        });
        self.update_ui_aliases();
    }

    fn on_project_save(&mut self, project: &mut aviutl2::generic::ProjectFile) {
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

    fn update_ui_aliases(&self) {
        if let Ok(mut state) = self.ui_state.lock() {
            state.aliases = self.aliases.clone();
            if let Some(selected) = state.selected_index
                && selected >= state.aliases.len()
            {
                state.selected_index = None;
            }
            ui::sync_current_alias(&state);
        }
        self.request_ui_repaint();
    }

    fn request_ui_repaint(&self) {
        if let Ok(slot) = self.ui_repaint.lock()
            && let Some(ctx) = slot.as_ref()
        {
            ctx.request_repaint();
        }
    }

    fn add_alias_from_focus() -> anyhow::Result<Option<AliasEntry>> {
        LocalAliasPlugin::with_instance(|instance| {
            let handle = instance.edit_handle.get().unwrap();
            handle
                .call_edit_section(|section| {
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
                .map_err(anyhow::Error::from)
        })
        .flatten()
    }
}

aviutl2::register_generic_plugin!(LocalAliasPlugin);
