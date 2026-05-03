use super::AnimationInfo;
use ordered_float::OrderedFloat;

pub fn read_headers<R: std::io::BufRead + std::io::Seek>(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_apng_headers() {
        let data = include_bytes!("../../test_data/dummy.apng");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }

    #[test]
    fn test_read_png_headers() {
        let data = include_bytes!("../../test_data/static.png");
        let mut cursor = std::io::Cursor::new(data.as_ref());
        let animation_info = read_headers(&mut cursor).unwrap();

        insta::assert_debug_snapshot!(animation_info);
    }
}
