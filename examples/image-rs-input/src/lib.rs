use aviutl2::FileFilter;
use aviutl2::input::AnyResult;
use aviutl2::input::InputPlugin;
use aviutl2::input::IntoImage;
use aviutl2::register_input_plugin;

struct ImageRsInputPlugin {}

impl InputPlugin for ImageRsInputPlugin {
    type InputHandle = image::RgbaImage;

    fn new() -> Self {
        ImageRsInputPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "Rusty Images Input Plugin".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: vec![FileFilter {
                name: "Image Files".to_string(),
                extensions: vec!["webp".to_string()],
            }],
            information: format!(
                "image-rs Input for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-input",
                version = env!("CARGO_PKG_VERSION")
            ),
            can_config: false,
            concurrent: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> AnyResult<Self::InputHandle> {
        image::open(file)
            .map(|img| img.into_rgba8())
            .map_err(|e| anyhow::anyhow!("Failed to open image file: {}", e))
    }

    fn get_input_info(
        &self,
        handle: &mut Self::InputHandle,
        _video_track: u32,
        _audio_track: u32,
    ) -> AnyResult<aviutl2::input::InputInfo> {
        let width = handle.width();
        let height = handle.height();

        Ok(aviutl2::input::InputInfo {
            video: Some(aviutl2::input::VideoInputInfo {
                fps: aviutl2::input::Rational32::new(30, 1),
                num_frames: 1,
                width,
                height,
                format: aviutl2::input::ImageFormat::Bgra,
                manual_frame_index: false,
            }),
            audio: None, // No audio for image files
        })
    }

    fn read_video(&self, handle: &Self::InputHandle, frame: u32) -> AnyResult<impl IntoImage> {
        anyhow::ensure!(frame == 0, "Only frame 0 is valid for image input");
        let mut final_buffer =
            Vec::with_capacity(handle.width() as usize * handle.height() as usize * 4);
        let buffer_writer = final_buffer.spare_capacity_mut();
        let width = handle.width();
        let height = handle.height();
        for y in 0..handle.height() {
            for x in 0..handle.width() {
                let pixel = handle.get_pixel(x, y).0;
                let dest_idx = ((height - 1 - y) * width + x) as usize * 4;
                buffer_writer[dest_idx].write(pixel[2]);
                buffer_writer[dest_idx + 1].write(pixel[1]);
                buffer_writer[dest_idx + 2].write(pixel[0]);
                buffer_writer[dest_idx + 3].write(pixel[3]);
            }
        }
        unsafe {
            final_buffer.set_len(handle.width() as usize * handle.height() as usize * 4);
        }
        Ok(final_buffer)
    }

    fn close(&self, handle: Self::InputHandle) -> AnyResult<()> {
        drop(handle);
        Ok(())
    }
}

register_input_plugin!(ImageRsInputPlugin);
