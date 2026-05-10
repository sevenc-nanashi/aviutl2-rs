use crate::DEFAULT_ARGS;
use anyhow::Context;

const CONFIG_VERSION: u64 = 3;
const PROJECT_CONFIG_KEY: &str = "config";

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
            PixelFormat::Yuy2 => "YUV422（YUY2、透過なし）",
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

impl TryFrom<FfmpegOutputConfigContainer> for FfmpegOutputConfig {
    type Error = anyhow::Error;

    fn try_from(container: FfmpegOutputConfigContainer) -> Result<Self, Self::Error> {
        match container.version {
            1 => {
                let config: FfmpegOutputConfigV1 = serde_json::from_value(container.value)
                    .context("Failed to parse FFmpeg output plugin config v1")?;
                Ok(Self {
                    args: config.args,
                    pixel_format: PixelFormat::Bgr24,
                })
            }
            2 => {
                let config: FfmpegOutputConfigV2 = serde_json::from_value(container.value)
                    .context("Failed to parse FFmpeg output plugin config v2")?;
                Ok(Self {
                    args: config.args,
                    pixel_format: config.pixel_format,
                })
            }
            3 => serde_json::from_value(container.value)
                .context("Failed to parse FFmpeg output plugin config v3"),
            version => Err(anyhow::anyhow!(
                "Unsupported FFmpeg output plugin config version: {}",
                version
            )),
        }
    }
}

pub fn load_project_config(
    project: &aviutl2::generic::ProjectFile<'_>,
) -> anyhow::Result<FfmpegOutputConfig> {
    match project.deserialize::<FfmpegOutputConfigContainer>(PROJECT_CONFIG_KEY) {
        Ok(container) => container.try_into(),
        Err(container_error) => project
            .deserialize::<FfmpegOutputConfig>(PROJECT_CONFIG_KEY)
            .with_context(|| {
                format!(
                    "Failed to load FFmpeg output plugin config from project file: {container_error}"
                )
            }),
    }
}

pub fn save_project_config(
    project: &mut aviutl2::generic::ProjectFile<'_>,
    config: &FfmpegOutputConfig,
) -> anyhow::Result<()> {
    let container = FfmpegOutputConfigContainer {
        version: CONFIG_VERSION,
        value: serde_json::to_value(config)
            .context("Failed to serialize FFmpeg output plugin config")?,
    };
    project
        .serialize(PROJECT_CONFIG_KEY, &container)
        .context("Failed to save FFmpeg output plugin config to project file")?;
    Ok(())
}
