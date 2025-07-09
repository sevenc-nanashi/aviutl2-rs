use crate::sys::input2::{HINSTANCE, HWND};

pub use anyhow::Result as AnyResult;

use super::FileFilter;

pub struct InputPluginTable {
    pub name: String,
    pub input_type: InputType,
    pub file_filters: Vec<FileFilter>,
    pub information: String,

    pub can_config: bool,
}

pub struct VideoInputInfo {
    pub fps: i32,
    pub scale: i32,
    pub num_frames: i32,
    pub image_format: ImageFormat,
}

pub struct AudioInputInfo {
    pub num_samples: i32,
    pub audio_format: AudioFormat,
}

pub struct AudioFormat {
    pub channels: u16,
    pub sample_rate: u32,
}

pub struct InputInfo {
    pub video: Option<VideoInputInfo>,
    pub audio: Option<AudioInputInfo>,

    pub concurrent: bool,
}

pub enum InputType {
    Video,
    Audio,
    Both,
    BothConcurrent,
}

impl InputType {
    pub fn from_bits(bits: i32) -> Self {
        match bits {
            1 => InputType::Video,
            2 => InputType::Audio,
            3 => InputType::Both,
            16 => InputType::BothConcurrent,
            _ => panic!("Unknown InputInfoFlag bits: {}", bits),
        }
    }

    pub fn to_bits(&self) -> i32 {
        match self {
            InputType::Video => 1,
            InputType::Audio => 2,
            InputType::Both => 3,
            InputType::BothConcurrent => 16,
        }
    }
}

pub struct ImageFormat {
    pub width: u32,
    pub height: u32,
}

pub struct ImageBuffer(pub Vec<u8>);

pub trait IntoImage {
    fn into_image(self) -> ImageBuffer;
}

impl IntoImage for ImageBuffer {
    fn into_image(self) -> ImageBuffer {
        self
    }
}

impl IntoImage for Vec<u8> {
    /// `Vec<u8>`から [`ImageBuffer`] へ変換します。
    fn into_image(self) -> ImageBuffer {
        debug_assert!(
            self.len() % 4 == 0,
            "Image data length must be a multiple of 4"
        );
        ImageBuffer(self)
    }
}

impl IntoImage for Vec<(u8, u8, u8)> {
    fn into_image(self) -> ImageBuffer {
        let mut image_data = Vec::with_capacity(self.len() * 4);
        for (r, g, b) in self {
            image_data.push(r);
            image_data.push(g);
            image_data.push(b);
            image_data.push(255);
        }
        ImageBuffer(image_data)
    }
}

impl IntoImage for Vec<(u8, u8, u8, u8)> {
    fn into_image(self) -> ImageBuffer {
        let mut image_data = Vec::with_capacity(self.len() * 4);
        for (r, g, b, a) in self {
            image_data.push(r);
            image_data.push(g);
            image_data.push(b);
            image_data.push(a);
        }
        ImageBuffer(image_data)
    }
}

pub struct AudioBuffer(pub Vec<f32>);

pub trait IntoAudio {
    fn into_audio(self) -> AudioBuffer;
}
impl IntoAudio for AudioBuffer {
    fn into_audio(self) -> AudioBuffer {
        self
    }
}

impl IntoAudio for Vec<f32> {
    fn into_audio(self) -> AudioBuffer {
        // Assuming the Vec<f32> is already in a suitable format
        AudioBuffer(self)
    }
}
impl IntoAudio for Vec<i16> {
    fn into_audio(self) -> AudioBuffer {
        let audio_data = self
            .into_iter()
            .map(|s| s as f32 / i16::MAX as f32)
            .collect();
        AudioBuffer(audio_data)
    }
}

impl IntoAudio for Vec<(f32, f32)> {
    fn into_audio(self) -> AudioBuffer {
        let audio_data = self.into_iter().flat_map(|(l, r)| [l, r]).collect();
        AudioBuffer(audio_data)
    }
}

impl IntoAudio for Vec<(i16, i16)> {
    fn into_audio(self) -> AudioBuffer {
        let audio_data = self
            .into_iter()
            .flat_map(|(l, r)| [l as f32 / i16::MAX as f32, r as f32 / i16::MAX as f32])
            .collect();
        AudioBuffer(audio_data)
    }
}

pub trait InputPlugin: Send + Sync {
    type InputHandle: std::any::Any + Send + Sync;

    fn new() -> Self;

    fn plugin_info(&self) -> InputPluginTable;

    fn open(&self, file: std::path::PathBuf) -> Option<Self::InputHandle>;
    fn close(&self, handle: Self::InputHandle) -> AnyResult<()>;

    fn get_input_info(&self, handle: &Self::InputHandle) -> AnyResult<InputInfo>;
    fn read_video(&self, _handle: &Self::InputHandle, _frame: i32) -> AnyResult<impl IntoImage> {
        Ok(ImageBuffer(vec![])) // Default implementation, can be overridden
    }
    fn read_audio(
        &self,
        _handle: &Self::InputHandle,
        _start: i32,
        _length: i32,
    ) -> AnyResult<impl IntoAudio> {
        Ok(AudioBuffer(vec![])) // Default implementation, can be overridden
    }

    fn config(&self, _hwnd: HWND, _dll_hinst: HINSTANCE) -> AnyResult<()> {
        anyhow::bail!("This plugin does not support configuration");
    }
}
