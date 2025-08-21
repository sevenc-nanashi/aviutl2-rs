use aviutl2::FileFilter;
use aviutl2::input::{
    AnyResult, ImageFormat, InputInfo, InputPlugin, InputPluginTable, IntoImage, VideoInputInfo,
    f16,
};
use aviutl2::register_input_plugin;

struct PixelFormatTestPlugin;

#[derive(Clone)]
struct Handle {
    format: ImageFormat,
    width: u32,
    height: u32,
}

impl InputPlugin for PixelFormatTestPlugin {
    type InputHandle = Handle;

    fn new() -> Self {
        PixelFormatTestPlugin
    }

    fn plugin_info(&self) -> InputPluginTable {
        InputPluginTable {
            name: "Pixel Format Tester".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: vec![FileFilter {
                name: "Pixel Formats".to_string(),
                extensions: vec![
                    "bgra".to_string(),
                    "bgr".to_string(),
                    "yuy2".to_string(),
                    "pa64".to_string(),
                    "hf64".to_string(),
                    "yc48".to_string(),
                ],
            }],
            information: "Generates test patterns for various pixel formats".to_string(),
            can_config: false,
            concurrent: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> AnyResult<Self::InputHandle> {
        let format_str = file.extension().and_then(|s| s.to_str()).unwrap_or("bgra");
        let format = match format_str {
            "bgr" => ImageFormat::Bgr,
            "yuy2" => ImageFormat::Yuy2,
            "bgra" => ImageFormat::Bgra,
            "pa64" => ImageFormat::Pa64,
            "hf64" => ImageFormat::Hf64,
            "yc48" => ImageFormat::Yc48,
            _ => return Err(anyhow::anyhow!("Unsupported pixel format: {}", format_str)),
        };
        Ok(Handle {
            format,
            width: 256,
            height: 256,
        })
    }

    fn get_input_info(
        &self,
        handle: &mut Self::InputHandle,
        _video_track: u32,
        _audio_track: u32,
    ) -> AnyResult<InputInfo> {
        Ok(InputInfo {
            video: Some(VideoInputInfo {
                fps: aviutl2::input::Rational32::new(30, 1),
                num_frames: 1,
                width: handle.width,
                height: handle.height,
                format: handle.format,
                manual_frame_index: false,
            }),
            audio: None,
        })
    }

    fn read_video(&self, handle: &Self::InputHandle, frame: u32) -> AnyResult<impl IntoImage> {
        anyhow::ensure!(frame == 0, "Only frame 0 is valid");
        let (width, height) = (handle.width, handle.height);
        match handle.format {
            ImageFormat::Bgra => {
                let mut buffer = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        buffer.push((
                            x as u8,                                                  // B
                            y as u8,                                                  // G
                            ((x + y) as f64 / (width + height) as f64 * 255.0) as u8, // R
                            255, // A (fully opaque)
                        ));
                    }
                }
                Ok(buffer.into_image())
            }
            ImageFormat::Bgr => {
                let mut buffer = Vec::with_capacity((width * height * 3) as usize);
                for y in 0..height {
                    for x in 0..width {
                        buffer.push((
                            x as u8,                                                  // B
                            y as u8,                                                  // G
                            ((x + y) as f64 / (width + height) as f64 * 255.0) as u8, // R
                        ));
                    }
                }
                Ok(buffer.into_image())
            }
            ImageFormat::Yuy2 => {
                let mut buffer = Vec::with_capacity((width * height / 2) as usize);
                for y in 0..height {
                    for x in (0..width).step_by(2) {
                        let y0 = ((x + y) as f64 / (width + height) as f64 * 256.0) as u8; // Y0
                        let y1 = ((x + y + 1) as f64 / (width + height) as f64 * 256.0) as u8; // Y1
                        let u = ((x as f64 / width as f64) * 256.0) as u8; // U
                        let v = ((y as f64 / height as f64) * 256.0) as u8; // V
                        buffer.push((y0, u, y1, v)); // Y0, U, Y1, V
                    }
                }
                Ok(buffer.into_image())
            }
            ImageFormat::Pa64 => {
                let mut buffer = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        let r = (x as f64 / width as f64) * 65535.0;
                        let g = (y as f64 / height as f64) * 65535.0;
                        let b = ((x + y) as f64 / (width + height) as f64) * 65535.0;
                        buffer.push((
                            r as u16, g as u16, b as u16, 65535, // A (fully opaque)
                        ));
                    }
                }
                Ok(buffer.into_image())
            }
            ImageFormat::Hf64 => {
                let mut buffer = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        let r = x as f64 / width as f64;
                        let g = y as f64 / height as f64;
                        let b = (x + y) as f64 / (width + height) as f64;
                        buffer.push((
                            f16::from_f64(r),   // R
                            f16::from_f64(g),   // G
                            f16::from_f64(b),   // B
                            f16::from_f64(1.0), // A
                        ));
                    }
                }
                Ok(buffer.into_image())
            }
            ImageFormat::Yc48 => {
                let mut buffer = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        let y_val = (((x + y) as f64 / (width + height) as f64) * 4096.0) as i16; // Y
                        let cb = ((x as f64 / width as f64) * 4096.0 - 2048.0) as i16; // Cb
                        let cr = ((y as f64 / height as f64) * 4096.0 - 2048.0) as i16; // Cr
                        buffer.push((y_val, cb, cr)); // Y, Cb, Cr
                    }
                }
                Ok(buffer.into_image())
            }
        }
    }

    fn close(&self, _handle: Self::InputHandle) -> AnyResult<()> {
        Ok(())
    }
}

register_input_plugin!(PixelFormatTestPlugin);
