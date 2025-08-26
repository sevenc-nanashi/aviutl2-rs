use aviutl2::{
    FileFilter,
    input::{AnyResult, ImageBuffer, InputPlugin, IntoImage, Rational32},
    register_input_plugin,
};
use image::AnimationDecoder;
use ordered_float::OrderedFloat;

struct ImageInputPlugin {}

struct ImageHandle {
    inner: Vec<ImageBuffer>,
    format: aviutl2::input::ImageFormat,
    width: u32,
    height: u32,
    frame_timings: std::collections::BTreeMap<OrderedFloat<f32>, usize>,
    length_in_seconds: f32,
}

impl InputPlugin for ImageInputPlugin {
    type InputHandle = ImageHandle;

    fn new() -> Self {
        ImageInputPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "Rusty Image Input".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: vec![FileFilter {
                name: "Image Files".to_string(),
                extensions: vec![
                    "webp".to_string(),
                    "png".to_string(),
                    "apng".to_string(),
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "bmp".to_string(),
                    "tiff".to_string(),
                    "gif".to_string(),
                    "hdr".to_string(),
                ],
            }],
            information: format!(
                "ril & image-rs Input for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-input",
                version = env!("CARGO_PKG_VERSION")
            ),
            can_config: false,
            concurrent: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> AnyResult<Self::InputHandle> {
        let decoder = image::ImageReader::open(&file)?.with_guessed_format()?;
        let format = decoder
            .format()
            .ok_or_else(|| anyhow::anyhow!("Failed to guess image format"))?;
        let frames = match format {
            image::ImageFormat::Gif => {
                let decoder = image::codecs::gif::GifDecoder::new(std::io::BufReader::new(
                    std::fs::File::open(&file)?,
                ))?;
                Some(decoder.into_frames())
            }
            image::ImageFormat::WebP => {
                let decoder = image::codecs::webp::WebPDecoder::new(std::io::BufReader::new(
                    std::fs::File::open(&file)?,
                ))?;
                decoder.has_animation().then(|| decoder.into_frames())
            }
            image::ImageFormat::Png => {
                let decoder = image::codecs::png::PngDecoder::new(std::io::BufReader::new(
                    std::fs::File::open(&file)?,
                ))?;
                decoder
                    .is_apng()?
                    .then(|| decoder.apng())
                    .transpose()?
                    .map(|apng| apng.into_frames())
            }
            _ => None,
        };

        match frames {
            Some(frames) => {
                let frames = frames.collect_frames()?;
                let mut inner = Vec::with_capacity(frames.len());
                let mut frame_timings = std::collections::BTreeMap::new();
                let mut total_duration = 0.0;
                let mut width = 0;
                let mut height = 0;
                for frame in frames {
                    let delay = frame.delay().numer_denom_ms();
                    let duration = delay.0 as f32 / delay.1 as f32 / 1000.0;
                    let img = frame.into_buffer();
                    let mut img_pixels = img
                        .pixels()
                        .map(|p| (p.0[2], p.0[1], p.0[0], p.0[3]))
                        .collect::<Vec<_>>();
                    if width == 0 && height == 0 {
                        width = img.width();
                        height = img.height();
                    } else {
                        anyhow::ensure!(
                            width == img.width() && height == img.height(),
                            "All frames must have the same dimensions"
                        );
                    }
                    aviutl2::utils::flip_vertical(
                        &mut img_pixels,
                        img.width() as usize,
                        img.height() as usize,
                    );
                    inner.push(img_pixels.into_image());

                    frame_timings.insert(OrderedFloat(total_duration), inner.len() - 1);
                    total_duration += duration;
                }
                anyhow::ensure!(!inner.is_empty(), "No frames found in the image");

                Ok(ImageHandle {
                    inner,
                    format: aviutl2::input::ImageFormat::Bgra,
                    frame_timings,
                    length_in_seconds: total_duration,
                    width,
                    height,
                })
            }
            None => {
                let decoded = decoder.decode()?;
                let (width, height) = (decoded.width(), decoded.height());
                let (format, img) = match decoded {
                    img @ (image::DynamicImage::ImageRgb8(_)
                    | image::DynamicImage::ImageRgba8(_)) => {
                        let mut img_pixels = img
                            .to_rgba8()
                            .pixels()
                            .map(|p| (p.0[2], p.0[1], p.0[0], p.0.get(3).copied().unwrap_or(255)))
                            .collect::<Vec<_>>();
                        aviutl2::utils::flip_vertical(
                            &mut img_pixels,
                            img.width() as usize,
                            img.height() as usize,
                        );
                        (aviutl2::input::ImageFormat::Bgra, img_pixels.into_image())
                    }
                    img => {
                        let img = img.to_rgba16();
                        let img_pixels = img
                            .pixels()
                            .map(|p| (p.0[0], p.0[1], p.0[2], p.0[3]))
                            .collect::<Vec<_>>();
                        (aviutl2::input::ImageFormat::Pa64, img_pixels.into_image())
                    }
                };
                let inner = vec![img];
                let mut frame_timings = std::collections::BTreeMap::new();
                frame_timings.insert(OrderedFloat(0.0), 0);

                Ok(ImageHandle {
                    inner,
                    format,
                    frame_timings,
                    length_in_seconds: 0.0,
                    width,
                    height,
                })
            }
        }
    }

    fn get_input_info(
        &self,
        handle: &mut Self::InputHandle,
        _video_track: u32,
        _audio_track: u32,
    ) -> AnyResult<aviutl2::input::InputInfo> {
        let fps = if handle.frame_timings.len() > 1 {
            let total_duration = handle.length_in_seconds;
            let frame_count = handle.frame_timings.len() as f32;
            let fps = frame_count / total_duration;
            Rational32::new((fps * 1000.0).round() as i32, 1000)
        } else {
            Rational32::new(1, 1)
        };

        Ok(aviutl2::input::InputInfo {
            video: Some(aviutl2::input::VideoInputInfo {
                fps,
                num_frames: handle.frame_timings.len() as u32,
                width: handle.width,
                height: handle.height,
                format: handle.format,
                manual_frame_index: true,
            }),
            audio: None, // No audio for image files
        })
    }

    fn read_video(&self, handle: &Self::InputHandle, frame: u32) -> AnyResult<impl IntoImage> {
        let frame = frame as usize;
        anyhow::ensure!(
            frame < handle.inner.len(),
            "Frame index out of bounds: {} >= {}",
            frame,
            handle.inner.len()
        );
        let img = &handle.inner[frame];

        Ok(img.to_owned())
    }

    fn time_to_frame(
        &self,
        handle: &mut Self::InputHandle,
        _track: u32,
        time: f64,
    ) -> AnyResult<u32> {
        if handle.frame_timings.len() == 1 {
            return Ok(0);
        }
        if handle.length_in_seconds == 0.0 {
            return Ok(0);
        }

        let time = OrderedFloat((time % (handle.length_in_seconds as f64)) as f32);
        let (&_, &frame) = handle
            .frame_timings
            .range(..=time)
            .next_back()
            .expect("unreachable: ensure at least one frame");

        Ok(frame as u32)
    }

    fn close(&self, handle: Self::InputHandle) -> AnyResult<()> {
        drop(handle);
        Ok(())
    }
}

register_input_plugin!(ImageInputPlugin);
