use anyhow::Context;
use aviutl2::{
    output::{OutputPlugin, Pa64VideoFrame},
    register_output_plugin,
};

struct ImageRsOutputPlugin;

impl ImageRsOutputPlugin {
    fn write(
        &self,
        info: &aviutl2::output::OutputInfo,
        path: &std::path::Path,
        frame: &Pa64VideoFrame,
    ) -> anyhow::Result<()> {
        let video_info = info.video.as_ref().context("Video format not available")?;
        let mut rgba_data = Vec::with_capacity(frame.data.len() * 4);
        for &pixel in &frame.data {
            rgba_data.push((pixel.0 >> 8) as u8); // R
            rgba_data.push((pixel.1 >> 8) as u8); // G
            rgba_data.push((pixel.2 >> 8) as u8); // B
            rgba_data.push((pixel.3 >> 8) as u8); // A
        }

        let image = image::RgbaImage::from_raw(video_info.width, video_info.height, rgba_data)
            .context("Failed to create image from raw data")?;
        image
            .save(path)
            .with_context(|| format!("Failed to save image to {}", path.display()))?;

        Ok(())
    }
}

impl OutputPlugin for ImageRsOutputPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        Ok(ImageRsOutputPlugin)
    }

    fn plugin_info(&self) -> aviutl2::output::OutputPluginTable {
        aviutl2::output::OutputPluginTable {
            name: "Rusty Image Output".to_string(),
            output_type: aviutl2::output::OutputType::Video,
            file_filters: aviutl2::file_filters! {
                "WebP Image" => ["webp"],
                "PNG Image" => ["png"],
                "JPEG Image" => ["jpg", "jpeg"],
                "All Image Formats" => [],
            },

            information: format!(
                "image-rs Output for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-output",
                version = env!("CARGO_PKG_VERSION")
            ),
            can_config: false,
        }
    }

    fn output(&self, info: aviutl2::output::OutputInfo) -> aviutl2::AnyResult<()> {
        let Some(video_info) = &info.video else {
            anyhow::bail!("動画情報がありません。");
        };
        let path = info.path.clone();
        let pattern = lazy_regex::regex!(r"#+");
        let filename = path
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?
            .to_string_lossy();
        let replaces = pattern.find_iter(&filename).collect::<Vec<_>>();
        if replaces.is_empty() {
            anyhow::bail!(
                "ファイル名には連続する「`#`」を含めてください。その部分が連番になります。"
            );
        }
        if replaces.len() > 1 {
            anyhow::bail!("ファイル名には連続する「`#`」を1箇所だけ含めてください。");
        }
        let required_len = (video_info.num_frames - 1).to_string().len();
        if replaces[0].as_str().len() < required_len {
            anyhow::bail!("連続する「`#`」の数が足りません。最低でも{required_len}つ必要です。");
        }

        for (i, frame) in info.get_video_frames_iter() {
            let frame_str = format!("{:0width$}", i, width = replaces[0].as_str().len());
            let new_filename = pattern.replace(&filename, frame_str.as_str()).to_string()
                + "."
                + info
                    .path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("webp");
            let new_path = path.with_file_name(new_filename);
            self.write(&info, &new_path, &frame).with_context(|| {
                format!(
                    "{}フレーム目を{}に保存できませんでした。",
                    i,
                    new_path.display()
                )
            })?;
        }
        Ok(())
    }
}

register_output_plugin!(ImageRsOutputPlugin);
