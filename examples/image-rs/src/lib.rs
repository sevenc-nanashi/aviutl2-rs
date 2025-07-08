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

    fn info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "ImageRs Plugin".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: vec![InputFilter {
                name: "Image Files".to_string(),
                extensions: vec![
                    "png".to_string(),
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "bmp".to_string(),
                ],
            }],
            information: "A plugin to handle image files using the image crate.".to_string(),
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

    fn get_info(&self, handle: &Self::InputHandle) -> AnyResult<aviutl2::input::InputInfo> {
        let width = handle.width() as u32;
        let height = handle.height() as u32;
        let format = aviutl2::input::ImageFormat { width, height };

        Ok(aviutl2::input::InputInfo {
            video: Some(aviutl2::input::VideoInputInfo {
                fps: 30,       // Default FPS for images
                scale: 1,      // No scaling
                num_frames: 1, // Single frame for image
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
            .map(|p| (p.0[0], p.0[1], p.0[2], p.0[3]))
            .collect::<Vec<_>>();
        buffer.into_image()
    }

    fn close(&self, handle: Self::InputHandle) -> bool {
        drop(handle);
        true
    }
}

register_input_plugin!(ImageRsPlugin);
