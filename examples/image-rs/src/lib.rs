use aviutl2::input::InputFilter;
use aviutl2::input::InputPlugin;

struct ImageRsPlugin {}

impl InputPlugin for ImageRsPlugin {
    type InputHandle = image::RgbaImage;

    fn new() -> Self {
        ImageRsPlugin {}
    }

    fn info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "ImageRs Plugin".to_string(),
            filefilter: vec![InputFilter {
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
        todo!();
    }

    fn get_info(&self, handle: &Self::InputHandle) -> Result<aviutl2::input::InputInfo, String> {
        todo!();
    }

    fn close(&self, handle: &mut Self::InputHandle) -> bool {
        todo!();
    }
}
