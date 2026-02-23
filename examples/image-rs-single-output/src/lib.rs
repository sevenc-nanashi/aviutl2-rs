use anyhow::Context;
use aviutl2::{log, output::OutputPlugin};

#[aviutl2::plugin(OutputPlugin)]
struct ImageRsOutputPlugin;

impl OutputPlugin for ImageRsOutputPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        aviutl2::logger::LogBuilder::new()
            .filter_level(if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .init();
        Ok(ImageRsOutputPlugin)
    }

    fn plugin_info(&self) -> aviutl2::output::OutputPluginTable {
        aviutl2::output::OutputPluginTable {
            name: "Rusty Single Image Output".to_string(),
            output_type: aviutl2::output::OutputType::Image,
            file_filters: aviutl2::file_filters! {
                "WebP Image" => ["webp"],
                "PNG Image" => ["png"],
                "JPEG Image" => ["jpg", "jpeg"],
                "All Image Formats" => [],
            },

            information: format!(
                "image-rs Single Output for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-single-output",
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

        let image: aviutl2::output::video_frame::Hf64VideoFrame = info
            .get_video_frame(0)
            .context("Failed to get video frame")?;
        let output = image::DynamicImage::from(image::ImageBuffer::<image::Rgba<f32>, _>::from_fn(
            video_info.width,
            video_info.height,
            |x, y| {
                let idx = ((y) * video_info.width + (x)) as usize;
                image::Rgba([
                    image.data[idx].0.into(),
                    image.data[idx].1.into(),
                    image.data[idx].2.into(),
                    image.data[idx].3.into(),
                ])
            },
        ));
        output.save(path)?;

        Ok(())
    }
}

aviutl2::register_output_plugin!(ImageRsOutputPlugin);
