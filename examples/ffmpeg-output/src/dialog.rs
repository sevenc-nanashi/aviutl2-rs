use crate::{DEFAULT_ARGS, REQUIRED_ARGS, config::FfmpegOutputConfig};
use dedent::dedent;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

pub struct FfmpegOutputConfigDialog {
    pub args_buffer: String,
    pub pixel_format: crate::config::PixelFormat,
    pub result_sender: std::sync::mpsc::Sender<FfmpegOutputConfig>,
}

fn buffer_to_args(buffer: &str) -> Vec<String> {
    buffer
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

impl FfmpegOutputConfigDialog {
    pub fn new(
        config: FfmpegOutputConfig,
        sender: std::sync::mpsc::Sender<FfmpegOutputConfig>,
    ) -> Self {
        Self {
            args_buffer: config.args.join("\n"),
            pixel_format: config.pixel_format,
            result_sender: sender,
        }
    }
}

impl eframe::App for FfmpegOutputConfigDialog {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("main-grid")
                .min_col_width(ui.available_width() / 2.0)
                .min_row_height(ui.available_height())
                .num_columns(2)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(f32::INFINITY)
                        .id_salt("l")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                let mut cache = CommonMarkCache::default();
                                CommonMarkViewer::new().show(
                                    ui,
                                    &mut cache,
                                    &format!(
                                        dedent!(
                                            r#"
                                            # Rusty FFmpeg Output Plugin v{}
                                            FFmpegを使用して動画と音声を出力するプラグインです。\
                                            FFmpegに渡す引数を設定できます。\
                                            引数は行区切りで入力してください。\
                                            以下の引数は実行時に置換されます：
                                            - `{{video_source}}`：動画の入力ソース
                                            - `{{video_pixel_format}}`：動画のピクセルフォーマット
                                            - `{{video_size}}`：動画の解像度
                                            - `{{video_fps}}`：動画のフレームレート
                                            - `{{audio_source}}`：音声の入力ソース
                                            - `{{audio_sample_rate}}`：音声のサンプルレート
                                            - `{{maybe_vflip}}`：Bgr24でのみ`vflip`、それ以外では`null`
                                            - `{{output_path}}`：出力ファイルのパス

                                            上の引数はすべて含まれている必要があります。
                                            FFmpegについて詳しくない場合は、この設定を手動で変更せず、\
                                            プリセットを使用することをお勧めします。
                                            "#
                                        ),
                                        env!("CARGO_PKG_VERSION")
                                    )
                                );

                                ui.collapsing("プリセット", |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        for preset in crate::presets::PRESETS {
                                            if ui
                                                .button(preset.name)
                                                .on_hover_text(preset.description)
                                                .clicked()
                                            {
                                                self.args_buffer = preset.args.join("\n");
                                                self.pixel_format = preset.pixel_format;
                                            }
                                        }
                                    });
                                });

                                ui.horizontal(|ui| {
                                    ui.label("ピクセルフォーマット:");
                                    egui::ComboBox::from_id_salt("pixel_format")
                                        .selected_text(self.pixel_format.as_str())
                                        .show_ui(ui, |ui| {
                                            for format in [
                                                crate::config::PixelFormat::Yuy2,
                                                crate::config::PixelFormat::Bgr24,
                                                crate::config::PixelFormat::Pa64,
                                                crate::config::PixelFormat::Hf64,
                                            ] {
                                                ui.selectable_value(
                                                    &mut self.pixel_format,
                                                    format,
                                                    format.as_str(),
                                                );
                                            }
                                        });
                                });

                                ui.horizontal(|ui| {
                                    let args = buffer_to_args(&self.args_buffer);
                                    let can_save = REQUIRED_ARGS
                                        .iter()
                                        .all(|arg| args.iter().any(|a| a.contains(arg)));
                                    if ui
                                        .add_enabled(can_save, egui::Button::new("保存"))
                                        .clicked()
                                    {
                                        self.result_sender
                                            .send(FfmpegOutputConfig {
                                                args,
                                                pixel_format: self.pixel_format,
                                            })
                                            .expect("Failed to send args");
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                    if ui.button("リセット").clicked() {
                                        self.pixel_format =
                                            FfmpegOutputConfig::default().pixel_format;
                                        self.args_buffer = DEFAULT_ARGS.join("\n");
                                    }
                                    if ui.button("キャンセル").clicked() {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                });
                            });
                        });

                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .max_height(f32::INFINITY)
                        .id_salt("r")
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.args_buffer)
                                    .desired_width(f32::INFINITY)
                                    .min_size(egui::vec2(
                                        ui.available_width(),
                                        ui.available_height(),
                                    ))
                                    .font(egui::TextStyle::Monospace),
                            );
                        })
                });
        });
    }
}
