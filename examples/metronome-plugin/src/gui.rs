use aviutl2_eframe::{AviUtl2EframeHandle, eframe, egui};
use std::{collections::VecDeque, time::Instant};

const MAX_TAP_INTERVAL_SECS: f64 = 3.0;
const MAX_INTERVALS: usize = 8;

pub(crate) struct MetronomeApp {
    show_info: bool,
    suppress_info_close_once: bool,
    version: String,
    handle: AviUtl2EframeHandle,
    last_tap: Option<Instant>,
    tap_intervals: VecDeque<f64>,
    bpm: Option<f64>,
    will_reset_on_next_tap: bool,
}

impl MetronomeApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, handle: AviUtl2EframeHandle) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "M+ 1p".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(mplus::MPLUS_1P_REGULAR)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .expect("Failed to get Proportional font family")
            .insert(0, "M+ 1p".to_owned());

        fonts.font_data.insert(
            "M+ 1m".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(mplus::MPLUS_1M_REGULAR)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .expect("Failed to get Monospace font family")
            .insert(0, "M+ 1m".to_owned());

        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals = aviutl2_eframe::aviutl2_visuals();
        });
        cc.egui_ctx.set_fonts(fonts);

        Self {
            show_info: false,
            suppress_info_close_once: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
            handle,
            last_tap: None,
            tap_intervals: VecDeque::new(),
            bpm: None,
            will_reset_on_next_tap: false,
        }
    }
}

impl eframe::App for MetronomeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 常に再描画を要求して、リアルタイムに反応するようにする
        ctx.request_repaint();

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.register_tap();
        }
        self.render_toolbar(ctx);
        self.render_main_panel(ctx);
        self.render_info_window(ctx);
    }
}

