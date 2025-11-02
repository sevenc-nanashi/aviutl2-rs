use aviutl2::AnyResult;

#[easy_ext::ext]
impl srtlib::Timestamp {
    fn to_milliseconds(&self) -> u32 {
        let (h, m, s, ms) = self.get();
        srtlib::Timestamp::convert_to_milliseconds(h, m, s, ms)
    }
}

#[aviutl2::plugin(GenericPlugin)]
struct SrtImportPlugin {}

impl aviutl2::generic::GenericPlugin for SrtImportPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(SrtImportPlugin {})
    }

    fn register(&self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_menus::<SrtImportPlugin>();
    }
}

#[aviutl2::generic::menus]
impl SrtImportPlugin {
    #[import(name = "SRTファイル（*.srt）")]
    fn import_menu(&mut self, edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        let current_object = edit_section.get_focused_object()?;
        let Some(obj) = current_object else {
            anyhow::bail!("オブジェクトが選択されていません。");
        };
        let obj = edit_section.object(&obj);
        if obj.get_effect_item("テキスト", 0, "テキスト").is_err() {
            anyhow::bail!("選択されたオブジェクトはテキストオブジェクトではありません。");
        }

        let file_path = native_dialog::FileDialogBuilder::default()
            .add_filter("SRTファイル", ["srt"])
            .set_title("SRTファイルを選択")
            .open_single_file()
            .show()?;
        let Some(file_path) = file_path else {
            return Ok(());
        };

        let srt = srtlib::Subtitles::parse_from_file(&file_path, None)
            .map_err(|e| anyhow::anyhow!("SRTファイルの解析に失敗しました: {}", e))?;

        let aviutl2::generic::ObjectLayerFrame {
            layer,
            start: existing_start_frame,
            end: existing_end_frame,
        } = obj.get_layer_frame()?;
        let layer = edit_section.layer(layer);
        let fps = edit_section.info.fps;
        let fps = *fps.numer() as f64 / *fps.denom() as f64;

        let mut subtitles = srt.to_vec();
        subtitles.sort_by_key(|s| (s.start_time, s.end_time));
        let Some(last_subtitle) = subtitles.last() else {
            anyhow::bail!("SRTファイルに字幕が含まれていません。");
        };
        let last_subtitle_ms = last_subtitle.end_time.to_milliseconds();
        let total_frames = (last_subtitle_ms as f64 / 1000.0 * fps).ceil() as u32;
        let next_object = layer.find_object_after(existing_end_frame + 1)?;
        let existing_next_frame = if let Some(next_object) = next_object.as_ref() {
            let next_obj = edit_section.object(next_object);
            let next_layer_frame = next_obj.get_layer_frame()?;
            next_layer_frame.start
        } else {
            usize::MAX
        };
        if existing_start_frame + total_frames as usize > existing_next_frame {
            edit_section.focus_object(obj.handle)?;
            anyhow::bail!("字幕を追加すると既存のオブジェクトと重なってしまいます。");
        }

        let alias = obj.get_alias()?;
        let mut alias = alias.lines().collect::<Vec<_>>();
        if alias.len() < 2 || !alias.remove(1).starts_with("frame=") {
            anyhow::bail!("オブジェクトの編集に失敗しました。");
        }
        let alias = alias.join("\n");
        obj.delete_object()?;
        let mut next_frame = existing_start_frame;
        for subtitle in subtitles {
            let start_ms = subtitle.start_time.to_milliseconds();
            let end_ms = subtitle.end_time.to_milliseconds();
            let mut start_frame =
                existing_start_frame + (start_ms as f64 / 1000.0 * fps).round() as usize;
            let end_frame = existing_start_frame + (end_ms as f64 / 1000.0 * fps).round() as usize;
            if start_frame >= end_frame {
                continue;
            }
            if start_frame < next_frame {
                start_frame = next_frame;
            }
            edit_section.output_log(&format!(
                "Adding subtitle: {} --> {} (frames {} to {})",
                subtitle.start_time, subtitle.end_time, start_frame, end_frame
            ))?;
            let new_obj = edit_section.create_object_from_alias(
                &alias,
                layer.index,
                start_frame,
                end_frame - start_frame + 1,
            )?;
            let new_obj = edit_section.object(&new_obj);
            new_obj.set_effect_item("テキスト", 0, "テキスト", &subtitle.text)?;
            next_frame = end_frame + 1;
        }

        Ok(())
    }

    #[export(name = "SRTを書き出し（*.srt）")]
    fn export_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        for (i, layer) in edit_section.layers().enumerate() {
            edit_section.output_log(&format!("Layer {}:", i))?;
            for (j, obj) in layer.objects() {
                let alias = edit_section.object(&obj).get_alias()?;
                edit_section.output_log(&format!("  Object {:?}: {}", j, alias))?;
            }
        }
        Ok(())
    }
}

aviutl2::register_generic_plugin!(SrtImportPlugin);
