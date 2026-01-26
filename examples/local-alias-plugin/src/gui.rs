use crate::{AliasEntry, AliasState};
use aviutl2_eframe::AviUtl2EframeHandle;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub(crate) struct LocalAliasApp {
    state: Arc<Mutex<AliasState>>,
    show_info: bool,
    rename_dialog: Option<RenameDialog>,
    delete_dialog: Option<DeleteDialog>,
    version: String,
    handle: AviUtl2EframeHandle,
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
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        state: Arc<Mutex<AliasState>>,
        handle: AviUtl2EframeHandle,
    ) -> Self {
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

        fonts.font_data.insert(
            "M+ 1 Code".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(mplus::MPLUS1CODE_MEDIUM)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "M+ 1 Code".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        Self {
            state,
            show_info: false,
            rename_dialog: None,
            delete_dialog: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            handle,
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
        let (aliases, selected_index) = self.snapshot();

        // TODO: toolbarの右クリックイベントに右クリックメニューを割り当てる
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let clicked = ui
                    .heading("Rusty Local Alias Plugin")
                    .interact(egui::Sense::click());
                if clicked.secondary_clicked() {
                    let _ = self.handle.show_context_menu();
                }
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
                        let pressed_enter =
                            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
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
        if let Some((index, name)) = rename_action
            && !name.is_empty()
        {
            self.rename_alias(index, name);
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
                        ui.label(format!("エイリアス \"{}\" を削除しますか？", dialog.name));
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
