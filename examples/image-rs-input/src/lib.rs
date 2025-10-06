mod codecs;
use aviutl2::{
    FileFilter,
    input::{AnyResult, ImageBuffer, ImageReturner, InputPlugin, IntoImage as _, Rational32},
    register_input_plugin,
};
use image::{AnimationDecoder, GenericImageView};
use ordered_float::OrderedFloat;
use std::io::Seek;

struct ImageInputPlugin {}

enum ImageReader {
    Animated(OwnedFrames),
    Single(Box<dyn image::ImageDecoder>),
    SingleCached(ImageBuffer),
}

unsafe impl Send for ImageReader {}
unsafe impl Sync for ImageReader {}

#[ouroboros::self_referencing]
struct OwnedFrames {
    file: std::io::BufReader<std::fs::File>,
    format: image::ImageFormat,
    #[borrows(mut file)]
    #[covariant]
    frames: image::Frames<'this>,
}

impl OwnedFrames {
    fn reset(self) -> anyhow::Result<Self> {
        let heads = self.into_heads();
        let file = heads.file;

        into_frames(file, heads.format)
    }
}

struct ImageHandle {
    reader: Option<ImageReader>,
    current_frame: usize,
    format: aviutl2::input::InputPixelFormat,
    width: u32,
    height: u32,
    frame_timings: std::collections::BTreeMap<OrderedFloat<f32>, usize>,
    length_in_seconds: f32,
}

impl InputPlugin for ImageInputPlugin {
    type InputHandle = ImageHandle;

    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(Self {})
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
                "image-rs Input for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-input",
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
        match format {
            image::ImageFormat::Png | image::ImageFormat::Gif | image::ImageFormat::WebP => {
                let mut file = std::io::BufReader::new(std::fs::File::open(&file)?);
                let animation_info = match format {
                    image::ImageFormat::Png => codecs::read_apng_headers(&mut file)?,
                    image::ImageFormat::Gif => codecs::read_gif_headers(&mut file)?,
                    image::ImageFormat::WebP => codecs::read_webp_headers(&mut file)?,
                    _ => unreachable!(),
                };
                if animation_info.frame_timings.len() > 1 {
                    let frames = into_frames(file, format)?;
                    return Ok(ImageHandle {
                        current_frame: 0,
                        reader: Some(ImageReader::Animated(frames)),
                        format: aviutl2::input::InputPixelFormat::Bgra,
                        frame_timings: animation_info.frame_timings,
                        length_in_seconds: animation_info.length_in_seconds,
                        width: animation_info.width,
                        height: animation_info.height,
                    });
                }
            }
            _ => {}
        }
        let frames = into_frames(std::io::BufReader::new(std::fs::File::open(&file)?), format);
        // 自分が実装をミスっている可能性もあるので、codecsモジュールの関数でパースできなくてもimage-rsの実装でパースできるか試す
        if let Ok(mut frames) = frames {
            let (width, height, total_duration, frame_timings) =
                frames.with_frames_mut(|frames| {
                    let mut frame_timings = std::collections::BTreeMap::new();
                    let mut total_duration = 0.0;
                    let mut width = 0;
                    let mut height = 0;
                    for frame in frames {
                        let frame = frame?;
                        let delay = frame.delay().numer_denom_ms();
                        let duration = delay.0 as f32 / delay.1 as f32 / 1000.0;
                        if width == 0 && height == 0 {
                            let img = frame.into_buffer();
                            width = img.width();
                            height = img.height();
                        }
                        frame_timings.insert(OrderedFloat(total_duration), frame_timings.len());
                        total_duration += duration;
                    }

                    anyhow::Ok((width, height, total_duration, frame_timings))
                })?;
            if frame_timings.len() > 1 {
                return Ok(ImageHandle {
                    current_frame: 0,
                    reader: Some(ImageReader::Animated(frames.reset()?)),
                    format: aviutl2::input::InputPixelFormat::Bgra,
                    frame_timings,
                    length_in_seconds: total_duration,
                    width,
                    height,
                });
            }
        }

