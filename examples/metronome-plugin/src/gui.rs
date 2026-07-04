use aviutl2::config::translate as tr;
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
    bpm_text_input: String,
    will_reset_on_next_tap: bool,
    header_collapsed: bool,
}

impl MetronomeApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>, handle: AviUtl2EframeHandle) -> Self {
        let header_collapsed = cc
            .egui_ctx
            .data_mut(|data| data.get_persisted::<bool>(egui::Id::new("header_collapsed")))
            .unwrap_or(false);
        let fonts = aviutl2_eframe::aviutl2_fonts();

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
            bpm_text_input: String::new(),
            will_reset_on_next_tap: false,
            header_collapsed,
        }
    }
}

impl eframe::App for MetronomeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.register_tap();
        }
        if self.header_collapsed {
            self.render_collapsed_header(ui);
        } else {
            self.render_toolbar(ui);
        }
        self.render_main_panel(ui);
        self.render_info_window(ui);
        ui.data_mut(|data| {
            data.insert_persisted(egui::Id::new("header_collapsed"), self.header_collapsed);
        });
    }
}

impl MetronomeApp {
    fn render_collapsed_header(&mut self, ui: &mut egui::Ui) {
        let toolbar = egui::Panel::top("header")
            .exact_size(8.0)
            .show_inside(ui, |_ui| {});
        let response = toolbar
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.hovered() {
            let hover_color = egui::Color32::from_white_alpha(32);
            response.ctx.layer_painter(response.layer_id).rect_filled(
                response.rect,
                0.0,
                hover_color,
            );
        }
        if response.interact(egui::Sense::click()).clicked() {
            self.header_collapsed = false;
        }
    }
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let clicked = ui
                    .heading(tr("Rusty Metronome Plugin"))
                    .interact(egui::Sense::click());
                if clicked.secondary_clicked() {
                    let _ = self.handle.show_context_menu();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let info = ui
                        .add_sized(
                            egui::vec2(
                                ui.text_style_height(&egui::TextStyle::Heading),
                                ui.text_style_height(&egui::TextStyle::Heading),
                            ),
                            egui::Button::new("i"),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(tr("プラグイン情報"));
                    if info.clicked() {
                        self.show_info = true;
                        self.suppress_info_close_once = true;
                    }

                    let collapse = ui
                        .add_sized(
                            egui::vec2(
                                ui.text_style_height(&egui::TextStyle::Heading),
                                ui.text_style_height(&egui::TextStyle::Heading),
                            ),
                            egui::Button::new("^"),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(tr("ヘッダーを折りたたむ"));
                    if collapse.clicked() {
                        self.header_collapsed = true;
                    }
                });
            });
        });
    }

    fn render_main_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                if let Some(last_tap) = self.last_tap {
                    let since = last_tap.elapsed().as_secs_f64();
                    if since > MAX_TAP_INTERVAL_SECS {
                        self.will_reset_on_next_tap = true;
                    } else {
                        ui.request_repaint();
                    }
                }
                let bpm_input_id = ui.make_persistent_id("bpm_input");
                if !ui.memory(|m| m.has_focus(bpm_input_id)) {
                    self.bpm_text_input =
                        self.bpm.map(|bpm| format!("{bpm:.2}")).unwrap_or_default();
                }
                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let button_width = 40.0;
                    let response = ui.add_sized(
                        egui::vec2(
                            width - button_width - 8.0,
                            ui.text_style_height(&egui::TextStyle::Heading),
                        ),
                        egui::TextEdit::singleline(&mut self.bpm_text_input)
                            .horizontal_align(egui::Align::Max)
                            .id(bpm_input_id)
                            .font(egui::TextStyle::Heading),
                    );
                    ui.add_sized(
                        egui::vec2(button_width, response.rect.height()),
                        egui::Label::new(
                            egui::RichText::new(tr("BPM")).text_style(egui::TextStyle::Heading),
                        ),
                    );

                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if response.lost_focus() || (response.changed() && enter_pressed) {
                        self.apply_bpm_input();
                    }
                });
                ui.add_space(12.0);
                // Shiftキーを押してるときはプロジェクトからBPMを持ってくる
                if ui.input(|i| i.modifiers.shift) {
                    let set_button =
                        egui::Button::new(tr("BPM取得")).min_size(egui::vec2(160.0, 48.0));
                    if ui
                        .add(set_button)
                        .on_hover_text(tr("プロジェクトからBPMを取得します"))
                        .clicked()
                    {
                        self.bpm = get_current_bpm_from_host();
                        self.will_reset_on_next_tap = true;
                    }
                } else {
                    let tap_label = if self.will_reset_on_next_tap {
                        tr("Reset")
                    } else if self.last_tap.is_some() {
                        tr("Tap")
                    } else {
                        tr("Start")
                    };
                    let tap_button = egui::Button::new(tap_label).min_size(egui::vec2(160.0, 48.0));
                    if ui
                        .add(tap_button)
                        .on_hover_text(tr("Spaceキーでもタップできます"))
                        .clicked()
                    {
                        self.register_tap();
                    }
                }
                ui.add_space(8.0);
                ui.columns(3, |columns| {
                    if columns[0]
                        .add_enabled(self.bpm.is_some(), egui::Button::new(tr("÷ 2")))
                        .clicked()
                    {
                        self.bpm = self.bpm.map(|bpm| bpm / 2.0);
                    }
                    if columns[1].button(tr("リセット")).clicked() {
                        self.reset_taps();
                    }
                    if columns[2]
                        .add_enabled(self.bpm.is_some(), egui::Button::new(tr("× 2")))
                        .clicked()
                    {
                        self.bpm = self.bpm.map(|bpm| bpm * 2.0);
                    }
                });
                ui.add_space(8.0);
                ui.columns_const(|[ui]| {
                    if ui
                        .add_enabled(
                            self.bpm.is_some(),
                            egui::Button::new(tr("0:00を基準に反映")),
                        )
                        .clicked()
                    {
                        self.apply_bpm_to_origin();
                    }
                    if ui
                        .add_enabled(
                            self.bpm.is_some(),
                            egui::Button::new(tr("現在のテンポに反映")),
                        )
                        .clicked()
                    {
                        self.apply_bpm_to_current();
                    }
                    if ui
                        .add_enabled(
                            self.bpm.is_some(),
                            egui::Button::new(tr("現在位置に新しく追加")),
                        )
                        .clicked()
                    {
                        self.add_bpm_at_current_position();
                    }
                });
            });
        });
    }

    fn render_info_window(&mut self, ctx: &egui::Context) {
        if !self.show_info {
            return;
        }
        let screen_rect = ctx.content_rect();
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
        let response = egui::Window::new(tr("Rusty Metronome Plugin"))
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let version_label = tr("バージョン: {version}");
                ui.label(version_label.replace("{version}", &self.version));
                ui.label(tr(
                    "BPMを合わせるタップボタンとメトロノームのエフェクトを提供します。",
                ));
                ui.add_space(8.0);
                ui.label(tr("開発者"));
                ui.hyperlink_to("Nanashi.", "https://sevenc7c.com");
                ui.add_space(4.0);
                ui.label(tr("ソースコード:"));
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

    fn apply_bpm_input(&mut self) {
        let trimmed = self.bpm_text_input.trim();
        if trimmed.is_empty() {
            self.bpm = None;
            self.last_tap = None;
            self.tap_intervals.clear();
            self.will_reset_on_next_tap = false;
            return;
        }
        match trimmed.parse::<f64>() {
            Ok(value) if value.is_finite() && value > 0.0 => {
                self.bpm = Some(value);
                self.last_tap = None;
                self.tap_intervals.clear();
                self.will_reset_on_next_tap = false;
            }
            _ => {
                self.bpm_text_input = self.bpm.map(|bpm| format!("{bpm:.2}")).unwrap_or_default();
            }
        }
    }

    fn apply_bpm_to_origin(&self) {
        if let Some(bpm) = self.bpm {
            let res = crate::EDIT_HANDLE.call_edit_section(|edit| {
                let mut bpm_infos = edit.get_bpm_info()?;
                bpm_infos[0].tempo = bpm as f32;
                edit.set_bpm_info(&bpm_infos)
            });
            tracing::info!("Applied BPM: {:?}", res);
        }
    }

    fn apply_bpm_to_current(&self) {
        if let Some(bpm) = self.bpm {
            let res = crate::EDIT_HANDLE.call_edit_section(|edit| {
                let current_frame = edit.info.frame;
                let fps = *edit.info.fps.numer() as f64 / *edit.info.fps.denom() as f64;
                let current_time = current_frame as f64 / fps;
                let mut bpm_infos = edit.get_bpm_info()?;
                let index = bpm_infos.partition_point(|bpm| bpm.offset <= current_time);
                if index == 0 {
                    bpm_infos[0].tempo = bpm as f32;
                } else {
                    bpm_infos[index - 1].tempo = bpm as f32;
                }
                edit.set_bpm_info(&bpm_infos)
            });
            tracing::info!("Applied BPM: {:?}", res);
        }
    }

    fn add_bpm_at_current_position(&self) {
        if let Some(bpm) = self.bpm {
            let res = crate::EDIT_HANDLE.call_edit_section(|edit| {
                let current_frame = edit.info.frame;
                let fps = *edit.info.fps.numer() as f64 / *edit.info.fps.denom() as f64;
                let current_time = current_frame as f64 / fps;
                let mut bpm_infos = edit.get_bpm_info()?;
                let new_bpm_info = aviutl2::generic::BpmInfo {
                    offset: current_time,
                    tempo: bpm as f32,
                    beat: 4,
                };
                if let Some(existing) = bpm_infos
                    .iter_mut()
                    .find(|bpm| (bpm.offset - current_time).abs() < 1e-6)
                {
                    *existing = new_bpm_info;
                } else {
                    bpm_infos.push(new_bpm_info);
                }
                bpm_infos.sort_by(|a, b| a.offset.total_cmp(&b.offset));
                edit.set_bpm_info(&bpm_infos)
            });
            tracing::info!("Added BPM: {:?}", res);
        }
    }
}

fn get_current_bpm_from_host() -> Option<f64> {
    let info = crate::EDIT_HANDLE.get_edit_info();
    let current_time = info.frame as f64 * *info.fps.denom() as f64 / *info.fps.numer() as f64;
    let mut bpm_info = crate::EDIT_HANDLE
        .call_read_section(|read| read.get_bpm_info())
        .ok()?
        .ok()?;
    bpm_info.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    let index = bpm_info.partition_point(|bpm| bpm.offset <= current_time);
    if index == 0 {
        None
    } else {
        Some(bpm_info[index - 1].tempo as f64)
    }
}
