use aviutl2::input::{ImageBuffer, IntoImage};
use jxl::api::{
    Endianness, JxlBasicInfo, JxlBitDepth, JxlColorType, JxlDataFormat, JxlDecoder,
    JxlDecoderOptions, JxlOutputBuffer, JxlPixelFormat, ProcessingResult, VisibleFrameInfo, states,
};
use ordered_float::OrderedFloat;
use std::io::Read;

pub struct OpenedImage {
    pub reader: Reader,
    pub format: aviutl2::input::InputPixelFormat,
    pub width: u32,
    pub height: u32,
    pub frame_timings: std::collections::BTreeMap<OrderedFloat<f32>, usize>,
    pub length_in_seconds: f32,
}

pub struct Reader {
    file: std::path::PathBuf,
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Rgba8,
    Rgba16,
}

pub fn is_file(path: &std::path::Path) -> anyhow::Result<bool> {
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jxl"))
    {
        return Ok(true);
    }

    let mut file = std::fs::File::open(path)?;
    let mut header = [0; 12];
    let len = file.read(&mut header)?;
    Ok(matches!(
        jxl::api::check_signature(&header[..len]),
        ProcessingResult::Complete { result: Some(_) }
    ))
}

pub fn open(file: std::path::PathBuf) -> anyhow::Result<OpenedImage> {
    let data = std::fs::read(&file)?;
    let (basic_info, scanned_frames) = scan_info(&data)?;
    let (width, height) = basic_info.size;
    let output_format = output_format(&basic_info.bit_depth);
    let format = match output_format {
        OutputFormat::Rgba8 => aviutl2::input::InputPixelFormat::Bgra,
        OutputFormat::Rgba16 => aviutl2::input::InputPixelFormat::Pa64,
    };
    let (frame_timings, length_in_seconds) = frame_timings(&basic_info, &scanned_frames);

    Ok(OpenedImage {
        reader: Reader {
            file,
            output_format,
        },
        format,
        frame_timings,
        length_in_seconds,
        width: width as u32,
        height: height as u32,
    })
}

pub fn decode_frame(reader: &Reader, target_frame: usize) -> anyhow::Result<ImageBuffer> {
    let data = std::fs::read(&reader.file)?;
    let options = JxlDecoderOptions::default();
    let decoder = JxlDecoder::<states::Initialized>::new(options);
    let mut input = data.as_slice();
    let mut decoder = advance_to_image_info(decoder, &mut input)?;
    let basic_info = decoder.basic_info().clone();
    decoder.set_pixel_format(pixel_format(
        reader.output_format,
        basic_info.extra_channels.len(),
    ));

    for frame in 0..=target_frame {
        anyhow::ensure!(
            decoder.has_more_frames(),
            "JXL frame index out of bounds: {target_frame}"
        );

        let decoder_with_frame = advance_to_frame_info(decoder, &mut input)?;
        if frame == target_frame {
            return decode_current_frame(
                decoder_with_frame,
                &mut input,
                basic_info.size,
                reader.output_format,
            );
        }
        decoder = skip_frame(decoder_with_frame, &mut input)?;
    }

    unreachable!()
}

fn scan_info(data: &[u8]) -> anyhow::Result<(JxlBasicInfo, Vec<VisibleFrameInfo>)> {
    let mut options = JxlDecoderOptions::default();
    options.scan_frames_only = true;
    options.skip_preview = false;
    let decoder = JxlDecoder::<states::Initialized>::new(options);
    let mut input = data;
    let mut decoder = advance_to_image_info(decoder, &mut input)?;
    let basic_info = decoder.basic_info().clone();

    while decoder.has_more_frames() {
        let decoder_with_frame = advance_to_frame_info(decoder, &mut input)?;
        decoder = skip_frame(decoder_with_frame, &mut input)?;
    }

    Ok((basic_info, decoder.scanned_frames().to_vec()))
}

fn frame_timings(
    basic_info: &JxlBasicInfo,
    scanned_frames: &[VisibleFrameInfo],
) -> (std::collections::BTreeMap<OrderedFloat<f32>, usize>, f32) {
    let mut frame_timings = std::collections::BTreeMap::new();
    frame_timings.insert(OrderedFloat(0.0), 0);

    if basic_info.animation.is_none() || scanned_frames.len() <= 1 {
        return (frame_timings, 0.0);
    }

    let mut total_duration = 0.0f32;
    frame_timings.clear();
    for frame in scanned_frames {
        frame_timings.insert(OrderedFloat(total_duration), frame.index);
        total_duration += (frame.duration_ms / 1000.0) as f32;
    }

    (frame_timings, total_duration)
}

fn output_format(bit_depth: &JxlBitDepth) -> OutputFormat {
    match bit_depth {
        JxlBitDepth::Int { bits_per_sample } if *bits_per_sample <= 8 => OutputFormat::Rgba8,
        _ => OutputFormat::Rgba16,
    }
}

fn pixel_format(output_format: OutputFormat, extra_channels: usize) -> JxlPixelFormat {
    let color_data_format = match output_format {
        OutputFormat::Rgba8 => JxlDataFormat::U8 { bit_depth: 8 },
        OutputFormat::Rgba16 => JxlDataFormat::U16 {
            endianness: Endianness::native(),
            bit_depth: 16,
        },
    };

    JxlPixelFormat {
        color_type: JxlColorType::Rgba,
        color_data_format: Some(color_data_format),
        extra_channel_format: vec![None; extra_channels],
    }
}

