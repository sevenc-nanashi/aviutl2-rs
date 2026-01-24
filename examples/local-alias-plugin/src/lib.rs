use aviutl2::AnyResult;
use aviutl2::raw_window_handle::HasWindowHandle;
use eframe::egui;
use std::{num::NonZeroIsize, sync::{Arc, Mutex}};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AliasEntry {
    name: String,
    alias: String,
}

#[derive(Default)]
struct AliasState {
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
        if let Some(index) = self.selected_index {
            if index >= self.aliases.len() {
                self.selected_index = None;
            }
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

struct LocalAliasApp {
    state: Arc<Mutex<AliasState>>,
    show_info: bool,
    rename_dialog: Option<RenameDialog>,
    delete_dialog: Option<DeleteDialog>,
    version: String,
    hwnd: Option<NonZeroIsize>,
}

struct RenameDialog {
    index: usize,
    buffer: String,
}

struct DeleteDialog {
    index: usize,
    name: String,
}

impl LocalAliasApp {
    fn new(cc: &eframe::CreationContext<'_>, state: Arc<Mutex<AliasState>>) -> Self {
        let hwnd = cc
            .window_handle()
            .ok()
            .and_then(|handle| match handle.as_raw() {
                aviutl2::raw_window_handle::RawWindowHandle::Win32(handle) => Some(handle.hwnd),
                _ => None,
            });
        Self {
            state,
            show_info: false,
            rename_dialog: None,
            delete_dialog: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            hwnd,
        }
    }

    fn with_state<T>(&self, f: impl FnOnce(&mut AliasState) -> T) -> T {
        let mut state = self.state.lock().unwrap();
        f(&mut state)
    }

    fn snapshot(&self) -> (Vec<AliasEntry>, Option<usize>) {
        let state = self.state.lock().unwrap();
        (state.aliases.clone(), state.selected_index)
    }

    fn set_selected_index(&self, index: Option<usize>) {
        self.with_state(|state| state.set_selected_index(index));
    }

    fn rename_alias(&self, index: usize, name: String) {
        self.with_state(|state| state.rename_alias(index, name));
    }

    fn delete_alias(&self, index: usize) {
        self.with_state(|state| state.delete_alias(index));
    }

    fn move_alias(&self, index: usize, dir: i32) {
        self.with_state(|state| state.move_alias(index, dir));
    }
}

impl eframe::App for LocalAliasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.wants_keyboard_input() {
            if let Some(hwnd) = self.hwnd {
                aviutl2_egui::focus_hwnd(hwnd);
            }
        }
        let (aliases, selected_index) = self.snapshot();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Rusty Local Alias Plugin");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("?").clicked() {
                        self.show_info = true;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if aliases.is_empty() {
                ui.label("エイリアスがありません。オブジェクトを選択して「ローカルエイリアスに追加」メニューで追加してください。");
                return;
            }

            for (index, alias) in aliases.iter().enumerate() {
                let selected = selected_index == Some(index);
                let frame = egui::Frame::group(ui.style())
                    .fill(if selected {
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().faint_bg_color
                    })
                    .stroke(if selected {
                        ui.visuals().selection.stroke
                    } else {
                        ui.visuals().widgets.noninteractive.bg_stroke
                    });
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let select_button =
                            egui::Button::new(&alias.name).selected(selected).frame(false);
                        if ui.add(select_button).clicked() {
                            self.set_selected_index(Some(index));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(index + 1 < aliases.len(), egui::Button::new("下へ"))
                                .clicked()
                            {
                                self.move_alias(index, 1);
                            }
                            if ui
                                .add_enabled(index > 0, egui::Button::new("上へ"))
                                .clicked()
                            {
                                self.move_alias(index, -1);
                            }
                            if ui.button("削除").clicked() {
                                self.delete_dialog = Some(DeleteDialog {
                                    index,
                                    name: alias.name.clone(),
                                });
                            }
                            if ui.button("名前変更").clicked() {
                                self.rename_dialog = Some(RenameDialog {
                                    index,
                                    buffer: alias.name.clone(),
                                });
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            }
        });

        if self.show_info {
            let mut open = true;
            egui::Window::new("Rusty Local Alias Plugin")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(format!("バージョン: {}", self.version));
                    ui.label(
                        "プロジェクトローカルなエイリアスを管理するAviUtl2プラグイン。\nここでエイリアスを選択した後、「ローカルエイリアスを配置」メニューで配置してください。",
                    );
                    ui.add_space(8.0);
                    ui.label("Developed by");
                    ui.hyperlink_to("Nanashi.", "https://sevenc7c.com");
                    ui.add_space(4.0);
                    ui.label("Source Code:");
                    ui.hyperlink_to(
                        "sevenc-nanashi/aviutl2-rs",
                        "https://github.com/sevenc-nanashi/aviutl2-rs",
                    );
                    ui.hyperlink_to(
                        "examples/local-alias-plugin",
                        format!(
                            "https://github.com/sevenc-nanashi/aviutl2-rs/tree/{}/examples/local-alias-plugin",
                            self.version
                        ),
                    );
                });
            if !open {
                self.show_info = false;
            }
        }

        let mut rename_action = None;
        if let Some(dialog) = self.rename_dialog.as_mut() {
            if dialog.index >= aliases.len() {
                self.rename_dialog = None;
            } else {
                let mut open = true;
                let mut save = false;
                let mut cancel = false;
                egui::Window::new("名前変更")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.label("新しいエイリアス名");
                        let response = ui.text_edit_singleline(&mut dialog.buffer);
                        let pressed_enter = response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        ui.horizontal(|ui| {
                            if ui.button("保存").clicked() || pressed_enter {
                                save = true;
                            }
                            if ui.button("キャンセル").clicked() {
                                cancel = true;
                            }
                        });
                    });
                if save {
                    rename_action = Some((dialog.index, dialog.buffer.trim().to_string()));
                    open = false;
                }
                if cancel {
                    open = false;
                }
                if !open {
                    self.rename_dialog = None;
                }
            }
        }
        if let Some((index, name)) = rename_action {
            if !name.is_empty() {
                self.rename_alias(index, name);
            }
        }

        let mut delete_action = None;
        if let Some(dialog) = self.delete_dialog.as_ref() {
            if dialog.index >= aliases.len() {
                self.delete_dialog = None;
            } else {
                let mut open = true;
                let mut cancel = false;
                let mut confirm_delete = false;
                egui::Window::new("削除")
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "エイリアス \"{}\" を削除しますか？",
                            dialog.name
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("削除").clicked() {
                                confirm_delete = true;
                            }
                            if ui.button("キャンセル").clicked() {
                                cancel = true;
                            }
                        });
                    });
                if confirm_delete {
                    delete_action = Some(dialog.index);
                    open = false;
                }
                if cancel {
                    open = false;
                }
                if !open {
                    self.delete_dialog = None;
                }
            }
        }
        if let Some(index) = delete_action {
            self.delete_alias(index);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::from_rgb(20, 24, 33).to_normalized_gamma_f32()
    }
}

#[aviutl2::plugin(GenericPlugin)]
pub struct LocalAliasPlugin {
    window: aviutl2_egui::EguiWindow,
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
        let window = aviutl2_egui::spawn_eframe_window(
            "Rusty Local Alias Plugin",
            (800.0, 600.0),
            move |cc| Ok(Box::new(LocalAliasApp::new(cc, ui_state))),
        )?;

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
        self.window
            .ensure_embedded_with_parent_title("AviUtl ExEdit2");
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
