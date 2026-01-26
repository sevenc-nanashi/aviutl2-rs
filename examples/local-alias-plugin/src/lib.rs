use aviutl2::AnyResult;
use std::sync::{Arc, Mutex};

mod gui;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AliasEntry {
    name: String,
    alias: String,
}

#[derive(Default)]
pub(crate) struct AliasState {
    aliases: Vec<AliasEntry>,
    selected_index: Option<usize>,
}

impl AliasState {
    fn set_aliases(&mut self, aliases: Vec<AliasEntry>) {
        self.aliases = aliases;
        self.clamp_selection();
    }

    fn set_selected_index(&mut self, index: Option<usize>) {
        self.selected_index = index;
        self.clamp_selection();
    }

    fn add_alias(&mut self, alias: AliasEntry) {
        self.aliases.push(alias);
        update_current_alias(self);
    }

    fn rename_alias(&mut self, index: usize, name: String) {
        if let Some(alias) = self.aliases.get_mut(index) {
            alias.name = name;
            update_current_alias(self);
        }
    }

    fn delete_alias(&mut self, index: usize) {
        if index >= self.aliases.len() {
            return;
        }
        self.aliases.remove(index);
        if let Some(selected) = self.selected_index {
            if selected == index {
                self.selected_index = None;
            } else if selected > index {
                self.selected_index = Some(selected - 1);
            }
        }
        update_current_alias(self);
    }

    fn move_alias(&mut self, index: usize, dir: i32) {
        if index >= self.aliases.len() {
            return;
        }
        let new_index = if dir < 0 {
            match index.checked_sub(1) {
                Some(idx) => idx,
                None => return,
            }
        } else {
            index + 1
        };
        if new_index >= self.aliases.len() {
            return;
        }
        let item = self.aliases.remove(index);
        self.aliases.insert(new_index, item);
        if self.selected_index == Some(index) {
            self.selected_index = Some(new_index);
        }
        update_current_alias(self);
    }

    fn clamp_selection(&mut self) {
        if let Some(index) = self.selected_index
            && index >= self.aliases.len()
        {
            self.selected_index = None;
        }
        update_current_alias(self);
    }
}

fn update_current_alias(state: &AliasState) {
    let current = state
        .selected_index
        .and_then(|index| state.aliases.get(index).cloned());
    *CURRENT_ALIAS.lock().unwrap() = current;
}

pub static CURRENT_ALIAS: Mutex<Option<AliasEntry>> = Mutex::new(None);

#[aviutl2::plugin(GenericPlugin)]
pub struct LocalAliasPlugin {
    window: aviutl2_eframe::EframeWindow,
    state: Arc<Mutex<AliasState>>,
}
unsafe impl Send for LocalAliasPlugin {}
unsafe impl Sync for LocalAliasPlugin {}

impl aviutl2::generic::GenericPlugin for LocalAliasPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Local Alias Plugin...");
        let state = Arc::new(Mutex::new(AliasState::default()));
        let ui_state = Arc::clone(&state);
        let window =
            aviutl2_eframe::EframeWindow::new("RustyLocalAliasPlugin", move |cc, handle| {
                Ok(Box::new(gui::LocalAliasApp::new(cc, ui_state, handle)))
            })?;

        Ok(LocalAliasPlugin { window, state })
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
        let aliases = project.deserialize("alias_entries").unwrap_or_else(|e| {
            log::warn!("Failed to load alias entries from project: {}", e);
            Vec::new()
        });
        let mut state = self.state.lock().unwrap();
        state.set_aliases(aliases);
        state.set_selected_index(None);
        self.window.egui_ctx().request_repaint();
    }

    fn on_project_save(&mut self, project: &mut aviutl2::generic::ProjectFile) {
        project.clear_params();
        let aliases = self.state.lock().unwrap().aliases.clone();
        let _ = project.serialize("alias_entries", &aliases);
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
        self.state.lock().unwrap().add_alias(AliasEntry {
            name: "New Alias".to_string(),
            alias,
        });
        self.window.egui_ctx().request_repaint();
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