impl MetronomeApp {
    fn render_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let clicked = ui
                    .heading("Rusty Metronome Plugin")
                    .interact(egui::Sense::click());
                if clicked.secondary_clicked() {
                    let _ = self.handle.show_context_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let resp = ui
                        .add_sized(
                            egui::vec2(
                                ui.text_style_height(&egui::TextStyle::Heading),
                                ui.text_style_height(&egui::TextStyle::Heading),
                            ),
                            egui::Button::new("i"),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("プラグイン情報");
                    if resp.clicked() {
                        self.show_info = true;
                        self.suppress_info_close_once = true;
                    }
                });
            });
        });
    }

    fn render_main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                if let Some(last_tap) = self.last_tap {
                    let since = last_tap.elapsed().as_secs_f64();
                    if since > MAX_TAP_INTERVAL_SECS {
                        self.will_reset_on_next_tap = true;
                    }
                }

                let bpm_text = self
                    .bpm
                    .map(|bpm| format!("{bpm:.2} BPM"))
                    .unwrap_or_else(|| "---.-- BPM".to_string());
                ui.label(egui::RichText::new(bpm_text).size(28.0).color(
                    if self.will_reset_on_next_tap {
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().text_color()
                    },
                ));
                ui.add_space(12.0);
                // Shiftキーを押してるときはプロジェクトからBPMを持ってくる
                if ui.input(|i| i.modifiers.shift) {
                    let set_button = egui::Button::new("BPM取得").min_size(egui::vec2(160.0, 48.0));
                    if ui
                        .add(set_button)
                        .on_hover_text("プロジェクトからBPMを取得します")
                        .clicked()
                    {
                        let info = crate::EDIT_HANDLE.get().unwrap().get_edit_info();
                        self.bpm = Some(info.grid_bpm_tempo as f64);
                        self.will_reset_on_next_tap = true;
                    }
                } else {
                    let tap_button = egui::Button::new(if self.will_reset_on_next_tap {
                        "Reset"
                    } else {
                        "Tap"
                    })
                    .min_size(egui::vec2(160.0, 48.0));
                    if ui
                        .add(tap_button)
                        .on_hover_text("Spaceキーでもタップできます")
                        .clicked()
                    {
                        self.register_tap();
                    }
                }
                ui.add_space(8.0);
                ui.columns(3, |columns| {
                    if columns[0]
                        .add_enabled(self.bpm.is_some(), egui::Button::new("÷ 2"))
                        .clicked()
                    {
                        self.bpm = self.bpm.map(|bpm| bpm / 2.0);
                    }
                    if columns[1].button("リセット").clicked() {
                        self.reset_taps();
                    }
                    if columns[2]
                        .add_enabled(self.bpm.is_some(), egui::Button::new("× 2"))
                        .clicked()
                    {
                        self.bpm = self.bpm.map(|bpm| bpm * 2.0);
                    }
                });
                ui.add_space(8.0);
                ui.columns(2, |columns| {
                    if columns[0]
                        .add_enabled(self.bpm.is_some(), egui::Button::new("0:00を基準に反映"))
                        .clicked()
                    {
                        self.apply_bpm_to_host_origin();
                    }
                    if columns[1]
                        .add_enabled(
                            self.bpm.is_some(),
                            egui::Button::new("現在位置を基準に反映"),
                        )
                        .clicked()
                    {
                        self.apply_bpm_to_host_relative();
                    }
                });
            });
        });
    }

    fn render_info_window(&mut self, ctx: &egui::Context) {
        if !self.show_info {
            return;
        }
        let screen_rect = ctx.screen_rect();
        let dim_color = egui::Color32::from_black_alpha(128);
        let dim_response = egui::Area::new(egui::Id::new("info_window_dim_layer"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                ui.set_min_size(screen_rect.size());
                let (rect, response) =
                    ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
                ui.painter().rect_filled(rect, 0.0, dim_color);
                response
            })
            .inner;
        let mut open = true;
        let response = egui::Window::new("Rusty Metronome Plugin")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label(format!("バージョン: {}", self.version));
                ui.label("BPMを合わせるタップボタンとメトロノームのエフェクトを提供します。");
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
                    "examples/metronome-plugin",
                    format!(
                        "https://github.com/sevenc-nanashi/aviutl2-rs/tree/{}/examples/metronome-plugin",
                        self.version
                    ),
                );
            });
        if self.suppress_info_close_once {
            self.suppress_info_close_once = false;
        } else if dim_response.clicked() {
            self.show_info = false;
        } else if let Some(response) = response
            && response.response.clicked_elsewhere()
        {
            self.show_info = false;
        }
        if !open {
            self.show_info = false;
        }
    }

    fn register_tap(&mut self) {
        if self.will_reset_on_next_tap {
            self.reset_taps();
            self.will_reset_on_next_tap = false;
        }
        let now = Instant::now();
        if let Some(last_tap) = self.last_tap {
            let delta = now.duration_since(last_tap).as_secs_f64();
            self.tap_intervals.push_back(delta);
            while self.tap_intervals.len() > MAX_INTERVALS {
                self.tap_intervals.pop_front();
            }
            let avg =
                self.tap_intervals.iter().copied().sum::<f64>() / (self.tap_intervals.len() as f64);
            if avg > 0.0 {
                self.bpm = Some(60.0 / avg);
            }
        }
        self.last_tap = Some(now);
    }

    fn reset_taps(&mut self) {
        self.last_tap = None;
        self.tap_intervals.clear();
        self.bpm = None;
        self.will_reset_on_next_tap = false;
    }

    fn apply_bpm_to_host_origin(&self) {
        if let Some(bpm) = self.bpm {
            let res = crate::EDIT_HANDLE.get().unwrap().call_edit_section(|edit| {
                // TODO: 拍子情報も変更できるようにする
                edit.set_grid_bpm(bpm as f32, 4, 0.0)
            });
            log::info!("Applied BPM: {:?}", res);
        }
    }

    fn apply_bpm_to_host_relative(&self) {
        if let Some(bpm) = self.bpm {
            let res = crate::EDIT_HANDLE.get().unwrap().call_edit_section(|edit| {
                let current_frame = edit.info.frame;
                let fps = *edit.info.fps.numer() as f32 / *edit.info.fps.denom() as f32;
                // TODO: 拍子情報も変更できるようにする
                edit.set_grid_bpm(bpm as f32, 4, current_frame as f32 / fps)
            });
            log::info!("Applied BPM: {:?}", res);
        }
    }
}
