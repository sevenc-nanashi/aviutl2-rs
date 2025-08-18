use crate::common::{AnyResult, FileFilter};
pub use raw_window_handle::Win32WindowHandle;

pub struct InputPluginTable {
    pub name: String,
    pub input_type: InputType,
    pub concurrent: bool,
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
    // TODO:
    // pub dynamic_time: bool,
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
            _ => panic!("Unknown InputInfoFlag bits: {bits}"),
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
    fn into_image(self, width: u32, height: u32) -> ImageBuffer;
}

impl IntoImage for ImageBuffer {
    fn into_image(self, _width: u32, _height: u32) -> ImageBuffer {
        self
    }
}

impl IntoImage for Vec<u8> {
    /// `Vec<u8>`から [`ImageBuffer`] へ変換します。
    /// > [!NOTE]
    /// > BGRA、下から上に並んだピクセルデータを想定しています。
    fn into_image(self, width: u32, height: u32) -> ImageBuffer {
        debug_assert!(
            self.len() == (width * height * 4) as usize,
            "Image data length does not match the specified width and height."
        );
        ImageBuffer(self)
    }
}

impl IntoImage for Vec<(u8, u8, u8)> {
    /// `Vec<(u8, u8, u8)>`から [`ImageBuffer`] へ変換します。
    /// > [!NOTE]
    /// > RGB、上から下に並んだピクセルデータを想定しています。
    fn into_image(self, width: u32, height: u32) -> ImageBuffer {
        let mut new_image_data = Vec::with_capacity(self.len() * 4);
        let new_image_data_writer = new_image_data.spare_capacity_mut();
        for x in 0..width {
            for y in 0..height {
                let (r, g, b) = self[(y * width + x) as usize];
                let dest_idx = (((height - 1 - y) * width + x) as usize) * 4;
                new_image_data_writer[dest_idx].write(b);
                new_image_data_writer[dest_idx + 1].write(g);
                new_image_data_writer[dest_idx + 2].write(r);
                new_image_data_writer[dest_idx + 3].write(255);
            }
        }
        unsafe { new_image_data.set_len(width as usize * height as usize * 4) };
        ImageBuffer(new_image_data)
    }
}

impl IntoImage for Vec<(u8, u8, u8, u8)> {
    /// `Vec<(u8, u8, u8, u8)>`から [`ImageBuffer`] へ変換します。
    /// > [!NOTE]
    /// > RGBA、上から下に並んだピクセルデータを想定しています。
    fn into_image(self, width: u32, height: u32) -> ImageBuffer {
        let mut new_image_data = Vec::with_capacity(self.len() * 4);
        let new_image_data_writer = new_image_data.spare_capacity_mut();
        for x in 0..width {
            for y in 0..height {
                let (r, g, b, a) = self[(y * width + x) as usize];
                let dest_idx = (((height - 1 - y) * width + x) as usize) * 4;
                new_image_data_writer[dest_idx].write(b);
                new_image_data_writer[dest_idx + 1].write(g);
                new_image_data_writer[dest_idx + 2].write(r);
                new_image_data_writer[dest_idx + 3].write(a);
            }
        }
        unsafe { new_image_data.set_len(width as usize * height as usize * 4) };
        ImageBuffer(new_image_data)
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

    fn open(&self, file: std::path::PathBuf) -> AnyResult<Self::InputHandle>;
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

    fn config(&self, _hwnd: Win32WindowHandle) -> AnyResult<()> {
        Ok(())
    }
}
