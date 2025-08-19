use crate::common::{AnyResult, FileFilter};
pub use half::f16;
pub use num_rational::Rational32;
pub use raw_window_handle::Win32WindowHandle;
use zerocopy::IntoBytes;

/// 入力プラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct InputPluginTable {
    /// プラグインの名前。
    pub name: String,
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,

    /// 入力の種類。
    pub input_type: InputType,
    /// 音声・動画の同時取得が可能かどうか。
    pub concurrent: bool,
    /// プラグインがサポートするファイルフィルタのリスト。
    pub file_filters: Vec<FileFilter>,

    /// プラグインが設定可能かどうか。
    pub can_config: bool,
}

/// 動画・画像の入力情報を表す構造体。
#[derive(Debug, Clone)]
pub struct VideoInputInfo {
    /// 動画のフレームレート。
    pub fps: Rational32,
    /// 動画のフレーム数。
    /// 画像の場合は1フレームとしてください。
    ///
    /// # Safety
    /// 内部ではi32に変換されます。
    pub num_frames: u32,

    /// 画像の幅。
    pub width: u32,
    /// 画像の高さ。
    pub height: u32,

    /// 画像のフォーマット。
    pub format: ImageFormat,

    /// トラック数。
    pub num_tracks: u32,
}

/// 画像のフォーマット。
#[derive(Debug, Clone)]
pub enum ImageFormat {
    /// RGB形式。
    /// `(u8, u8, u8)`相当。
    Rgb,
    /// RGBA形式。
    /// `(u8, u8, u8, u8)`相当。
    Rgba,
    /// YUV 4:2:2形式。
    /// `(u8, u8, u8, u8)`相当。
    Yuy2,
    /// DXGI_FORMAT_R16G16B16A16_UNORM（乗算済みα）形式。
    /// `(u16, u16, u16, u16)`相当。
    Pa64,
    /// YC48（互換対応の旧内部フォーマット）形式。
    /// `(u16, i16, i16)`相当。
    Yc48,
    /// DXGI_FORMAT_R16G16B16A16_FLOAT（乗算済みα）形式。
    /// `(f16, f16, f16, f16)`相当。
    Hf64,
}

/// 音声の入力情報を表す構造体。
#[derive(Debug, Clone)]
pub struct AudioInputInfo {
    /// 音声のサンプルレート。
    pub sample_rate: u32,
    /// 音声のサンプル数。
    pub num_samples: u32,
    /// 音声のチャンネル数。
    pub channels: u16,

    /// トラック数。
    pub num_tracks: u32,

    /// 音声のフォーマット。
    pub format: AudioFormat,
}

/// 音声のフォーマットを表す列挙型。
#[derive(Debug, Clone)]
pub enum AudioFormat {
    /// PCM 16bit形式。
    Pcm16,
    /// IEEE Float 32bit形式。
    IeeeFloat32,
}

/// 動画・画像と音声の入力情報をまとめた構造体。
#[derive(Debug, Clone)]
pub struct InputInfo {
    /// 動画・画像のフォーマット。
    pub video: Option<VideoInputInfo>,
    /// 音声のフォーマット。
    pub audio: Option<AudioInputInfo>,
}

/// 入力の種類を表す列挙型。
#[derive(Debug, Clone)]
pub enum InputType {
    /// 動画のみ。
    Video,
    /// 音声のみ。
    Audio,
    /// 動画と音声の両方。
    Both,
}

impl InputType {
    pub(crate) fn to_bits(&self) -> i32 {
        match self {
            InputType::Video => 1,
            InputType::Audio => 2,
            InputType::Both => 3,
        }
    }
}

/// 画像のバッファを表す構造体。
#[derive(Debug, Clone)]
pub struct ImageBuffer(pub Vec<u8>);

/// 画像データを [`ImageBuffer`] に変換するトレイト。
pub trait IntoImage {
    fn into_image(self) -> ImageBuffer;
}

impl IntoImage for ImageBuffer {
    fn into_image(self) -> ImageBuffer {
        self
    }
}

impl IntoImage for Vec<u8> {
    fn into_image(self) -> ImageBuffer {
        ImageBuffer(self)
    }
}

#[duplicate::duplicate_item(
    T;
    [Vec<u16>];
    [Vec<i16>];
    [Vec<f16>];
)]
impl IntoImage for T {
    fn into_image(self) -> ImageBuffer {
        let image_data = self.as_bytes().to_vec();
        ImageBuffer(image_data)
    }
}

macro_rules! into_image_impl_for_tuple {
    ($type:ty, $($name:ident),+) => {
        impl IntoImage for Vec<$type> {
            fn into_image(self) -> ImageBuffer {
                let mut image_data = Vec::with_capacity(self.len() * std::mem::size_of::<$type>());
                for ($($name,)+) in self {
                    $(image_data.extend_from_slice(&$name.to_le_bytes());)+
                }
                ImageBuffer(image_data)
            }
        }
    };
}

into_image_impl_for_tuple!((u8, u8, u8), r, g, b);
into_image_impl_for_tuple!((u8, u8, u8, u8), r, g, b, a);
into_image_impl_for_tuple!((u16, u16, u16, u16), r, g, b, a);
into_image_impl_for_tuple!((f16, f16, f16, f16), r, g, b, a);
into_image_impl_for_tuple!((u16, i16, i16), y, u, v);

#[derive(Debug, Clone)]
pub struct AudioBuffer(pub Vec<u8>);

pub trait IntoAudio {
    fn into_audio(self) -> AudioBuffer;
}
impl IntoAudio for AudioBuffer {
    fn into_audio(self) -> AudioBuffer {
        self
    }
}
impl IntoAudio for Vec<u8> {
    fn into_audio(self) -> AudioBuffer {
        AudioBuffer(self)
    }
}
#[duplicate::duplicate_item(
    T;
    [Vec<u16>];
    [Vec<f32>];
)]
impl IntoAudio for T {
    fn into_audio(self) -> AudioBuffer {
        let audio_data = self.as_bytes().to_vec();
        AudioBuffer(audio_data)
    }
}

/// 入力プラグインのトレイト。
/// このトレイトを実装し、[`register_input_plugin!`] マクロを使用してプラグインを登録します。
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
