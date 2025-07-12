use anyhow::Context;
use aviutl2::{output::OutputPlugin, register_output_plugin};
use std::io::Write;
use std::sync::Arc;

struct FfmpegOutputPlugin {}

fn tcp_server_for_callback<T: Fn(std::net::TcpStream) -> anyhow::Result<()> + Send + 'static>(
    callback: T,
) -> anyhow::Result<(
    std::net::SocketAddr,
    std::thread::JoinHandle<anyhow::Result<()>>,
)> {
    let server = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind server");
    let local_addr = server.local_addr().expect("Failed to get local address");
    let server_thread = std::thread::Builder::new()
        .name("aviutl2_ffmpeg_output_tcp_server".to_string())
        .spawn(move || {
            let stream = server.incoming().next();
            match stream {
                Some(Ok(stream)) => {
                    let ret = callback(stream.try_clone()?);
                    stream
                        .shutdown(std::net::Shutdown::Both)
                        .expect("Failed to close stream");
                    ret
                }
                Some(Err(e)) => Err(anyhow::anyhow!("Failed to accept connection: {}", e)),
                None => Ok(()), // No incoming connections
            }
        })?;
    Ok((local_addr, server_thread))
}

fn get_data_dir() -> anyhow::Result<std::path::PathBuf> {
    let dll_path = process_path::get_dylib_path()
        .ok_or_else(|| anyhow::anyhow!("failed to get the directory of the dll"))?;
    let path = dll_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to get the parent directory of the dll"))?
        .join("rusty_ffmpeg");
    std::fs::create_dir_all(&path).context("Failed to create data directory")?;
    Ok(path)
}

fn get_ffmpeg_dir() -> anyhow::Result<std::path::PathBuf> {
    let data_dir = get_data_dir()?;
    let path = data_dir.join("ffmpeg");
    Ok(path)
}

fn download_ffmpeg_if_missing() -> anyhow::Result<std::path::PathBuf> {
    let data_dir = get_data_dir()?;
    let ffmpeg_dir = get_ffmpeg_dir()?;
    if ffmpeg_dir.exists() {
        return Ok(ffmpeg_dir);
    }
    let ffmpeg_zip_path = data_dir.join("ffmpeg.zip");
    let ffmpeg_tmp_zip_path = data_dir.join("ffmpeg.tmp.zip");
    let ffmpeg_tmp_dir = data_dir.join("ffmpeg.tmp");

    if !ffmpeg_zip_path.exists() {
        let url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-win64-lgpl-shared-7.1.zip";
        let response = ureq::get(url)
            .config()
            .max_redirects(8)
            .build()
            .call()
            .context("Failed to download FFmpeg")?;
        let mut file = std::fs::File::create(&ffmpeg_tmp_zip_path)
            .context("Failed to create FFmpeg zip file")?;
        if response.status() != 200 {
            return Err(anyhow::anyhow!(
                "Failed to download FFmpeg: HTTP status {}",
                response.status()
            ));
        }
        std::io::copy(&mut response.into_body().into_reader(), &mut file)
            .context("Failed to write FFmpeg zip file")?;
        std::fs::rename(&ffmpeg_tmp_zip_path, &ffmpeg_zip_path)
            .context("Failed to rename FFmpeg zip file")?;
    }

    let ffmpeg_zip =
        std::fs::File::open(&ffmpeg_zip_path).context("Failed to open FFmpeg zip file")?;
    zip_extract::extract(&ffmpeg_zip, &ffmpeg_tmp_dir, true)?;
    std::fs::remove_file(&ffmpeg_zip_path).context("Failed to remove FFmpeg zip file")?;
    std::fs::rename(&ffmpeg_tmp_dir, &ffmpeg_dir)
        .context("Failed to move extracted FFmpeg directory")?;

    Ok(ffmpeg_dir)
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
                })?;
                threads.push(server_thread);

                (Some(local_addr), Some(tx))
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
                })?;
                threads.push(server_thread);

                (Some(local_addr), Some(tx))
            }
            None => (None, None),
        };

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
                        let info = Arc::clone(&info);
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

        assert!(
            video_input.is_some() || audio_input.is_some(),
            "At least one of video_input or audio_input must be provided"
        );

        let ffmpeg_dir = download_ffmpeg_if_missing()
            .map_err(|e| anyhow::anyhow!("Failed to download FFmpeg: {}", e))?;
        let ffmpeg_path = ffmpeg_dir.join("bin").join("ffmpeg.exe");
        if !ffmpeg_path.exists() {
            return Err(anyhow::anyhow!(
                "FFmpeg executable not found at {:?}",
                ffmpeg_path
            ));
        }
        let mut args = vec![];
        args.push("-y".to_string()); // Overwrite output files without asking
        if let Some(video_input) = video_input {
            args.push("-f".to_string());
            args.push("rawvideo".to_string());
            args.push("-pix_fmt".to_string());
            args.push("rgb24".to_string());
            args.push("-video_size".to_string());
            args.push(format!(
                "{}x{}",
                info.video.as_ref().unwrap().width,
                info.video.as_ref().unwrap().height
            ));
            args.push("-framerate".to_string());
            args.push(info.video.as_ref().unwrap().fps.to_string());
            args.push("-i".to_string());
            args.push(format!("tcp://{}", video_input));
        } else {
            args.push("-f".to_string());
            args.push("null".to_string());
            args.push("-".to_string());
        }
        if let Some(audio_input) = audio_input {
            args.push("-f".to_string());
            args.push("f32le".to_string());
            args.push("-ar".to_string());
            args.push(info.audio.as_ref().unwrap().sample_rate.to_string());
            args.push("-ac".to_string());
            args.push("2".to_string());
            args.push("-i".to_string());
            args.push(format!("tcp://{}", audio_input));
        } else {
            args.push("-f".to_string());
            args.push("null".to_string());
            args.push("-".to_string());
        }
        args.push("-map".to_string());
        args.push("0:v:0".to_string());
        args.push("-map".to_string());
        args.push("1:a:0".to_string());
        args.push(info.path.to_string_lossy().to_string());

        threads.push(
            std::thread::Builder::new()
                .name("aviutl2_ffmpeg_process".to_string())
                .spawn({
                    let killed = Arc::clone(&killed);
                    move || -> anyhow::Result<()> {
                        let mut child = std::process::Command::new(ffmpeg_path)
                            .args(&args)
                            .spawn()
                            .map_err(|e| {
                                anyhow::anyhow!("Failed to start FFmpeg process: {}", e)
                            })?;

                        while !killed.load(std::sync::atomic::Ordering::Relaxed) {
                            std::thread::yield_now();
                            if child.try_wait().is_ok() {
                                break; // FFmpeg process has exited
                            }
                        }
                        let status = child.wait().map_err(|e| {
                            anyhow::anyhow!("Failed to wait for FFmpeg process: {}", e)
                        })?;
                        if !status.success() {
                            return Err(anyhow::anyhow!(
                                "FFmpeg process exited with non-zero status: {}",
                                status
                            ));
                        }
                        Ok(())
                    }
                })?,
        );

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
