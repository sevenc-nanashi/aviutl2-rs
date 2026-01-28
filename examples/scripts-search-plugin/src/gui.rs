use aviutl2::anyhow;
use aviutl2_eframe::{AviUtl2EframeHandle, eframe, egui};
use itertools::Itertools;

pub(crate) struct ScriptsSearchApp {
    show_info: bool,
    suppress_info_close_once: bool,
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
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            show_info: false,
            suppress_info_close_once: false,
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
                    let resp = ui
                        .add_sized(
                            egui::vec2(
                                ui.text_style_height(&egui::TextStyle::Heading),
                                ui.text_style_height(&egui::TextStyle::Heading),
                            ),
                            egui::Button::image(include_iconify!("mdi:information-outline")),
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
        egui::CentralPanel::default().show(ctx, |ui| match crate::EFFECTS.get() {
            None => {
                ui.label("エフェクト情報を読み込み中...");
            }
            Some(effects) => {
                ui.label(format!("登録されているエフェクト数: {}", effects.len()));
                ui.add_space(8.0);
                let search_response = egui::TextEdit::singleline(&mut self.needle)
                    .desired_width(f32::INFINITY)
                    .hint_text("検索...")
                    .show(ui);
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    search_response.response.request_focus();
                }
                ui.add_space(8.0);
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
        let needle_raw_str = self.needle.trim();
        let needle_normalized_str = Self::normalize_kana_for_search(needle_raw_str);
        let needle_raw = nucleo_matcher::Utf32String::from(needle_raw_str);
        let needle_normalized = if needle_normalized_str == needle_raw_str {
            None
        } else {
            Some(nucleo_matcher::Utf32String::from(needle_normalized_str))
        };
        let mut sorted_effects = effects
            .iter()
            .filter_map(|effect| {
                let mut indices_raw = vec![];
                let score_raw = self.matcher.fuzzy_indices(
                    effect.u32_label.slice(..),
                    needle_raw.slice(..),
                    &mut indices_raw,
                );
                let Some(needle_normalized) = needle_normalized.as_ref() else {
                    return score_raw.map(|score| (score, effect, indices_raw));
                };
                let mut indices_normalized = vec![];
                let score_normalized = self.matcher.fuzzy_indices(
                    effect.u32_label.slice(..),
                    needle_normalized.slice(..),
                    &mut indices_normalized,
                );
                match (score_raw, score_normalized) {
                    (Some(raw), Some(normalized)) if normalized > raw => {
                        Some((normalized, effect, indices_normalized))
                    }
                    (Some(raw), _) => Some((raw, effect, indices_raw)),
                    (None, Some(normalized)) => Some((normalized, effect, indices_normalized)),
                    (None, None) => None,
                }
            })
            .collect::<Vec<_>>();
        if sorted_effects.is_empty() {
            ui.label("一致するエフェクトが見つかりませんでした。");
        } else {
            sorted_effects.sort_by(|a, b| b.0.cmp(&a.0));
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
        let response = egui::Window::new("Rusty Scripts Search Plugin")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
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

    fn render_effect_card(
        &self,
        ui: &mut egui::Ui,
        effect: &crate::EffectData,
        match_indices: &[u32],
    ) {
        let frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .inner_margin(egui::Margin::symmetric(8, 4));
        let available_width = ui.available_width();
        let response = ui.allocate_ui_with_layout(
            egui::vec2(available_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                frame
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_min_height(24.0);
                            let (name, icon) = Self::effect_type_display(effect);
                            ui.add(
                                egui::Image::new(icon)
                                    .max_size(egui::vec2(24.0, 24.0))
                                    .tint(ui.visuals().text_color()),
                            )
                            .on_hover_text(name);

                            let colored_label =
                                Self::build_highlighted_label(ui, effect, match_indices);
                            ui.add(egui::Label::new(colored_label).selectable(false));
                        });
                    })
                    .response
            },
        );
        let response = response
            .response
            .interact(egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        // フィルターエフェクトの場合、ホバー時にオーバーレイを表示
        if effect.effect.effect_type == aviutl2::generic::EffectType::Filter {
            if !self.show_info {
                let clip_rect = ui.clip_rect();
                let hovered = ui.ctx().pointer_hover_pos().is_some_and(|pos| {
                    Self::is_filter_actions_hovered(ui.ctx(), response.rect, clip_rect, effect, pos)
                });
                if hovered || response.hovered() {
                    Self::render_filter_actions_overlay(
                        ui.ctx(),
                        response.id,
                        response.rect,
                        clip_rect,
                        effect,
                    );
                }
            }
            if response.clicked() {
                let modifiers = response.ctx.input(|i| i.modifiers);
                let res = if modifiers.alt && effect.effect.flag.as_filter {
                    Self::add_filter_as_filter_object(effect)
                } else if modifiers.shift {
                    Self::add_filter_to_focused_object(effect)
                } else {
                    Self::add_filter_as_object(effect)
                };
                log::debug!("Filter card clicked: {:?}", res);
            }
        } else {
            Self::handle_non_filter_click(effect, &response);
        }
    }

    fn build_highlighted_label(
        ui: &egui::Ui,
        effect: &crate::EffectData,
        match_indices: &[u32],
    ) -> egui::text::LayoutJob {
        let mut colored_label = egui::text::LayoutJob::default();
        let chunks = effect
            .label
            .chars()
            .enumerate()
            .chunk_by(|(i, _)| match_indices.contains(&(*i as u32)));

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

    fn normalize_kana_for_search(input: &str) -> String {
        if input.is_empty() {
            return String::new();
        }
        input
            .chars()
            .map(|c| {
                if ('\u{3041}'..='\u{3096}').contains(&c) {
                    let code = u32::from(c) + 0x60;
                    char::from_u32(code).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }

    fn render_filter_actions_overlay(
        ctx: &egui::Context,
        id: egui::Id,
        rect: egui::Rect,
        clip_rect: egui::Rect,
        effect: &crate::EffectData,
    ) {
        let button_size = egui::vec2(20.0, 20.0);
        let button_margin = egui::vec2(12.0, 4.0);
        let gap = 4.0;
        let button_count = if effect.effect.flag.as_filter { 3 } else { 2 };
        let total_width = button_size.x * button_count as f32 + gap * (button_count - 1) as f32;
        let inner_rect = rect.shrink2(button_margin);
        let top_left = egui::pos2(
            inner_rect.right() - total_width,
            inner_rect.top() + inner_rect.height() / 2.0 - button_size.y / 2.0,
        );
        let actions_rect =
            egui::Rect::from_min_size(top_left, egui::vec2(total_width, button_size.y));
        if !clip_rect.contains(actions_rect.min) || !clip_rect.contains(actions_rect.max) {
            return;
        }
        let actions_id = id.with("filter_actions_overlay");

        egui::Area::new(actions_id)
            .order(egui::Order::Middle)
            .fixed_pos(top_left)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                ui.set_min_size(egui::vec2(total_width, button_size.y));
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().button_padding = egui::vec2(2.0, 2.0);
                    let mut action_button = |icon: egui::ImageSource<'static>,
                                             tooltip: &str,
                                             action: fn(
                        &crate::EffectData,
                    )
                        -> anyhow::Result<()>| {
                        let response = ui
                            .add_sized(button_size, egui::Button::image(icon))
                            .on_hover_text(tooltip)
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if response.clicked() {
                            let res = action(effect);
                            log::debug!("Filter action {}: {:?}", tooltip, res);
                        }
                    };

                    if effect.effect.flag.as_filter {
                        action_button(
                            include_iconify!("mdi:card-multiple"),
                            "フィルタオブジェクトとして追加",
                            Self::add_filter_as_filter_object,
                        );
                    }
                    action_button(
                        include_iconify!("material-symbols:add-row-below"),
                        "選択中のオブジェクトに追加",
                        Self::add_filter_to_focused_object,
                    );
                    action_button(
                        include_iconify!("material-symbols:view-timeline"),
                        "オブジェクトとして追加",
                        Self::add_filter_as_object,
                    );
                });
            });
    }

    fn is_filter_actions_hovered(
        _ctx: &egui::Context,
        rect: egui::Rect,
        clip_rect: egui::Rect,
        effect: &crate::EffectData,
        pos: egui::Pos2,
    ) -> bool {
        if rect.contains(pos) {
            return true;
        }
        let button_size = egui::vec2(16.0, 16.0);
        let button_padding = egui::vec2(14.0, 4.0);
        let gap = 4.0;
        let button_count = if effect.effect.flag.as_filter { 3 } else { 2 };
        let total_width = button_size.x * button_count as f32 + gap * (button_count - 1) as f32;
        let inner_rect = rect.shrink2(egui::vec2(8.0, 4.0));
        let top_left = egui::pos2(
            inner_rect.right() - total_width - button_padding.x,
            inner_rect.top() + button_padding.y,
        );
        let actions_rect =
            egui::Rect::from_min_size(top_left, egui::vec2(total_width, button_size.y));
        clip_rect.contains(actions_rect.min)
            && clip_rect.contains(actions_rect.max)
            && actions_rect.contains(pos)
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
                } => (
                    "入力（映像）",
                    include_iconify!("material-symbols:animated-images"),
                ),
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
