use aviutl2::{output::OutputPlugin, register_output_plugin};
use std::io::Write;
use std::sync::Arc;

struct FfmpegOutputPlugin {}

fn tcp_server_for_callback<T: Fn(std::net::TcpStream) -> anyhow::Result<()> + Send + 'static>(
    callback: T,
) -> (
    std::net::SocketAddr,
    std::thread::JoinHandle<anyhow::Result<()>>,
) {
    let server = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind server");
    let local_addr = server.local_addr().expect("Failed to get local address");
    let server_thread = std::thread::spawn(move || {
        let stream = server.incoming().next();
        match stream {
            Some(Ok(stream)) => {
                println!("Accepted connection from {}", stream.peer_addr().unwrap());
                let ret = callback(stream.try_clone().expect("Failed to clone stream"));
                stream
                    .shutdown(std::net::Shutdown::Both)
                    .expect("Failed to close stream");
                ret
            }
            Some(Err(e)) => Err(anyhow::anyhow!("Failed to accept connection: {}", e)),
            None => Ok(()), // No incoming connections
        }
    });
    (local_addr, server_thread)
}

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
        let mut threads: Vec<std::thread::JoinHandle<anyhow::Result<()>>> = Vec::new();

        let (video_input, video_tx) = match &info.video {
            Some(video) => {
                anyhow::ensure!(video.num_frames > 0, "空の動画は出力できません。");
                let buf_size = video.fps.to_integer() as usize;
                let (tx, rx) = std::sync::mpsc::sync_channel::<(u8, u8, u8)>(buf_size);

                // TODO: ez_ffmpegのnew_by_read_callbackを使ってデータを読み込む。
                // （TCPサーバーは普通に回りくどいしアンチウイルスに引っかかる可能性があるので）
                let (local_addr, server_thread) = tcp_server_for_callback({
                    let killed = Arc::clone(&killed);
                    move |mut stream: std::net::TcpStream| -> anyhow::Result<()> {
                        let mut buf = [0u8; 3];
                        while !killed.load(std::sync::atomic::Ordering::Relaxed)
                            && let Ok(read) = rx.recv()
                        {
                            buf[0] = read.0;
                            buf[1] = read.1;
                            buf[2] = read.2;
                            stream.write_all(&buf)?;
                        }
                        stream.flush()?;
                        Ok(())
                    }
                });
                threads.push(server_thread);

                (
                    Some(
                        ez_ffmpeg::Input::new(format!("tcp://{}", local_addr))
                            .set_format("rawvideo")
                            .set_video_codec("rawvideo")
                            .set_input_opt("pixel_format", "rgb24")
                            .set_input_opt(
                                "video_size",
                                &format!("{}x{}", video.width, video.height),
                            )
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
                let (local_addr, server_thread) = tcp_server_for_callback({
                    let killed = Arc::clone(&killed);
                    move |mut stream: std::net::TcpStream| -> anyhow::Result<()> {
                        let mut buf = [0u8; 8]; // 2 f32 values, each 4 bytes
                        while !killed.load(std::sync::atomic::Ordering::Relaxed)
                            && let Ok(read) = rx.recv()
                        {
                            buf[0..4].copy_from_slice(&read.0.to_le_bytes());
                            buf[4..8].copy_from_slice(&read.1.to_le_bytes());
                            stream.write_all(&buf)?;
                        }
                        stream.flush()?;
                        Ok(())
                    }
                });
                threads.push(server_thread);

                (
                    Some(
                        ez_ffmpeg::Input::new(format!("tcp://{}", local_addr))
                            .set_format("f32le")
                            .set_audio_codec("pcm_f32le")
                            .set_input_opt("sample_rate", &format!("{}", audio.sample_rate))
                            .set_input_opt("ch_layout", "stereo"),
                    ),
                    Some(tx),
                )
            }
            None => (None, None),
        };

        threads.push(
            std::thread::Builder::new()
                .name("aviutl2_ffmpeg_output".to_string())
                .spawn({
                    let killed = Arc::clone(&killed);
                    move || -> anyhow::Result<()> {
                        let f = match (video_input, audio_input) {
                            (Some(video), Some(audio)) => ez_ffmpeg::FfmpegContext::builder()
                                .inputs(vec![video, audio])
                                .output(output.add_stream_map("0:v").add_stream_map("1:a"))
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
                        let killed = Arc::clone(&killed);
                        move || -> anyhow::Result<()> {
                            for (_i, frames) in info.get_video_frames_iter() {
                                for frame in frames {
                                    tx.send(frame).expect("Failed to send video frame");
                                }
                                if killed.load(std::sync::atomic::Ordering::Relaxed) {
                                    return Err(anyhow::anyhow!("Output was killed"));
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
                    .spawn({
                        let killed = Arc::clone(&killed);
                        move || -> anyhow::Result<()> {
                            for (_i, samples) in
                                info.get_stereo_audio_samples_iter((sample_rate / 10) as i32)
                            {
                                for sample in samples {
                                    tx.send(sample).expect("Failed to send audio sample");
                                }
                                if killed.load(std::sync::atomic::Ordering::Relaxed) {
                                    return Err(anyhow::anyhow!("Output was killed"));
                                }
                            }

                            Ok(())
                        }
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
