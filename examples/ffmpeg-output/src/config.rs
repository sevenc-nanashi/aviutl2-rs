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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FfmpegOutputConfigV3 {
    pub args: Vec<String>,
    pub pixel_format: PixelFormat,
}
impl Default for FfmpegOutputConfigV3 {
    fn default() -> Self {
        Self {
            args: DEFAULT_ARGS.iter().map(|s| s.to_string()).collect(),
            pixel_format: PixelFormat::Bgr24,
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum PixelFormat {
    Yuy2,
    Bgr24,
    Pa64,
    Hf64,
}
impl PixelFormat {
    pub fn as_str(&self) -> &str {
        match self {
            PixelFormat::Yuy2 => "YUY2（YUV422、透過なし）",
            PixelFormat::Bgr24 => "BGR u8x3（BGR24、透過なし）",
            PixelFormat::Pa64 => "RGBA u16x4（PA64、透過対応）",
            PixelFormat::Hf64 => "RGBA f16x4（HF64、透過対応）",
        }
    }

    pub fn as_ffmpeg_str(&self) -> &str {
        match self {
            PixelFormat::Yuy2 => "yuyv422",
            PixelFormat::Bgr24 => "bgr24",
            PixelFormat::Pa64 => "rgba64le",
            PixelFormat::Hf64 => "rgbaf16le",
        }
    }
}

pub type FfmpegOutputConfig = FfmpegOutputConfigV3;
const CONFIG_VERSION: u64 = 3;

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
    if config.version <= 2 {
        config.version = 3;
        config.value = serde_json::to_value(FfmpegOutputConfigV3::default())?;
    }

    serde_json::from_value(config.value).context("Failed to parse FFmpeg output plugin config")
}

pub fn save_config(config: &FfmpegOutputConfig) -> anyhow::Result<()> {
    let path = config_path()?;
    let container = FfmpegOutputConfigContainer {
        version: CONFIG_VERSION,
        value: serde_json::to_value(config)
            .context("Failed to serialize FFmpeg output plugin config")?,
    };
    let file = std::fs::File::create(&path)
        .context("Failed to create FFmpeg output plugin config file")?;
    serde_json::to_writer(file, &container)
        .context("Failed to write FFmpeg output plugin config file")?;
    Ok(())
}
