use crate::common::{AnyResult, FileFilter};
use aviutl2_sys::output2::OUTPUT_INFO;
use std::ops::Deref;

pub use half::f16;
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

    fn check(video: &VideoOutputInfo) -> Result<(), String>;
    /// # Safety
    /// func_get_videoの戻り値のポインタのみが許容される。
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self;
}

duplicate::duplicate! {
    [
        Name                Type;
        [RgbVideoFrame]     [(u8, u8, u8)];
        [Yuy2VideoFrame]    [(u8, u8, u8, u8)];
        [Hf64VideoFrame]    [(f16, f16, f16, f16)];
        [Yc48VideoFrame]    [(u16, u16, u16)];
        [Pa64VideoFrame]    [(u16, u16, u16, u16)];

        [RawBgrVideoFrame]  [u8];
        [RawYuy2VideoFrame] [u8];
        [RawHf64VideoFrame] [u16];
        [RawYc48VideoFrame] [u16];
        [RawPa64VideoFrame] [u16];
    ]
    #[derive(Debug, Clone)]
    pub struct Name {
        pub data: Vec<Type>, // Raw RGB format
    }
    impl Deref for Name {
        type Target = [Type];

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }
}

pub trait FromRawAudioSamples: Sized + Send + Sync + Copy {
    const FORMAT: u32;

    /// # Safety
    /// func_get_audioの戻り値のポインタのみが許容される。
    unsafe fn from_raw(length: i32, num_channels: u32, audio_data_ptr: *const u8) -> Vec<Self>;
}
