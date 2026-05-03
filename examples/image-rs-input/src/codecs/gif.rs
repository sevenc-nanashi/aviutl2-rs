use super::AnimationInfo;
use ordered_float::OrderedFloat;

pub fn read_headers<R: std::io::Read + std::io::Seek>(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_gif_headers() {
        let data = include_bytes!("../../test_data/dummy.gif");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_static_gif_headers() {
        let data = include_bytes!("../../test_data/static.gif");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();
        insta::assert_debug_snapshot!(animation_info);
    }
}
