use aviutl2::anyhow;
use aviutl2_eframe::{AviUtl2EframeHandle, eframe, egui};
use itertools::Itertools;

pub(crate) struct ScriptsSearchApp {
    show_info: bool,
    version: String,
    handle: AviUtl2EframeHandle,

    matcher: nucleo_matcher::Matcher,
    needle: String,
}

macro_rules! include_iconify {
    ($icon:expr) => {
        egui::ImageSource::Bytes {
            uri: (concat!("iconify://", $icon, ".svg")).into(),
            bytes: egui::load::Bytes::Static(iconify::svg!($icon, color = "white").as_bytes()),
        }
    };
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
        let mut config = nucleo_matcher::Config::DEFAULT.clone();
        config.ignore_case = true;
        config.prefer_prefix = true;
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            show_info: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
            handle,
            matcher: nucleo_matcher::Matcher::new(config),
            needle: String::new(),
        }
    }
}

impl eframe::App for ScriptsSearchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_toolbar(ctx);
        self.render_main_panel(ctx);
        self.render_info_window(ctx);
    }
}

impl ScriptsSearchApp {
    fn render_toolbar(&mut self, ctx: &egui::Context) {
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
    }

    fn render_main_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| match crate::EFFECTS.get() {
            None => {
                ui.label("エフェクト情報を読み込み中...");
            }
            Some(effects) => {
                ui.label(format!("登録されているエフェクト数: {}", effects.len()));
                ui.add_space(8.0);
                egui::TextEdit::singleline(&mut self.needle)
                    .desired_width(f32::INFINITY)
                    .hint_text("検索...")
                    .show(ui);
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_effects_list(ui, effects);
                    });
            }
        });
    }

    fn render_effects_list(&mut self, ui: &mut egui::Ui, effects: &[crate::EffectData]) {
        if self.needle.is_empty() {
            self.render_all_effects(ui, effects);
        } else {
            self.render_filtered_effects(ui, effects);
        }
    }

    fn render_all_effects(&self, ui: &mut egui::Ui, effects: &[crate::EffectData]) {
        for effect in effects.iter() {
            ui.add_space(4.0);
            self.render_effect_card(ui, effect, &[]);
        }
    }

    fn render_filtered_effects(&mut self, ui: &mut egui::Ui, effects: &[crate::EffectData]) {
        let needle = self.needle.trim();
        let needle = nucleo_matcher::Utf32String::from(needle);
        let mut sorted_effects = effects
            .iter()
            .filter_map(|effect| {
                let mut indices = vec![];
                let score = self.matcher.fuzzy_indices(
                    effect.u32_label.slice(..),
                    needle.slice(..),
                    &mut indices,
                );
                score.map(|score| (score, effect, indices))
            })
            .collect::<Vec<_>>();
        if sorted_effects.is_empty() {
            ui.add_space(4.0);
            ui.label("一致するエフェクトが見つかりませんでした。");
        } else {
            sorted_effects.sort_by(|a, b| b.0.cmp(&a.0));
            ui.add_space(4.0);
            ui.label(format!(
                "エフェクト数: {}",
                if sorted_effects.len() > 100 {
                    "100+".to_string()
                } else {
                    sorted_effects.len().to_string()
                }
            ));
            for (_score, effect, indices) in sorted_effects.iter().take(100) {
                ui.add_space(4.0);
                self.render_effect_card(ui, effect, indices);
            }
        }
    }

    fn render_info_window(&mut self, ctx: &egui::Context) {
        if !self.show_info {
            return;
        }
        let mut open = true;
        egui::Window::new("Rusty Scripts Search Plugin")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(format!("バージョン: {}", self.version));
                ui.label("オブジェクト・エフェクトを検索するプラグイン。");
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
                    "examples/scripts-search-plugin",
                    format!(
                        "https://github.com/sevenc-nanashi/aviutl2-rs/tree/{}/examples/scripts-search-plugin",
                        self.version
                    ),
                );
            });
        if !open {
            self.show_info = false;
        }
    }

    fn render_effect_card(
        &self,
        ui: &mut egui::Ui,
        effect: &crate::EffectData,
        match_indicies: &[u32],
    ) {
        let frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke);
        let available_width = ui.available_width();
        let response = ui.allocate_ui_with_layout(
            egui::vec2(available_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                frame
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let (name, icon) = Self::effect_type_display(effect);
                            ui.add(
                                egui::Image::new(icon)
                                    .max_size(egui::vec2(32.0, 32.0))
                                    .tint(ui.visuals().text_color()),
                            )
                            .on_hover_text(name);

                            let colored_label =
                                Self::build_highlighted_label(ui, effect, match_indicies);
                            ui.label(colored_label);
                        });
                    })
                    .response
            },
        );
        let response = response
            .inner
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if effect.effect.effect_type == aviutl2::generic::EffectType::Filter {
            Self::render_filter_popup(ui, effect, &response);
        } else {
            // シーンチェンジと入力は直接追加する
            Self::handle_non_filter_click(effect, &response);
        }
    }

    fn build_highlighted_label(
        ui: &egui::Ui,
        effect: &crate::EffectData,
        match_indicies: &[u32],
    ) -> egui::text::LayoutJob {
        let mut colored_label = egui::text::LayoutJob::default();
        let chunks = effect
            .label
            .chars()
            .enumerate()
            .chunk_by(|(i, _)| match_indicies.contains(&(*i as u32)));

        let chunks = chunks
            .into_iter()
            .map(|(is_matched, chunk)| {
                let (_, s): (Vec<_>, Vec<char>) = chunk.unzip();
                (is_matched, s.into_iter().collect::<String>())
            })
            .collect::<Vec<_>>();
        for (is_matched, chunk) in chunks {
            colored_label.append(
                &chunk,
                0.0,
                egui::TextFormat {
                    color: if is_matched {
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().text_color()
                    },
                    ..Default::default()
                },
            );
        }
        colored_label
    }

    fn render_filter_popup(
        _ui: &mut egui::Ui,
        effect: &crate::EffectData,
        response: &egui::Response,
    ) {
        egui::containers::Popup::menu(response)
            .width(f32::INFINITY)
            .show(|ui| {
                if ui.button("現在のオブジェクトに追加").clicked() {
                    let res = Self::add_filter_to_focused_object(effect);
                    log::debug!("Effect added to focused object: {:?}", res);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("フィルタ効果として追加").clicked() {
                    let res = Self::add_filter_as_object(effect);
                    log::debug!("Effect added as scene change: {:?}", res);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if effect.effect.flag.as_filter
                    && ui.button("フィルタオブジェクトとして追加").clicked()
                {
                    let res = Self::add_filter_as_filter_object(effect);
                    log::debug!("Effect added as scene change: {:?}", res);
                    ui.close_kind(egui::UiKind::Menu);
                }
            });
    }

    fn handle_non_filter_click(effect: &crate::EffectData, response: &egui::Response) {
        if response.clicked() {
            let res = crate::EDIT_HANDLE.get().unwrap().call_edit_section(|e| {
                let created =
                    e.create_object(&effect.effect.name, e.info.layer, e.info.frame, None)?;
                e.focus_object(&created)?;
                anyhow::Ok(())
            });
            log::debug!("Effect added: {:?}", res);
        }
    }

    fn add_filter_to_focused_object(effect: &crate::EffectData) -> anyhow::Result<()> {
        crate::EDIT_HANDLE
            .get()
            .unwrap()
            .call_edit_section(|edit| {
                // フィルターを追加するAPIがないため、エイリアスを編集して対応する
                let focused_object = edit
                    .get_focused_object()?
                    .ok_or_else(|| anyhow::anyhow!("オブジェクトが選択されていません。"))?;
                let alias_str = edit.object(&focused_object).get_alias()?;
                let mut alias: aviutl2::alias::Table = alias_str
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Failed to parse alias: {}", e))?;
                let alias_table = alias
                    .get_table_mut("Object")
                    .ok_or_else(|| anyhow::anyhow!("Failed to get Object table from alias"))?;
                let last_table = alias_table.subtables().last().ok_or_else(|| {
                    anyhow::anyhow!("Failed to get last subtable from Object table")
                })?;
                let effect_index =
                    last_table.0.parse::<u32>().map_err(|e| {
                        anyhow::anyhow!("Failed to parse last subtable index: {}", e)
                    })? + 1;
                alias_table.insert_table(&effect_index.to_string(), {
                    let mut table = aviutl2::alias::Table::new();
                    table.insert_value("effect.name", &effect.effect.name);
                    table
                });
                let base_position = edit.object(&focused_object).get_layer_frame()?;
                edit.object(&focused_object).delete_object()?;
                match edit.create_object_from_alias(
                    &alias.to_string(),
                    base_position.layer,
                    base_position.start,
                    0,
                ) {
                    Ok(created) => {
                        edit.focus_object(&created)?;
                        anyhow::Ok(())
                    }
                    Err(err) => {
                        edit.create_object_from_alias(
                            &alias_str,
                            base_position.layer,
                            base_position.start,
                            0,
                        )?;
                        Err(err.into())
                    }
                }
            })?
    }

    fn add_filter_as_object(effect: &crate::EffectData) -> anyhow::Result<()> {
        crate::EDIT_HANDLE.get().unwrap().call_edit_section(|e| {
            e.create_object(&effect.effect.name, e.info.layer, e.info.frame, None)?;
            anyhow::Ok(())
        })?
    }

    fn add_filter_as_filter_object(effect: &crate::EffectData) -> anyhow::Result<()> {
        crate::EDIT_HANDLE.get().unwrap().call_edit_section(|e| {
            let filter =
                e.create_object("フィルタオブジェクト", e.info.layer, e.info.frame, None)?;
            let mut filter_alias = e.object(&filter).get_alias_parsed()?;
            e.object(&filter).delete_object()?;
            filter_alias
                .get_table_mut("Object")
                .expect("Failed to get Object table")
                .insert_table("1", {
                    let mut table = aviutl2::alias::Table::new();
                    table.insert_value("effect.name", &effect.effect.name);
                    table
                });
            let created = e.create_object_from_alias(
                &filter_alias.to_string(),
                e.info.layer,
                e.info.frame,
                0,
            )?;
            e.focus_object(&created)?;

            anyhow::Ok(())
        })?
    }

    fn effect_type_display(
        effect: &crate::EffectData,
    ) -> (&'static str, egui::ImageSource<'static>) {
        match effect.effect.effect_type {
            aviutl2::generic::EffectType::Input => match effect.effect.flag {
                aviutl2::generic::EffectFlag {
                    video: true,
                    audio: true,
                    ..
                } => (
                    "入力（映像・音声）",
                    include_iconify!("material-symbols:movie"),
                ),
                aviutl2::generic::EffectFlag {
                    video: true,
                    audio: false,
                    ..
                } => ("入力（映像）", include_iconify!("material-symbols:image")),
                aviutl2::generic::EffectFlag {
                    video: false,
                    audio: true,
                    ..
                } => (
                    "入力（音声）",
                    include_iconify!("material-symbols:audio-file"),
                ),
                _ => ("入力", include_iconify!("mdi:file")),
            },
            aviutl2::generic::EffectType::Filter => match effect.effect.flag {
                aviutl2::generic::EffectFlag {
                    video: true,
                    audio: true,
                    ..
                } => (
                    "フィルタ（映像・音声）",
                    include_iconify!("material-symbols:sliders"),
                ),
                aviutl2::generic::EffectFlag {
                    video: true,
                    audio: false,
                    ..
                } => ("フィルタ（映像）", include_iconify!("mdi:paint")),
                aviutl2::generic::EffectFlag {
                    video: false,
                    audio: true,
                    ..
                } => (
                    "フィルタ（音声）",
                    include_iconify!("material-symbols:equalizer"),
                ),
                _ => ("フィルタ", include_iconify!("mdi:tune-vertical")),
            },
            aviutl2::generic::EffectType::SceneChange => (
                "シーンチェンジ",
                include_iconify!("material-symbols:transition-chop"),
            ),
            _ => ("その他", include_iconify!("mdi:puzzle-outline")),
        }
    }
}
