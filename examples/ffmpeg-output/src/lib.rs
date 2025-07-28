mod config;
mod dialog;
mod named_pipe;
use crate::{
    config::{FfmpegOutputConfig, load_config, save_config},
    dialog::FfmpegOutputConfigDialog,
    named_pipe::{NamedPipe, PipeWriter},
};
use anyhow::Context;
use aviutl2::{
    output::{
        OutputPlugin, RawBgrVideoFrame, RawHf64VideoFrame, RawPa64VideoFrame, RawYuy2VideoFrame,
    },
    register_output_plugin,
};
use eframe::egui;
use std::{
    io::{Read, Write},
    os::windows::process::CommandExt,
    sync::{Arc, Mutex},
};

fn create_send_only_named_pipe(name: &str) -> anyhow::Result<(String, NamedPipe)> {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let pipe_name = format!(r"\\.\pipe\{name}-{nonce}");
    let pipe =
        NamedPipe::new(&pipe_name).context("Failed to create named pipe for FFmpeg output")?;
    Ok((pipe_name, pipe))
}

struct FfmpegOutputPlugin {
    config: Mutex<FfmpegOutputConfig>,
}

pub static DEFAULT_ARGS: &[&str] = &[
    "-y",
    "-f",
    "rawvideo",
    "-pix_fmt",
    "{video_pixel_format}",
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
    "-vf",
    "{maybe_vflip}",
    "{output_path}",
];
pub static REQUIRED_ARGS: &[&str] = &[
    "{video_source}",
    "{video_pixel_format}",
    "{video_size}",
    "{video_fps}",
    "{audio_source}",
    "{audio_sample_rate}",
    "{output_path}",
    "{maybe_vflip}",
];

fn pipe_for_callback<T: Fn(PipeWriter) -> anyhow::Result<()> + Send + 'static>(
    name: &str,
    callback: T,
) -> anyhow::Result<(String, std::thread::JoinHandle<anyhow::Result<()>>)> {
    let (pipe_name, pipe) = create_send_only_named_pipe(name)
        .context("Failed to create named pipe for FFmpeg output")?;
    let server_thread = std::thread::Builder::new()
        .name(format!("aviutl2_ffmpeg_pipe_server_{name}"))
        .spawn(move || {
            callback(
                pipe.connect()
                    .context("Failed to connect named pipe for FFmpeg output")?,
            )
        })?;
    Ok((pipe_name, server_thread))
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

fn get_log_dir() -> anyhow::Result<std::path::PathBuf> {
    let data_dir = get_data_dir()?;
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).context("Failed to create log directory")?;
    Ok(log_dir)
}

fn get_log_writer() -> anyhow::Result<std::fs::File> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let log_file_path = get_log_dir()?.join(format!("ffmpeg_output_{timestamp}.log"));
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .context("Failed to open FFmpeg output log file")?;
    Ok(file)
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
                eprintln!("Failed to load FFmpeg output plugin config: {e}");
                FfmpegOutputConfig::default()
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
                extensions: vec![
                    "mp4".to_string(), "mkv".to_string(), "avi".to_string(), "webm".to_string(),
                    "mov".to_string(), "flv".to_string(), "ts".to_string(), "m4v".to_string()
                ],
            }],
            information: "FFmpeg for AviUtl, written in Rust / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/ffmpeg-output".to_string(),
            can_config: true,
        }
    }

    fn output(&self, info: aviutl2::output::OutputInfo) -> aviutl2::AnyResult<()> {
        let killed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut threads: Vec<std::thread::JoinHandle<anyhow::Result<()>>> = Vec::new();
        let info = Arc::new(info);
        let config = self
            .config
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock FFmpeg Output Plugin config: {}", e))?
            .clone();

        if info.video.as_ref().is_some_and(|v| {
            (v.width % 2 != 0 || v.height % 2 != 0)
                && config.pixel_format == config::PixelFormat::Yuy2
        }) {
            return Err(anyhow::anyhow!(
                "YUY2モードでは偶数以外の解像度はサポートされていません。現在の解像度は {}x{} です。",
                info.video.as_ref().map_or(0, |v| v.width),
                info.video.as_ref().map_or(0, |v| v.height)
            ));
        }

        let (video_path, video_server_thread) = pipe_for_callback("aviutl2_ffmpeg_video_pipe", {
            let info = Arc::clone(&info);
            move |stream: PipeWriter| -> anyhow::Result<()> {
                if info.video.is_none() {
                    return Ok(());
                }
                let mut writer = std::io::BufWriter::new(stream);
                match config.pixel_format {
                    config::PixelFormat::Yuy2 => {
                        for (_, frame) in info.get_video_frames_iter::<RawYuy2VideoFrame>() {
                            writer.write_all(&frame.data)?;
                        }
                    }
                    config::PixelFormat::Bgr24 => {
                        for (_, frame) in info.get_video_frames_iter::<RawBgrVideoFrame>() {
                            writer.write_all(&frame.data)?;
                        }
                    }
                    config::PixelFormat::Pa64 => {
                        for (_, frame) in info.get_video_frames_iter::<RawPa64VideoFrame>() {
                            writer.write_all(unsafe {
                                std::slice::from_raw_parts(
                                    frame.data.as_ptr() as *const u8,
                                    frame.data.len() * std::mem::size_of::<u16>(),
                                )
                            })?;
                        }
                    }
                    config::PixelFormat::Hf64 => {
                        for (_, frame) in info.get_video_frames_iter::<RawHf64VideoFrame>() {
                            writer.write_all(unsafe {
                                std::slice::from_raw_parts(
                                    frame.data.as_ptr() as *const u8,
                                    frame.data.len() * std::mem::size_of::<u16>(),
                                )
                            })?;
                        }
                    }
                }
                writer.flush()?;
                Ok(())
            }
        })?;
        threads.push(video_server_thread);

        let (audio_path, audio_server_thread) = pipe_for_callback("aviutl2_ffmpeg_audio_pipe", {
            let info = Arc::clone(&info);
            move |stream: PipeWriter| -> anyhow::Result<()> {
                if info.audio.is_none() {
                    return Ok(());
                }
                let mut buf = [0u8; 8]; // 2 f32 values, each 4 bytes
                let mut writer = std::io::BufWriter::new(stream);
                for (_, samples) in info.get_stereo_audio_samples_iter::<f32>(
                    (info.audio.as_ref().map_or(44100, |a| a.sample_rate) / 10) as i32,
                ) {
                    for sample in &samples {
                        buf[0..4].copy_from_slice(&sample.0.to_le_bytes());
                        buf[4..8].copy_from_slice(&sample.1.to_le_bytes());
                        writer.write_all(&buf)?;
                    }
                    writer.flush()?;
                }
                writer.flush()?;
                Ok(())
            }
        })?;
        threads.push(audio_server_thread);

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
                arg.replace("{video_source}", &video_path)
                    .replace("{video_pixel_format}", config.pixel_format.as_ffmpeg_str())
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
                    .replace("{audio_source}", &audio_path)
                    .replace(
                        "{audio_sample_rate}",
                        &info
                            .audio
                            .as_ref()
                            .map_or("44100".to_string(), |a| a.sample_rate.to_string()),
                    )
                    .replace(
                        "{maybe_vflip}",
                        if config.pixel_format == config::PixelFormat::Bgr24 {
                            "vflip"
                        } else {
                            "null"
                        },
                    )
                    .replace("{output_path}", info.path.to_string_lossy().as_ref()),
            );
        }

        threads.push(
            std::thread::Builder::new()
                .name("aviutl2_ffmpeg_process".to_string())
                .spawn({
                    let killed = Arc::clone(&killed);
                    move || ffmpeg_thread(ffmpeg_path, args, killed)
                })?,
        );

        while let Some(thread) = threads.pop() {
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
            let mut config = self.config.lock().map_err(|e| {
                anyhow::anyhow!("Failed to lock FFmpeg Output Plugin config: {}", e)
            })?;
            config.pixel_format = new_config.pixel_format;
            config.args = new_config.args;
        }
        Ok(())
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
        let pixel_format = config.pixel_format.as_str();
        Ok(format!(
            "引数：{args} | ピクセルフォーマット：{pixel_format}"
        ))
    }
}