        let decoded = decoder.decode()?;
        let (width, height) = decoded.dimensions();
        let format = match decoded {
            image::DynamicImage::ImageRgb8(_) | image::DynamicImage::ImageRgba8(_) => {
                aviutl2::input::InputPixelFormat::Bgra
            }
            _ => aviutl2::input::InputPixelFormat::Pa64,
        };
        let mut frame_timings = std::collections::BTreeMap::new();
        frame_timings.insert(OrderedFloat(0.0), 0);

        Ok(ImageHandle {
            current_frame: 0,
            reader: Some(ImageReader::Single(Box::new(
                image::ImageReader::open(&file)?
                    .with_guessed_format()?
                    .into_decoder()?,
            ))),
            format,
            frame_timings,
            length_in_seconds: 0.0,
            width,
            height,
        })
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

    fn read_video_mut(
        &self,
        handle: &mut Self::InputHandle,
        frame: u32,
        returner: &mut ImageReturner,
    ) -> AnyResult<()> {
        let frame = frame as usize;
        anyhow::ensure!(
            frame < handle.frame_timings.len(),
            "Frame index out of bounds: {} >= {}",
            frame,
            handle.frame_timings.len()
        );
        let reader = handle.reader.take();
        match reader {
            None => anyhow::bail!("Reader is used up"),
            Some(ImageReader::Animated(frames)) => {
                let mut frames = if frame < handle.current_frame {
                    handle.current_frame = 0;
                    frames.reset()?
                } else {
                    frames
                };
                while handle.current_frame < frame {
                    frames.with_frames_mut(|frames| frames.next().transpose())?;
                    handle.current_frame += 1;
                }
                let frame = frames
                    .with_frames_mut(|frames| frames.next().transpose())?
                    .ok_or_else(|| anyhow::anyhow!("Failed to get frame {}", frame))?;
                handle.current_frame += 1;
                let img = frame.into_buffer();

                returner.write(&img);
                handle.reader = Some(ImageReader::Animated(frames));
            }
            Some(ImageReader::Single(decoder)) => {
                let img = image::DynamicImage::from_decoder(decoder)?;
                match handle.format {
                    aviutl2::input::InputPixelFormat::Bgra => {
                        let img = img.to_rgba8();
                        let buffer = img.into_image();
                        returner.write(&buffer);
                        handle.reader = Some(ImageReader::SingleCached(buffer));
                    }
                    aviutl2::input::InputPixelFormat::Pa64 => {
                        let img = img.to_rgba16();
                        let buffer = img.into_image();
                        returner.write(&buffer);
                        handle.reader = Some(ImageReader::SingleCached(buffer));
                    }
                    _ => unreachable!(),
                }
            }
            Some(ImageReader::SingleCached(img)) => {
                returner.write(&img);
                handle.reader = Some(ImageReader::SingleCached(img));
            }
        };

        Ok(())
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

fn into_frames(
    mut file: std::io::BufReader<std::fs::File>,
    format: image::ImageFormat,
) -> Result<OwnedFrames, anyhow::Error> {
    file.seek(std::io::SeekFrom::Start(0))?;
    OwnedFramesTryBuilder {
        file,
        format,
        frames_builder: |file| match format {
            image::ImageFormat::Gif => {
                let decoder = image::codecs::gif::GifDecoder::new(file)?;
                Ok(decoder.into_frames())
            }
            image::ImageFormat::WebP => {
                let decoder = image::codecs::webp::WebPDecoder::new(file)?;
                Ok(decoder.into_frames())
            }
            image::ImageFormat::Png => {
                let decoder = image::codecs::png::PngDecoder::new(file)?;
                Ok(decoder.apng()?.into_frames())
            }
            _ => anyhow::bail!("Format {:?} does not support animation", format),
        },
    }
    .try_build()
}

register_input_plugin!(ImageInputPlugin);
