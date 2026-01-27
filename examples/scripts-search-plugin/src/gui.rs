use aviutl2::generic::Effect;
use aviutl2_eframe::{AviUtl2EframeHandle, eframe, egui};
use std::sync::{Arc, Mutex};

pub(crate) struct ScriptsSearchApp {
    show_info: bool,
    version: String,
    handle: AviUtl2EframeHandle,
}

impl ScriptsSearchApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, handle: AviUtl2EframeHandle) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "M+ 1".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(mplus::MPLUS1_REGULAR)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .expect("Failed to get Proportional font family")
            .insert(0, "M+ 1".to_owned());

        fonts.font_data.insert(
            "M+ 1 Code".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(mplus::MPLUS1CODE_MEDIUM)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .expect("Failed to get Monospace font family")
            .insert(0, "M+ 1 Code".to_owned());

        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals = aviutl2_eframe::aviutl2_visuals();
        });
        cc.egui_ctx.set_fonts(fonts);

        Self {
            show_info: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
            handle,
        }
    }
}

impl eframe::App for ScriptsSearchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: toolbarの右クリックイベントに右クリックメニューを割り当てる
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let clicked = ui
                    .heading("Rusty Scripts Search Plugin")
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

        egui::CentralPanel::default().show(ctx, |ui| match crate::EFFECTS.get() {
            None => {
                ui.label("エフェクト情報を読み込み中...");
            }
            Some(effects) => {
                ui.label(format!("登録されているエフェクト数: {}", effects.len()));
                ui.add_space(8.0);
                let mut search_text = String::new();
                egui::TextEdit::singleline(&mut search_text)
                    .hint_text("検索...")
                    .show(ui);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for effect in effects.iter() {
                        ui.group(|ui| {
                            ui.label(format!("名前: {}", effect.name));
                            ui.label(format!("フラグ: {:?}", effect.flag));
                            ui.label(format!("タイプ: {:?}", effect.effect_type));
                        });
                        ui.add_space(4.0);
                    }
                });
            }
        });

        if self.show_info {
            let mut open = true;
            egui::Window::new("Rusty Scripts Search Plugin")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(format!("バージョン: {}", self.version));
                    ui.label(
                        "オブジェクト・エフェクトを検索するプラグイン。",
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
                        "examples/objects-search-plugin",
                        format!(
                            "https://github.com/sevenc-nanashi/aviutl2-rs/tree/{}/examples/objects-search-plugin",
                            self.version
                        ),
                    );
                });
            if !open {
                self.show_info = false;
            }
        }
    }
}