fn ffmpeg_thread(
    ffmpeg_path: std::path::PathBuf,
    args: Vec<String>,
    killed: Arc<std::sync::atomic::AtomicBool>,
) -> anyhow::Result<()> {
    let mut writer = get_log_writer()?;
    writeln!(writer, "FFmpeg path: {ffmpeg_path:?}",)?;
    writeln!(writer, "Starting FFmpeg with args: {args:?}",)?;
    let mut child = std::process::Command::new(ffmpeg_path)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start FFmpeg process: {}", e))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get FFmpeg stdout"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to get FFmpeg stderr"))?;
    let writer = Arc::new(std::sync::Mutex::new(writer));
    let stdout_pipe_thread = std::thread::Builder::new()
        .name("aviutl2_ffmpeg_stdout_pipe".to_string())
        .spawn({
            let writer = Arc::clone(&writer);
            let killed = Arc::clone(&killed);
            move || -> anyhow::Result<()> { pipe_thread(&killed, &mut stdout, writer) }
        })?;
    let stderr_pipe_thread = std::thread::Builder::new()
        .name("aviutl2_ffmpeg_stderr_pipe".to_string())
        .spawn({
            let writer = Arc::clone(&writer);
            let killed = Arc::clone(&killed);
            move || -> anyhow::Result<()> { pipe_thread(&killed, &mut stderr, writer) }
        })?;
    while !killed.load(std::sync::atomic::Ordering::Relaxed) && child.try_wait().is_err() {
        std::thread::yield_now();
    }
    let status = child
        .wait()
        .map_err(|e| anyhow::anyhow!("Failed to wait for FFmpeg process: {}", e))?;
    stdout_pipe_thread
        .join()
        .map_err(|e| anyhow::anyhow!("FFmpeg stdout pipe thread panicked: {:?}", e))??;
    stderr_pipe_thread
        .join()
        .map_err(|e| anyhow::anyhow!("FFmpeg stderr pipe thread panicked: {:?}", e))??;
    writeln!(
        writer.lock().unwrap(),
        "FFmpeg process exited with status: {status}",
    )?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "FFmpeg process exited with non-zero status: {}",
            status
        ));
    }
    drop(writer);

    Ok(())
}

fn pipe_thread<F: Read + Send + 'static>(
    killed: &Arc<std::sync::atomic::AtomicBool>,
    stdout: &mut F,
    writer: Arc<Mutex<std::fs::File>>,
) -> Result<(), anyhow::Error> {
    let mut buf = [0u8; 1024];
    while !killed.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::yield_now();
        match stdout.read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                if let Err(e) = writer.lock().unwrap().write_all(&buf[..n]) {
                    eprintln!("Failed to write FFmpeg stdout: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to read FFmpeg stdout: {e}");
                break;
            }
        }
    }
    Ok(())
}

register_output_plugin!(FfmpegOutputPlugin);
