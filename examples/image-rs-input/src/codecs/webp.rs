use super::AnimationInfo;
use ordered_float::OrderedFloat;

// image-webpはフレームのデコードも行ってしまうため、自前でヘッダを読む
pub fn read_headers<R: std::io::Read + std::io::Seek>(
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
            "VP8 " => {
                anyhow::ensure!(
                    chunk_data.len() >= 10,
                    "VP8 chunk too small to contain width and height"
                );
                width = (chunk_data[6] as u32 | ((chunk_data[7] as u32) << 8)) & 0x3FFF;
                height = (chunk_data[8] as u32 | ((chunk_data[9] as u32) << 8)) & 0x3FFF;
            }
            "VP8L" => {
                anyhow::ensure!(
                    chunk_data.len() >= 5,
                    "VP8L chunk too small to contain width and height"
                );
                let b0 = chunk_data[1];
                let b1 = chunk_data[2];
                let b2 = chunk_data[3];
                let b3 = chunk_data[4];
                width = 1 + (((b1 as u32 & 0x3F) << 8) | (b0 as u32));
                height = 1 + (((b3 as u32 & 0x0F) << 10) | ((b2 as u32) << 2) | ((b1 as u32) >> 6));
            }
            "VP8X" => {
                anyhow::ensure!(
                    chunk_data.len() == 10,
                    "VP8X chunk must be exactly 10 bytes long"
                );
                width = 1 + read_u24_le(&chunk_data[4..7]);
                height = 1 + read_u24_le(&chunk_data[7..10]);
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

    Ok(AnimationInfo {
        width,
        height,
        frame_timings,
        length_in_seconds: total_duration,
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_webp_headers() {
        let data = include_bytes!("../../test_data/dummy.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_lossless_webp_headers() {
        let data = include_bytes!("../../test_data/dummy_lossless.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_static_webp_headers() {
        let data = include_bytes!("../../test_data/static.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_static_lossless_webp_headers() {
        let data = include_bytes!("../../test_data/dummy_static_lossless.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_animated_lossy_webp_headers() {
        // by sigma-axis: https://canary.discord.com/channels/1392018499072823327/1446412324851421255/1446418177096679568
        let data = include_bytes!("../../test_data/oklch_color_solid_1.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_animated_lossless_webp_headers() {
        // by sigma-axis: https://canary.discord.com/channels/1392018499072823327/1446412324851421255/1446425367857598676
        let data = include_bytes!("../../test_data/oklch_color_solid_lossless_1.webp");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }
}
