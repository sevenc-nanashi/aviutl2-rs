mod entry;
mod ui;

use crate::entry::DummyObject;
use aviutl2::{
    AnyResult,
    generic::{GenericPlugin, SubPlugin},
};
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
use ui::{LocalAliasUiApp, UiState};

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

    dummy: SubPlugin<DummyObject>,

    edit_handle: Arc<OnceLock<aviutl2::generic::EditHandle>>,
    _replace_thread: std::thread::JoinHandle<()>,
    replace_flag: std::sync::mpsc::Sender<()>,

    aliases: Vec<AliasEntry>,
}
unsafe impl Send for LocalAliasPlugin {}
unsafe impl Sync for LocalAliasPlugin {}

pub static CURRENT_ALIAS: Mutex<Option<AliasEntry>> = Mutex::new(None);

impl aviutl2::generic::GenericPlugin for LocalAliasPlugin {
    fn new(info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Local Alias Plugin...");
        let edit_handle = Arc::new(OnceLock::<aviutl2::generic::EditHandle>::new());

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
            if let Err(e) = eframe::run_native(
                "Rusty Local Alias Plugin",
                options,
                Box::new(move |cc| {
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
                    )))
                }),
            ) {
                log::error!("Failed to run egui UI: {}", e);
            }
        });

        let (replace_flag_tx, replace_flag_rx) = std::sync::mpsc::channel();
        let replace_thread = Self::spawn_replace_thread(Arc::clone(&edit_handle), replace_flag_rx);

        Ok(LocalAliasPlugin {
            ui_state,
            ui_repaint,
            _ui_thread: ui_thread,
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
        registry.register_filter_plugin(&self.dummy);
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
