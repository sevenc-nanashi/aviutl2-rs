use ordered_float::OrderedFloat;

#[derive(Debug, Clone)]
pub struct AnimationInfo {
    pub width: u32,
    pub height: u32,
    pub frame_timings: std::collections::BTreeMap<OrderedFloat<f32>, usize>,
    pub length_in_seconds: f32,
}

pub fn read_apng_headers<R: std::io::BufRead + std::io::Seek>(
    reader: &mut R,
) -> Result<AnimationInfo, anyhow::Error> {
    reader.seek(std::io::SeekFrom::Start(0))?;
    let png = png::Decoder::new(reader);
    let mut png = png.read_info()?;
    let info = png.info();
    let width = info.width;
    let height = info.height;

    let mut frame_timings = std::collections::BTreeMap::new();
    frame_timings.insert(OrderedFloat(0.0), 0);
    let Some(animation_control) = info.animation_control() else {
        return Ok(AnimationInfo {
            width,
            height,
            frame_timings,
            length_in_seconds: 0.0,
        });
    };
    let frame_info = png
        .info()
        .frame_control()
        .expect("APNG should have frame control");
    let mut total_duration = frame_info.delay_num as f32 / frame_info.delay_den as f32;
    for _ in 1..animation_control.num_frames {
        frame_timings.insert(OrderedFloat(total_duration), frame_timings.len());
        let frame_info = png.next_frame_info()?;
        let delay_seconds = frame_info.delay_num as f32 / frame_info.delay_den as f32;
        total_duration += delay_seconds;
    }
    Ok(AnimationInfo {
        width,
        height,
        frame_timings,
        length_in_seconds: total_duration,
    })
}

pub fn read_gif_headers<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
) -> Result<AnimationInfo, anyhow::Error> {
    reader.seek(std::io::SeekFrom::Start(0))?;
    let mut options = gif::DecodeOptions::new();
    options.skip_frame_decoding(true);
    let mut reader = options.read_info(reader)?;
    let width = reader.width() as u32;
    let height = reader.height() as u32;

    let mut frame_timings = std::collections::BTreeMap::new();
    frame_timings.insert(OrderedFloat(0.0), 0);
    let mut total_duration = 0.0;
    let mut frame_index = 0;
    while let Some(frame) = reader.read_next_frame()? {
        let delay_seconds = (frame.delay as f32) / 100.0; // delay is in hundredths of a second
        total_duration += delay_seconds;
        frame_index += 1;
        frame_timings.insert(OrderedFloat(total_duration), frame_index);
    }
    Ok(AnimationInfo {
        width,
        height,
        frame_timings,
        length_in_seconds: total_duration,
    })
}

// image-webpはフレームのデコードも行ってしまうため、自前でヘッダを読む
pub fn read_webp_headers<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
) -> Result<AnimationInfo, anyhow::Error> {
    reader.seek(std::io::SeekFrom::Start(0))?;
    let mut header = [0; 12];
    reader.read_exact(&mut header)?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WEBP" {
        return Err(anyhow::anyhow!("Not a valid WebP file"));
    }

    let mut width = 0;
    let mut height = 0;
    let mut frame_timings = std::collections::BTreeMap::new();
    let mut total_duration = 0.0f32;

    while let Some((chunk_type, chunk_data)) = read_chunk(reader) {
        match chunk_type.as_str() {
            "VP8 " | "VP8L" => {
                anyhow::ensure!(
                    chunk_data.len() >= 10,
                    "VP8 chunk too small to contain width and height"
                );
                width = u16::from_le_bytes(chunk_data[6..8].try_into().unwrap()) as u32;
                height = u16::from_le_bytes(chunk_data[8..10].try_into().unwrap()) as u32;
            }
            "ANIM" => {
                anyhow::ensure!(
                    chunk_data.len() == 6,
                    "ANIM chunk must be exactly 6 bytes long"
                );
            }
            "ANMF" => {
                anyhow::ensure!(
                    chunk_data.len() >= 20,
                    "ANMF chunk too small to contain frame data"
                );
                frame_timings.insert(OrderedFloat(total_duration), frame_timings.len());
                let _frame_x = read_u24_le(chunk_data[0..3].try_into().unwrap()) * 2;
                let _frame_y = read_u24_le(chunk_data[3..6].try_into().unwrap()) * 2;
                let frame_width = read_u24_le(chunk_data[6..9].try_into().unwrap()) + 1;
                let frame_height = read_u24_le(chunk_data[9..12].try_into().unwrap()) + 1;
                if width == 0 {
                    width = frame_width;
                }
                if height == 0 {
                    height = frame_height;
                }
                let frame_duration = read_u24_le(chunk_data[12..15].try_into().unwrap());
                let delay_seconds = (frame_duration as f32) / 1000.0;
                total_duration += delay_seconds;
            }
            _ => {}
        }
    }

    anyhow::ensure!(width > 0 && height > 0, "Failed to read image dimensions");

    return Ok(AnimationInfo {
        width,
        height,
        frame_timings,
        length_in_seconds: total_duration,
    });

    fn read_chunk<R: std::io::Read>(reader: &mut R) -> Option<(String, Vec<u8>)> {
        let mut chunk_header = [0; 8];
        if reader.read_exact(&mut chunk_header).is_err() {
            return None;
        }
        let chunk_type = String::from_utf8_lossy(&chunk_header[0..4]).to_string();
        let chunk_size = u32::from_le_bytes(chunk_header[4..8].try_into().unwrap()) as usize;
        let mut chunk_data = vec![0; chunk_size];
        if reader.read_exact(&mut chunk_data).is_err() {
            return None;
        }
        // Chunks are padded to even sizes
        if chunk_size % 2 == 1 {
            let mut padding = [0; 1];
            let _ = reader.read_exact(&mut padding);
        }
        Some((chunk_type, chunk_data))
    }

    fn read_u24_le(bytes: &[u8]) -> u32 {
        (bytes[0] as u32) | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_apng_headers() {
        let data = include_bytes!("../test_data/dummy.apng");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_apng_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_png_headers() {
        let data = include_bytes!("../test_data/static.png");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_apng_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_gif_headers() {
        let data = include_bytes!("../test_data/dummy.gif");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_gif_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_static_gif_headers() {
        let data = include_bytes!("../test_data/static.gif");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_gif_headers(&mut cursor).unwrap();
        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_webp_headers() {
        let data = include_bytes!("../test_data/dummy.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_webp_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_lossless_webp_headers() {
        let data = include_bytes!("../test_data/dummy_lossless.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_webp_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_static_webp_headers() {
        let data = include_bytes!("../test_data/static.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_webp_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }
}
