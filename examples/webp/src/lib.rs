use aviutl2::input::AnyResult;
use aviutl2::input::InputFilter;
use aviutl2::input::InputPlugin;
use aviutl2::input::IntoImage;
use aviutl2::register_input_plugin;

struct ImageRsPlugin {}

impl InputPlugin for ImageRsPlugin {
    type InputHandle = image::RgbaImage;

    fn new() -> Self {
        ImageRsPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "Rusty WebP".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: vec![InputFilter {
                name: "Image Files".to_string(),
                extensions: vec!["webp".to_string()],
            }],
            information: "WebP for AviUtl, powered by Rust / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/webp".to_owned(),
            can_config: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> Option<Self::InputHandle> {
        match image::open(file) {
            Ok(img) => Some(img.to_rgba8()),
            Err(e) => {
                eprintln!("Failed to open image: {}", e);
                None
            }
        }
    }

    fn get_input_info(&self, handle: &Self::InputHandle) -> AnyResult<aviutl2::input::InputInfo> {
        let width = handle.width() as u32;
        let height = handle.height() as u32;
        let format = aviutl2::input::ImageFormat { width, height };

        Ok(aviutl2::input::InputInfo {
            video: Some(aviutl2::input::VideoInputInfo {
                fps: 30,
                scale: 1,
                num_frames: 1,
                image_format: format,
            }),
            audio: None, // No audio for image files
        })
    }

    fn read_video(
        &self,
        handle: &Self::InputHandle,
        frame: i32,
    ) -> AnyResult<aviutl2::input::ImageBuffer> {
        anyhow::ensure!(frame == 0, "Only frame 0 is valid for image input");
        let buffer = handle
            .pixels()
            .map(|p| (p[0], p[1], p[2], p[3]))
            .collect::<Vec<_>>();
        buffer.into_image()
    }

    fn close(&self, handle: Self::InputHandle) -> bool {
        drop(handle);
        true
    }
}

register_input_plugin!(ImageRsPlugin);
