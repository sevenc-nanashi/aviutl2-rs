use crate::common::{AnyResult, FileFilter};
use aviutl2_sys::output2::OUTPUT_INFO;
use std::ops::Deref;

pub use num_rational::Rational32;
pub use raw_window_handle::Win32WindowHandle;

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

    pub(crate) internal: *mut OUTPUT_INFO,
}

unsafe impl Send for OutputInfo {}
unsafe impl Sync for OutputInfo {}

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
    fn new() -> Self;

    fn plugin_info(&self) -> OutputPluginTable;

    fn output(&self, info: OutputInfo) -> AnyResult<()>;

    fn config(&self, _hwnd: Win32WindowHandle) -> AnyResult<()> {
        Ok(())
    }

    fn config_text(&self) -> AnyResult<String> {
        Ok(String::new())
    }
}

pub trait FromRawVideoFrame {
    const FORMAT: u32;
    /// # Safety
    /// func_get_videoの戻り値のポインタのみが許容される。
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self;
}

pub struct RgbVideoFrame {
    pub data: Vec<(u8, u8, u8)>, // RGB format
}
impl Deref for RgbVideoFrame {
    type Target = [(u8, u8, u8)];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct Yuy2VideoFrame {
    pub data: Vec<(u8, u8)>, // YUY2 format
}

impl Deref for Yuy2VideoFrame {
    type Target = [(u8, u8)];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct RawBgrVideoFrame {
    pub data: Vec<u8>, // Raw RGB format
}
impl Deref for RawBgrVideoFrame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct RawYuy2VideoFrame {
    pub data: Vec<u8>, // Raw YUY2 format
}
impl Deref for RawYuy2VideoFrame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
