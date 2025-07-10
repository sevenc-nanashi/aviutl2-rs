use aviutl2::{output::OutputPlugin, register_output_plugin};
use std::sync::Arc;

struct FfmpegOutputPlugin {}

impl OutputPlugin for FfmpegOutputPlugin {
    fn new() -> Self {
        FfmpegOutputPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::output::OutputPluginTable {
        aviutl2::output::OutputPluginTable {
            name: "FFmpeg Output Plugin".to_string(),
            input_type: aviutl2::output::OutputType::Both,
            file_filters: vec![aviutl2::output::FileFilter {
                name: "Video Files".to_string(),
                extensions: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string()],
            }],
            information: "Outputs video and audio using FFmpeg".to_string(),
            can_config: true,
        }
    }

    fn output(&self, info: aviutl2::output::OutputInfo) -> aviutl2::AnyResult<()> {
        let output = ez_ffmpeg::Output::new(&*info.path.to_string_lossy());
        let killed = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let (video_input, video_tx) = match &info.video {
            Some(video) => {
                let buf_size = video.fps.to_integer() as usize;

                let (tx, rx) = std::sync::mpsc::sync_channel::<(u8, u8, u8, u8)>(buf_size);
                (
                    Some(
                        ez_ffmpeg::Input::new_by_read_callback(move |buf| {
                            let mut current = 0;
                            dbg!(buf.len());
                            while (current + 1) * 4 < buf.len() {
                                let Ok(read) = rx.recv() else {
                                    break;
                                };
                                buf[current * 4] = read.0;
                                buf[current * 4 + 1] = read.1;
                                buf[current * 4 + 2] = read.2;
                                buf[current * 4 + 3] = read.3;
                                current += 1;
                            }

                            if current == 0 {
                                ffmpeg_sys_next::AVERROR_EOF
                            } else {
                                (current * 4) as i32
                            }
                        })
                        .set_format("rawvideo")
                        .set_video_codec("rawvideo")
                        .set_input_opt("pixel_format", "rgba")
                        .set_input_opt("video_size", &format!("{}x{}", video.width, video.height))
                        .set_input_opt("framerate", &format!("{}", video.fps.to_integer())),
                    ),
                    Some(tx),
                )
            }
            None => (None, None),
        };
        let (audio_input, audio_tx) = match &info.audio {
            Some(audio) => {
                let (tx, rx) =
                    std::sync::mpsc::sync_channel::<(f32, f32)>(audio.sample_rate as usize);
                (
                    Some(
                        ez_ffmpeg::Input::new_by_read_callback(move |buf| {
                            let mut current = 0;
                            while (current + 1) * 8 < buf.len() {
                                let Ok((read_l, read_r)) = rx.recv() else {
                                    break;
                                };
                                let l_bytes = read_l.to_le_bytes();
                                let r_bytes = read_r.to_le_bytes();
                                for i in 0..4 {
                                    buf[current * 8 + i] = l_bytes[i];
                                    buf[current * 8 + i + 4] = r_bytes[i];
                                }

                                current += 1;
                            }

                            if current == 0 {
                                ffmpeg_sys_next::AVERROR_EOF
                            } else {
                                (current * 8) as i32
                            }
                        })
                        .set_format("f32le")
                        .set_audio_codec("pcm_f32le")
                        .set_input_opt("sample_rate", &format!("{}", audio.sample_rate))
                        .set_input_opt(
                            "ch_layout",
                            if audio.num_channels == 2 {
                                "stereo"
                            } else {
                                "mono"
                            },
                        ),
                    ),
                    Some(tx),
                )
            }
            None => (None, None),
        };

        let mut threads: Vec<std::thread::JoinHandle<anyhow::Result<()>>> = Vec::new();
        threads.push(
            std::thread::Builder::new()
                .name("aviutl2_ffmpeg_output".to_string())
                .spawn({
                    let killed = Arc::clone(&killed);
                    move || -> anyhow::Result<()> {
                        let f = match (video_input, audio_input) {
                            (Some(video), Some(audio)) => ez_ffmpeg::FfmpegContext::builder()
                                .inputs(vec![video, audio])
                                .output(output)
                                .build()?,
                            (Some(video), None) => ez_ffmpeg::FfmpegContext::builder()
                                .input(video)
                                .output(output)
                                .build()?,
                            (None, Some(audio)) => ez_ffmpeg::FfmpegContext::builder()
                                .input(audio)
                                .output(output)
                                .build()?,
                            (None, None) => {
                                return Err(anyhow::anyhow!("No video or audio input provided"));
                            }
                        };
                        let ctx = ez_ffmpeg::FfmpegScheduler::new(f).start()?;
                        while !killed.load(std::sync::atomic::Ordering::Relaxed) && !ctx.is_ended()
                        {
                            std::thread::yield_now();
                        }

                        if killed.load(std::sync::atomic::Ordering::Relaxed) {
                            return Err(anyhow::anyhow!("Output was killed"));
                        }

                        ctx.wait()?;
                        Ok(())
                    }
                })?,
        );

        let sample_rate = info.audio.as_ref().map(|a| a.sample_rate);
        let info = Arc::new(info);

        if let Some(tx) = video_tx {
            threads.push(
                std::thread::Builder::new()
                    .name("aviutl2_ffmpeg_video_output".to_string())
                    .spawn({
                        let info = Arc::clone(&info);
                        move || -> anyhow::Result<()> {
                            for (i, frames) in info.get_video_frames_iter() {
                                for frame in frames {
                                    tx.send(frame).expect("Failed to send video frame");
                                }
                            }

                            Ok(())
                        }
                    })?,
            );
        }

        if let (Some(sample_rate), Some(tx)) = (sample_rate, audio_tx) {
            threads.push(
                std::thread::Builder::new()
                    .name("aviutl2_ffmpeg_audio_output".to_string())
                    .spawn(move || -> anyhow::Result<()> {
                        for (i, samples) in
                            info.get_stereo_audio_samples_iter((sample_rate / 10) as i32)
                        {
                            for sample in samples {
                                tx.send(sample).expect("Failed to send audio sample");
                            }
                        }

                        Ok(())
                    })?,
            );
        }

        while !threads.is_empty() {
            let thread = threads.pop().expect("Thread list should not be empty");
            if thread.is_finished() {
                match thread.join() {
                    Ok(Ok(())) => continue, // Thread completed successfully
                    Ok(Err(e)) => {
                        killed.store(true, std::sync::atomic::Ordering::Relaxed);
                        return Err(e);
                    }
                    Err(e) => {
                        killed.store(true, std::sync::atomic::Ordering::Relaxed);
                        return Err(anyhow::anyhow!("Thread panicked: {:?}", e));
                    }
                }
            } else {
                threads.push(thread);
            }

            std::thread::yield_now(); // Yield to allow other threads to run
        }

        if killed.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Output was killed"));
        }
        Ok(())
    }
}

register_output_plugin!(FfmpegOutputPlugin);
