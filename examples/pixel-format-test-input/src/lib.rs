use aviutl2::input::{
    AnyResult, ImageReturner, InputInfo, InputPixelFormat, InputPlugin, InputPluginTable,
    VideoInputInfo, f16,
};
use aviutl2::register_input_plugin;

struct PixelFormatTestPlugin;

#[derive(Clone)]
struct Handle {
    format: InputPixelFormat,
    width: u32,
    height: u32,
}

impl InputPlugin for PixelFormatTestPlugin {
    type InputHandle = Handle;

    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        Ok(PixelFormatTestPlugin)
    }

    fn plugin_info(&self) -> InputPluginTable {
        InputPluginTable {
            name: "Pixel Format Tester".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: aviutl2::file_filters! {
                "Pixel Formats" => [
                    "bgra".to_string(),
                    "bgr".to_string(),
                    "yuy2".to_string(),
                    "pa64".to_string(),
                    "hf64".to_string(),
                    "yc48".to_string(),
                ],
            },
            information: format!(
                "Pixel Format Test Plugin / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/pixel-format-test-input",
                version = env!("CARGO_PKG_VERSION")
            ),
            can_config: false,
            concurrent: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> AnyResult<Self::InputHandle> {
        let format_str = file.extension().and_then(|s| s.to_str()).unwrap_or("bgra");
        let format = match format_str {
            "bgr" => InputPixelFormat::Bgr,
            "yuy2" => InputPixelFormat::Yuy2,
            "bgra" => InputPixelFormat::Bgra,
            "pa64" => InputPixelFormat::Pa64,
            "hf64" => InputPixelFormat::Hf64,
            "yc48" => InputPixelFormat::Yc48,
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

    fn read_video(
        &self,
        handle: &Self::InputHandle,
        frame: u32,
        returner: &mut ImageReturner,
    ) -> AnyResult<()> {
        anyhow::ensure!(frame == 0, "Only frame 0 is valid");
        let (width, height) = (handle.width, handle.height);
        match handle.format {
            InputPixelFormat::Bgra => {
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
                returner.write(&buffer);
            }
            InputPixelFormat::Bgr => {
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
                returner.write(&buffer);
            }
            InputPixelFormat::Yuy2 => {
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
                returner.write(&buffer);
            }
            InputPixelFormat::Pa64 => {
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
                returner.write(&buffer);
            }
            InputPixelFormat::Hf64 => {
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
                returner.write(&buffer);
            }
            InputPixelFormat::Yc48 => {
                let mut buffer = Vec::with_capacity((width * height) as usize);
                for y in 0..height {
                    for x in 0..width {
                        let y_val = (((x + y) as f64 / (width + height) as f64) * 4096.0) as i16; // Y
                        let cb = ((x as f64 / width as f64) * 4096.0 - 2048.0) as i16; // Cb
                        let cr = ((y as f64 / height as f64) * 4096.0 - 2048.0) as i16; // Cr
                        buffer.push((y_val, cb, cr)); // Y, Cb, Cr
                    }
                }
                returner.write(&buffer);
            }
        }

        Ok(())
    }

    fn close(&self, _handle: Self::InputHandle) -> AnyResult<()> {
        Ok(())
    }
}

register_input_plugin!(PixelFormatTestPlugin);