fn decode_current_frame(
    decoder: JxlDecoder<states::WithFrameInfo>,
    input: &mut &[u8],
    (width, height): (usize, usize),
    output_format: OutputFormat,
) -> anyhow::Result<ImageBuffer> {
    match output_format {
        OutputFormat::Rgba8 => {
            let bytes_per_row = width * 4;
            let mut img = vec![0u8; bytes_per_row * height];
            process_frame_into_buffer(decoder, input, &mut img, height, bytes_per_row)?;
            crate::alpha::straight_alpha_to_premultiplied_alpha(&mut img);
            aviutl2::utils::flip_vertical(&mut img, width * 4, height);
            aviutl2::utils::rgba_to_bgra_bytes(&mut img);
            Ok(ImageBuffer(img))
        }
        OutputFormat::Rgba16 => {
            let bytes_per_row = width * 4 * std::mem::size_of::<u16>();
            let mut img = vec![0u16; width * height * 4];
            // SAFETY: the byte slice covers the same initialized Vec allocation and preserves
            // the Vec<u16> alignment required by jxl's u16 output path.
            let img_bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    img.as_mut_ptr().cast::<u8>(),
                    img.len() * std::mem::size_of::<u16>(),
                )
            };
            process_frame_into_buffer(decoder, input, img_bytes, height, bytes_per_row)?;
            crate::alpha::straight_alpha_to_premultiplied_alpha_u16(&mut img);
            Ok(img.into_image())
        }
    }
}

fn process_frame_into_buffer(
    mut decoder: JxlDecoder<states::WithFrameInfo>,
    input: &mut &[u8],
    img: &mut [u8],
    height: usize,
    bytes_per_row: usize,
) -> anyhow::Result<()> {
    let decoder = {
        let mut buffers = [JxlOutputBuffer::new(img, height, bytes_per_row)];
        loop {
            match decoder.process(input, &mut buffers)? {
                ProcessingResult::Complete { result } => break result,
                ProcessingResult::NeedsMoreInput { fallback, .. } => {
                    anyhow::ensure!(!input.is_empty(), "Unexpected end of JXL input");
                    decoder = fallback;
                }
            }
        }
    };
    drop(decoder);
    Ok(())
}

fn advance_to_image_info(
    mut decoder: JxlDecoder<states::Initialized>,
    input: &mut &[u8],
) -> anyhow::Result<JxlDecoder<states::WithImageInfo>> {
    loop {
        match decoder.process(input)? {
            ProcessingResult::Complete { result } => return Ok(result),
            ProcessingResult::NeedsMoreInput { fallback, .. } => {
                anyhow::ensure!(!input.is_empty(), "Unexpected end of JXL input");
                decoder = fallback;
            }
        }
    }
}

fn advance_to_frame_info(
    mut decoder: JxlDecoder<states::WithImageInfo>,
    input: &mut &[u8],
) -> anyhow::Result<JxlDecoder<states::WithFrameInfo>> {
    loop {
        match decoder.process(input)? {
            ProcessingResult::Complete { result } => return Ok(result),
            ProcessingResult::NeedsMoreInput { fallback, .. } => {
                anyhow::ensure!(!input.is_empty(), "Unexpected end of JXL input");
                decoder = fallback;
            }
        }
    }
}

fn skip_frame(
    mut decoder: JxlDecoder<states::WithFrameInfo>,
    input: &mut &[u8],
) -> anyhow::Result<JxlDecoder<states::WithImageInfo>> {
    loop {
        match decoder.skip_frame(input)? {
            ProcessingResult::Complete { result } => return Ok(result),
            ProcessingResult::NeedsMoreInput { fallback, .. } => {
                anyhow::ensure!(!input.is_empty(), "Unexpected end of JXL input");
                decoder = fallback;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jxl_fixture() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("5_frames_numbered_jxli.jxl")
    }

    #[test]
    fn scan_jxl_animation_info() {
        let data = std::fs::read(jxl_fixture()).unwrap();
        let (basic_info, scanned_frames) = scan_info(&data).unwrap();
        let (frame_timings, length_in_seconds) = frame_timings(&basic_info, &scanned_frames);

        assert!(basic_info.size.0 > 0);
        assert!(basic_info.size.1 > 0);
        assert!(basic_info.animation.is_some());
        assert_eq!(scanned_frames.len(), 5);
        assert_eq!(frame_timings.len(), 5);
        assert!(length_in_seconds > 0.0);
    }

    #[test]
    fn decode_jxl_first_frame() {
        let path = jxl_fixture();
        let data = std::fs::read(&path).unwrap();
        let (basic_info, _) = scan_info(&data).unwrap();
        let output_format = output_format(&basic_info.bit_depth);
        let reader = Reader {
            file: path,
            output_format,
        };
        let decoded = decode_frame(&reader, 0).unwrap();
        let bytes_per_sample = match output_format {
            OutputFormat::Rgba8 => 1,
            OutputFormat::Rgba16 => 2,
        };

        assert_eq!(
            decoded.len(),
            basic_info.size.0 * basic_info.size.1 * 4 * bytes_per_sample
        );
    }
}
