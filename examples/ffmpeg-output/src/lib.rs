mod config;
mod dialog;
use crate::{
    config::{FfmpegOutputConfig, load_config, save_config},
    dialog::FfmpegOutputConfigDialog,
};
use anyhow::Context;
use aviutl2::{output::OutputPlugin, register_output_plugin};
use eframe::egui;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

struct FfmpegOutputPlugin {
    config: Mutex<FfmpegOutputConfig>,
}

pub static DEFAULT_ARGS: &[&str] = &[
    "-y",
    "-f",
    "rawvideo",
    "-pix_fmt",
    "rgb24",
    "-video_size",
    "{video_size}",
    "-framerate",
    "{video_fps}",
    "-i",
    "{video_source}",
    "-f",
    "f32le",
    "-ar",
    "{audio_sample_rate}",
    "-ac",
    "2",
    "-i",
    "{audio_source}",
    "-map",
    "0:v:0",
    "-map",
    "1:a:0",
    "{output_path}",
];
pub static REQUIRED_ARGS: &[&str] = &[
    "{video_source}",
    "{video_size}",
    "{video_fps}",
    "{audio_source}",
    "{audio_sample_rate}",
    "{output_path}",
];

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
        let config = match load_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load FFmpeg output plugin config: {}", e);
                FfmpegOutputConfig {
                    args: DEFAULT_ARGS.iter().map(|s| s.to_string()).collect(),
                }
            }
        };
        FfmpegOutputPlugin {
            config: Mutex::new(config),
        }
    }

    fn plugin_info(&self) -> aviutl2::output::OutputPluginTable {
        aviutl2::output::OutputPluginTable {
            name: "Rusty FFmpeg Output Plugin".to_string(),
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

        let buf_size = info
            .video
            .as_ref()
            .map_or(1, |v| v.fps.to_integer() as usize);
        let (video_tx, video_rx) = std::sync::mpsc::sync_channel::<Vec<(u8, u8, u8)>>(buf_size);

        let (video_local_addr, video_server_thread) = tcp_server_for_callback({
            let killed = Arc::clone(&killed);
            move |stream: std::net::TcpStream| -> anyhow::Result<()> {
                let mut writer = std::io::BufWriter::new(stream);
                let mut buf = [0u8; 3];
                while !killed.load(std::sync::atomic::Ordering::Relaxed)
                    && let Ok(read) = video_rx.recv()
                {
                    for pixel in &read {
                        buf[0] = pixel.0;
                        buf[1] = pixel.1;
                        buf[2] = pixel.2;
                        writer.write(&buf)?;
                    }
                    writer.flush()?;
                }
                writer.flush()?;
                Ok(())
            }
        })?;
        threads.push(video_server_thread);

        let (audio_tx, audio_rx) = std::sync::mpsc::sync_channel::<Vec<(f32, f32)>>(
            info.audio
                .as_ref()
                .map_or(1, |audio| audio.sample_rate as usize / 10),
        );
        let (audio_local_addr, audio_server_thread) = tcp_server_for_callback({
            let killed = Arc::clone(&killed);
            move |stream: std::net::TcpStream| -> anyhow::Result<()> {
                let mut buf = [0u8; 8]; // 2 f32 values, each 4 bytes
                let mut writer = std::io::BufWriter::new(stream);
                while !killed.load(std::sync::atomic::Ordering::Relaxed)
                    && let Ok(read) = audio_rx.recv()
                {
                    for sample in &read {
                        buf[0..4].copy_from_slice(&sample.0.to_le_bytes());
                        buf[4..8].copy_from_slice(&sample.1.to_le_bytes());
                        writer.write(&buf)?;
                    }
                    writer.flush()?;
                }
                writer.flush()?;
                Ok(())
            }
        })?;
        threads.push(audio_server_thread);

        let sample_rate = info.audio.as_ref().map_or(0, |a| a.sample_rate);
        let info = Arc::new(info);

        if info.video.is_some() {
            threads.push(
                std::thread::Builder::new()
                    .name("aviutl2_ffmpeg_video_output".to_string())
                    .spawn({
                        let info = Arc::clone(&info);
                        let killed = Arc::clone(&killed);
                        move || -> anyhow::Result<()> {
                            for (_i, frames) in info.get_video_frames_iter() {
                                video_tx.send(frames).expect("Failed to send video frame");
                                if killed.load(std::sync::atomic::Ordering::Relaxed) {
                                    return Err(anyhow::anyhow!("Output was killed"));
                                }
                            }

                            Ok(())
                        }
                    })?,
            );
        } else {
            drop(video_tx);
        }

        if info.audio.is_some() {
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
                                audio_tx.send(samples).expect("Failed to send audio sample");
                                if killed.load(std::sync::atomic::Ordering::Relaxed) {
                                    return Err(anyhow::anyhow!("Output was killed"));
                                }
                            }

                            Ok(())
                        }
                    })?,
            );
        } else {
            drop(audio_tx);
        }

        assert!(
            info.video.is_some() || info.audio.is_some(),
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
        let config_args = self
            .config
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock FFmpeg Output Plugin config: {}", e))?
            .args
            .clone();
        for arg in config_args {
            args.push(
                arg.replace("{video_source}", &format!("tcp://{}", video_local_addr))
                    .replace(
                        "{video_size}",
                        &format!(
                            "{}x{}",
                            info.video.as_ref().map_or(0, |v| v.width),
                            info.video.as_ref().map_or(0, |v| v.height)
                        ),
                    )
                    .replace(
                        "{video_fps}",
                        &info
                            .video
                            .as_ref()
                            .map_or("30".to_string(), |v| v.fps.to_string()),
                    )
                    .replace("{audio_source}", &format!("tcp://{}", audio_local_addr))
                    .replace(
                        "{audio_sample_rate}",
                        &info
                            .audio
                            .as_ref()
                            .map_or("44100".to_string(), |a| a.sample_rate.to_string()),
                    )
                    .replace("{output_path}", &info.path.to_string_lossy().to_string()),
            );
        }

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

    fn config(&self, _handle: aviutl2::output::Win32WindowHandle) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = std::sync::mpsc::channel();
        // TODO: eframeで親ウィンドウを指定できるようになったらそうする
        eframe::run_native(
            "Rusty FFmpeg Output Plugin",
            Default::default(),
            Box::new(|cc| {
                if !egui::FontDefinitions::default()
                    .font_data
                    .contains_key("M+ 1")
                {
                    let mut fonts = egui::FontDefinitions::default();
                    fonts.font_data.insert(
                        "M+ 1".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                            "../fonts/fonts/otf/Mplus1-Regular.otf"
                        ))),
                    );
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "M+ 1".to_owned());

                    fonts.font_data.insert(
                        "M+ 1 Code".to_owned(),
                        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                            "../fonts/fonts/otf/Mplus1Code-Medium.otf"
                        ))),
                    );
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .insert(0, "M+ 1 Code".to_owned());

                    cc.egui_ctx.set_fonts(fonts);
                }
                Ok(Box::new(FfmpegOutputConfigDialog::new(
                    self.config
                        .lock()
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to lock FFmpeg Output Plugin config: {}", e)
                        })?
                        .clone(),
                    result_sender,
                )))
            }),
        )
        .map_err(|e| anyhow::anyhow!("Failed to run FFmpeg Output Plugin configuration: {}", e))?;

        if let Ok(new_config) = result_receiver.try_recv() {
            save_config(&new_config).map_err(|e| {
                anyhow::anyhow!("Failed to save FFmpeg Output Plugin config: {}", e)
            })?;
            self.config
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock FFmpeg Output Plugin config: {}", e))?
                .args = new_config.args;
        }
        return Ok(());
    }

    fn config_text(&self) -> anyhow::Result<String> {
        let config = self
            .config
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock FFmpeg Output Plugin config: {}", e))?;
        let args = if config.args == DEFAULT_ARGS {
            "デフォルト"
        } else {
            "カスタム"
        };
        Ok(format!("引数：{args}"))
    }
}

register_output_plugin!(FfmpegOutputPlugin);
