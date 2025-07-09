use crate::sys::input2::{HINSTANCE, HWND};

pub use anyhow::Result as AnyResult;
use aviutl2_sys::output2::OUTPUT_INFO;
pub use num_rational::Rational32;

use super::FileFilter;

pub struct OutputPluginTable {
    pub name: String,
    pub input_type: OutputType,
    pub file_filters: Vec<FileFilter>,
    pub information: String,

    pub can_config: bool,
}

pub enum OutputType {
    Video,
    Audio,
    Both,
}

impl OutputType {
    pub fn to_bits(&self) -> i32 {
        match self {
            OutputType::Video => 1,
            OutputType::Audio => 2,
            OutputType::Both => 3,
        }
    }
}

pub struct OutputInfo {
    pub video: Option<VideoOutputInfo>,
    pub audio: Option<AudioOutputInfo>,
    pub path: std::path::PathBuf,

    internal: OUTPUT_INFO,
}

pub struct VideoOutputInfo {
    pub width: u32,
    pub height: u32,
    pub fps: Rational32,
    pub num_frames: u32,
}

pub struct AudioOutputInfo {
    pub sample_rate: u32,
    pub num_samples: u32,
    pub num_channels: u32,
}

pub trait OutputPlugin: Send + Sync {
    type OutputHandle: std::any::Any + Send + Sync;

    fn new() -> Self;

    fn plugin_info(&self) -> OutputPluginTable;

    fn output(&self, info: OutputInfo) -> Option<Self::OutputHandle>;

    fn config(&self, _hwnd: HWND, _dll_hinst: HINSTANCE) -> AnyResult<()> {
        anyhow::bail!("This plugin does not support configuration");
    }

    fn config_text(&self) -> AnyResult<String> {
        anyhow::bail!("This plugin does not support configuration");
    }
}
