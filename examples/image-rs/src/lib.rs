use aviutl2::input::InputFilter;
use aviutl2::input::InputPlugin;

struct ImageRsPlugin {}

impl InputPlugin for ImageRsPlugin {
    type InputHandle = image::RgbaImage;

    fn new() -> Self {
        ImageRsPlugin {}
    }
    //
    // fn info(&self) -> aviutl2::input::InputPluginTable {
    //     aviutl2::input::InputPluginTable {
    //         name: "ImageRs Plugin".to_string(),
    //         filefilter: vec![InputFilter {
    //             name: "Image Files".to_string(),
    //             extensions: vec![
    //                 "png".to_string(),
    //                 "jpg".to_string(),
    //                 "jpeg".to_string(),
    //                 "bmp".to_string(),
    //             ],
    //         }],
    //         information: "A plugin to handle image files using the image crate.".to_string(),
    //         can_config: false,
    //     }
    // }

    fn open(&self, file: std::path::PathBuf) -> Option<Self::InputHandle> {
        match image::open(file) {
            Ok(img) => Some(img.to_rgba8()),
            Err(e) => {
                eprintln!("Failed to open image: {}", e);
                None
            }
        }
    }

    fn get_info(&self, handle: &Self::InputHandle) -> Result<aviutl2::input::InputInfo, String> {
        let width = handle.width() as i32;
        let height = handle.height() as i32;
        let format = aviutl2::input::InputFormat {
            width,
            height,
            pixel_format: aviutl2::input::PixelFormat::Rgba,
        };

        Ok(aviutl2::input::InputInfo {
            flag: aviutl2::input::InputType::Video,
            fps: 0,
            scale: 1,
            num_frames: 1,
            num_samples: 0,
        })
    }

    fn close(&self, handle: &mut Self::InputHandle) -> bool {
        todo!();
    }
}

