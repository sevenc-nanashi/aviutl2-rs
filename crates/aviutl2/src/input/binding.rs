// pub struct InputPluginTable {
//     pub name: String,
//     pub filefilter: Vec<InputFilter>,
//     pub information: String,
//
//     pub can_config: bool,
// }
//
use crate::sys::input2::{HWND, HINSTANCE};

pub struct InputFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

pub struct InputInfo {
    pub flag: InputType,
    pub fps: i32,
    pub scale: i32,
    pub num_frames: i32,
    pub num_samples: i32,
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
    fn into_image(self) -> anyhow::Result<ImageBuffer>;
}

impl IntoImage for Vec<u8> {
    fn into_image(self) -> anyhow::Result<ImageBuffer> {
        // Assuming the Vec<u8> is already in a suitable format
        Ok(ImageBuffer(self))
    }
}

impl IntoImage for Vec<(u8, u8, u8)> {
    fn into_image(self) -> anyhow::Result<ImageBuffer> {
        let mut image_data = Vec::with_capacity(self.len() * 3);
        for (r, g, b) in self {
            image_data.push(r);
            image_data.push(g);
            image_data.push(b);
            image_data.push(255);
        }
        Ok(ImageBuffer(image_data))
    }
}

impl IntoImage for Vec<(u8, u8, u8, u8)> {
    fn into_image(self) -> anyhow::Result<ImageBuffer> {
        let mut image_data = Vec::with_capacity(self.len() * 4);
        for (r, g, b, a) in self {
            image_data.push(r);
            image_data.push(g);
            image_data.push(b);
            image_data.push(a);
        }
        Ok(ImageBuffer(image_data))
    }
}

pub struct AudioBuffer(pub Vec<f32>);

trait IntoAudio {
    fn into_audio(self) -> anyhow::Result<AudioBuffer>;
}
impl IntoAudio for Vec<f32> {
    fn into_audio(self) -> anyhow::Result<AudioBuffer> {
        // Assuming the Vec<f32> is already in a suitable format
        Ok(AudioBuffer(self))
    }
}
impl IntoAudio for Vec<i16> {
    fn into_audio(self) -> anyhow::Result<AudioBuffer> {
        let audio_data = self
            .into_iter()
            .map(|s| s as f32 / i16::MAX as f32)
            .collect();
        Ok(AudioBuffer(audio_data))
    }
}

pub trait InputPlugin: Send + Sync {
    type InputHandle: std::any::Any + Send + Sync;
    const PLUGIN_NAME: &'static str;
    const PLUGIN_FILE_FILTER: &'static [InputFilter];
    const PLUGIN_INFORMATION: &'static str;
    const PLUGIN_TYPE: InputType;

    const PLUGIN_CONFIG: bool = false;

    fn new() -> Self;

    // fn info(&self) -> InputPluginTable;

    fn open(&self, file: std::path::PathBuf) -> Option<Self::InputHandle>;
    fn close(&self, handle: &mut Self::InputHandle) -> bool;

    fn get_info(&self, handle: &Self::InputHandle) -> anyhow::Result<InputInfo>;
    fn read_video(&self, handle: &Self::InputHandle, frame: i32) -> anyhow::Result<ImageBuffer> {
        Ok(ImageBuffer(vec![])) // Default implementation, can be overridden
    }
    fn read_audio(
        &self,
        handle: &Self::InputHandle,
        start: i32,
        length: i32,
    ) -> anyhow::Result<AudioBuffer> {
        Ok(AudioBuffer(vec![])) // Default implementation, can be overridden
    }

    fn config(&self, hwnd: Option<HWND>, dll_hinst: Option<HINSTANCE>) -> bool {
        false
    }
}
