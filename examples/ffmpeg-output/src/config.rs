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

pub type FfmpegOutputConfig = FfmpegOutputConfigV1;

pub fn config_path() -> anyhow::Result<std::path::PathBuf> {
    let data_dir = get_data_dir()?;
    let config_path = data_dir.join("args.json");
    Ok(config_path)
}

pub fn load_config() -> anyhow::Result<FfmpegOutputConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(FfmpegOutputConfig {
            args: DEFAULT_ARGS.iter().map(|s| s.to_string()).collect(),
        });
    }
    let file =
        std::fs::File::open(&path).context("Failed to open FFmpeg output plugin config file")?;
    let config: FfmpegOutputConfigContainer = serde_json::from_reader(file)
        .context("Failed to parse FFmpeg output plugin config file")?;
    match config.version {
        1 => {
            let v1: FfmpegOutputConfigV1 = serde_json::from_value(config.value)
                .context("Failed to parse FFmpeg output plugin config version 1")?;
            Ok(v1)
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported FFmpeg output plugin config version: {}",
            config.version
        )),
    }
}

pub fn save_config(config: &FfmpegOutputConfig) -> anyhow::Result<()> {
    let path = config_path()?;
    let container = FfmpegOutputConfigContainer {
        version: 1,
        value: serde_json::to_value(config)
            .context("Failed to serialize FFmpeg output plugin config")?,
    };
    let file = std::fs::File::create(&path)
        .context("Failed to create FFmpeg output plugin config file")?;
    serde_json::to_writer(file, &container)
        .context("Failed to write FFmpeg output plugin config file")?;
    Ok(())
}
