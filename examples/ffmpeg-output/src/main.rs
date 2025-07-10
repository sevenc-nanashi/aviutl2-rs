use std::io::Write;
fn main() {
    // test ffmpeg
    let current_frame = std::sync::atomic::AtomicUsize::new(0);
    let current_frame = std::sync::Arc::new(current_frame);
    let width = 256;
    let height = 256;

    let context = ez_ffmpeg::FfmpegContext::builder()
        .input(
            ez_ffmpeg::Input::new_by_read_callback(move |buf| {
                let mut current = 0;
                while current * 4 + 3 < buf.len() {
                    let frame_index = current_frame.load(std::sync::atomic::Ordering::Relaxed);
                    if frame_index >= 30 * width * height {
                        break;
                    }
                    // Generate a simple color pattern for testing
                    let r = (frame_index % 256) as u8;
                    let g = ((frame_index + 85) % 256) as u8;
                    let b = ((frame_index + 170) % 256) as u8;
                    buf[current * 4] = r;
                    buf[current * 4 + 1] = g;
                    buf[current * 4 + 2] = b;
                    buf[current * 4 + 3] = 255; // Alpha channel
                    current += 1;
                    current_frame.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                if current == 0 {
                    ffmpeg_sys_next::AVERROR_EOF
                } else {
                    (current * 4) as i32
                }
            })
            .set_format("rawvideo")
            .set_input_opt("probesize", "32")
            .set_input_opt("analyzeduration", "0")
            .set_input_opt("pix_fmt", "rgba")
            .set_input_opt("video_size", &format!("{}x{}", width, height))
            .set_input_opt("framerate", "30")
        )
        .output(
            ez_ffmpeg::Output::from("./test.mov")
                .set_format("mov")
                .set_video_codec("libx264")
                .set_format_opt("pix_fmt", "yuv420p")
                .set_format_opt("movflags", "+faststart")
                .set_format_opt("preset", "ultrafast")
                .set_format_opt("crf", "23"),
        )
        .build()
        .expect("Failed to build FFmpeg context");

    let scheduler = ez_ffmpeg::FfmpegScheduler::new(context);
    let result = scheduler
        .start()
        .expect("Failed to start FFmpeg job")
        .wait();
    if let Err(e) = result {
        eprintln!("FFmpeg job failed: {}", e);
    } else {
        println!("FFmpeg job completed successfully");
    }
    // Free any leaked memory if necessary
}
