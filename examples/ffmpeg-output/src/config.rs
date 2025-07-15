use crate::{DEFAULT_ARGS, get_data_dir};
use anyhow::Context;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FfmpegOutputConfigContainer {
    version: u64,
    value: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FfmpegOutputConfigV1 {
    pub args: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FfmpegOutputConfigV2 {
    pub args: Vec<String>,
    pub pixel_format: PixelFormat,
}
impl Default for FfmpegOutputConfigV2 {
    fn default() -> Self {
        FfmpegOutputConfigV2 {
            args: DEFAULT_ARGS.iter().map(|s| s.to_string()).collect(),
            pixel_format: PixelFormat::Bgr24,
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum PixelFormat {
    Rgb24,
    Yuy2,
    Bgr24,
}
impl PixelFormat {
    pub fn to_str(&self) -> &str {
        match self {
            PixelFormat::Rgb24 => "RGB24",
            PixelFormat::Yuy2 => "YUY2",
            PixelFormat::Bgr24 => "BGR24",
        }
    }

    pub fn to_ffmpeg_str(&self) -> &str {
        match self {
            PixelFormat::Rgb24 => "rgb24",
            PixelFormat::Yuy2 => "yuyv422",
            PixelFormat::Bgr24 => "bgr24",
        }
    }
}

pub type FfmpegOutputConfig = FfmpegOutputConfigV2;

pub fn config_path() -> anyhow::Result<std::path::PathBuf> {
    let data_dir = get_data_dir()?;
    let config_path = data_dir.join("args.json");
    Ok(config_path)
}

pub fn load_config() -> anyhow::Result<FfmpegOutputConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Default::default());
    }
    let file =
        std::fs::File::open(&path).context("Failed to open FFmpeg output plugin config file")?;
    let mut config: FfmpegOutputConfigContainer = serde_json::from_reader(file)
        .context("Failed to parse FFmpeg output plugin config file")?;
    if config.version == 1 {
        let v1: FfmpegOutputConfigV1 = serde_json::from_value(config.value)
            .context("Failed to parse FFmpeg output plugin config version 1")?;
        config.value = serde_json::to_value(FfmpegOutputConfigV2 {
            args: v1
                .args
                .iter()
                .map(|s| {
                    if s == "rgb24" {
                        "{video_pixel_format}".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect(),
            pixel_format: PixelFormat::Rgb24,
        })?;
    }

    Ok(serde_json::from_value(config.value)
        .context("Failed to parse FFmpeg output plugin config")?)
}

pub fn save_config(config: &FfmpegOutputConfig) -> anyhow::Result<()> {
    let path = config_path()?;
    let container = FfmpegOutputConfigContainer {
        version: 2,
        value: serde_json::to_value(config)
            .context("Failed to serialize FFmpeg output plugin config")?,
    };
    let file = std::fs::File::create(&path)
        .context("Failed to create FFmpeg output plugin config file")?;
    serde_json::to_writer(file, &container)
        .context("Failed to write FFmpeg output plugin config file")?;
    Ok(())
}
