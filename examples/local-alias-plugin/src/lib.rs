mod ui;

use aviutl2::{AnyResult, generic::GenericPlugin};
use std::sync::{Arc, Mutex, OnceLock};
use ui::UiHandle;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AliasEntry {
    name: String,
    alias: String,
}

#[aviutl2::plugin(GenericPlugin)]
pub struct LocalAliasPlugin {
    ui_handle: Arc<UiHandle>,

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
        let ui_handle = Arc::new(UiHandle::new());

        Ok(LocalAliasPlugin {
            ui_handle,
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
        if let Err(e) = registry.register_window_client("Rusty Local Alias Plugin", &self.ui_handle)
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
        let _ = self.ui_handle.with_state(|state| {
            state.aliases = self.aliases.clone();
            if let Some(selected) = state.selected_index
                && selected >= state.aliases.len()
            {
                state.selected_index = None;
            }
            ui::sync_current_alias(&state);
        });
        self.ui_handle.request_repaint();
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
