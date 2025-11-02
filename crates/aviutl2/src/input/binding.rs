use std::borrow::Cow;

use crate::common::{AnyResult, AviUtl2Info, FileFilter, Rational32, Win32WindowHandle, Yc48, f16};
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
    ///
    /// <div class="warning">
    ///
    /// このフラグによって、呼ばれるトレイトのメソッドが変わります。
    /// `true` の場合は [`InputPlugin::read_video`] と [`InputPlugin::read_audio`] が呼ばれ、
    /// `false` の場合は [`InputPlugin::read_video_mut`] と [`InputPlugin::read_audio_mut`] が呼ばれます。
    ///
    /// </div>
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

    /// 動画のフレームを手動で算出するかどうか。
    ///
    /// # See Also
    /// [`InputPlugin::time_to_frame`]
    pub manual_frame_index: bool,

    /// 画像の幅。
    pub width: u32,
    /// 画像の高さ。
    pub height: u32,

    /// 画像のフォーマット。
    pub format: InputPixelFormat,
}

/// 画像のフォーマット。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputPixelFormat {
    /// RGB形式。
    /// `(u8, u8, u8)`相当。
    ///
    /// <div class="warning">
    ///
    /// この形式では、左下から右上に向かって色が並びます。
    ///
    /// </div>
    ///
    /// # See Also
    /// [`crate::utils::rgb_to_bgr`]
    Bgr,
    /// BGRA形式。
    /// `(u8, u8, u8, u8)`相当。
    ///
    /// <div class="warning">
    ///
    /// この形式では、左下から右上に向かって色が並びます。
    ///
    /// </div>
    ///
    /// # See Also
    /// [`crate::utils::rgba_to_bgra`]
    Bgra,
    /// YUV 4:2:2形式。
    /// `(u8, u8, u8, u8)`相当。
    Yuy2,
    /// DXGI_FORMAT_R16G16B16A16_UNORM（乗算済みα）形式。
    /// `(u16, u16, u16, u16)`相当。
    Pa64,
    /// YC48（互換対応の旧内部フォーマット）形式。
    /// `(i16, i16, i16)`相当。
    ///
    /// # See Also
    /// [`Yc48`]
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

impl std::ops::Deref for ImageBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 画像データを `ImageBuffer` に変換するトレイト。
pub trait IntoImage {
    fn into_image(self) -> ImageBuffer;
}

impl<T: AsImage> IntoImage for T {
    fn into_image(self) -> ImageBuffer {
        ImageBuffer(self.as_image().into_owned())
    }
}

/// 画像データを `Cow<[u8]>` に変換するトレイト。
pub trait AsImage {
    fn as_image(&'_ self) -> Cow<'_, [u8]>;
}

impl AsImage for ImageBuffer {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
}

impl AsImage for Vec<u8> {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}

impl AsImage for &[u8] {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}

impl AsImage for Cow<'_, [u8]> {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        match self {
            Cow::Borrowed(b) => Cow::Borrowed(b),
            Cow::Owned(b) => Cow::Borrowed(b),
        }
    }
}

duplicate::duplicate! {
    [
        Name            Trait     method;
        [ImageReturner] [AsImage] [as_image];
        [AudioReturner] [AsAudio] [as_audio];
    ]
    /// AviUtl2側にバイト列を返すためのstruct。
    pub struct Name {
        ptr: *mut u8,
        pub(crate) written: usize,
    }

    impl Name {
        /// # Safety
        ///
        /// AviUtl2側から渡されるポインタのみが許容される。
        pub(crate) unsafe fn new(ptr: *mut u8) -> Self {
            Self { ptr, written: 0 }
        }

        pub fn write(&mut self, data: &impl Trait) {
            let image = data.method();
            unsafe {
                std::ptr::copy_nonoverlapping(image.as_ptr(), self.ptr.add(self.written), image.len());
            }
            self.written += image.len();
        }
    }
}

#[duplicate::duplicate_item(
    T;
    [Vec<u16>];
    [Vec<i16>];
    [Vec<f16>];
    [Vec<Yc48>];
)]
impl AsImage for T {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}

#[cfg(feature = "image")]
impl AsImage for image::RgbImage {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        let mut data = self.as_raw().to_owned();
        crate::utils::bgr_to_rgb_bytes(&mut data);
        crate::utils::flip_vertical(&mut data, self.width() as usize * 3, self.height() as usize);
        Cow::Owned(data)
    }
}

#[cfg(feature = "image")]
impl AsImage for image::RgbaImage {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        let mut data = self.as_raw().to_owned();
        crate::utils::bgra_to_rgba_bytes(&mut data);
        crate::utils::flip_vertical(&mut data, self.width() as usize * 4, self.height() as usize);
        Cow::Owned(data)
    }
}

#[cfg(feature = "image")]
impl AsImage for image::ImageBuffer<image::Rgb<u16>, Vec<u16>> {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        let data = self.as_raw();
        Cow::Owned(data.as_bytes().to_vec())
    }
}

#[cfg(feature = "image")]
impl AsImage for image::ImageBuffer<image::Rgba<u16>, Vec<u16>> {
    fn as_image(&'_ self) -> Cow<'_, [u8]> {
        let data = self.as_raw();
        Cow::Owned(data.as_bytes().to_vec())
    }
}

macro_rules! as_image_impl_for_tuple {
    ($type:ty, $($name:ident),+) => {
        impl AsImage for Vec<$type> {
            fn as_image(&'_ self) -> Cow<'_, [u8]> {
                let mut img_data = Vec::with_capacity(self.len() * std::mem::size_of::<$type>());
                for ($($name,)+) in self {
                    $(img_data.extend_from_slice(&$name.to_le_bytes());)+
                }
                Cow::Owned(img_data)
            }
        }
    };
}

as_image_impl_for_tuple!((u8, u8, u8), r, g, b);
as_image_impl_for_tuple!((u8, u8, u8, u8), r, g, b, a);
as_image_impl_for_tuple!((u16, u16, u16, u16), r, g, b, a);
as_image_impl_for_tuple!((f16, f16, f16, f16), r, g, b, a);
as_image_impl_for_tuple!((i16, i16, i16), y, cb, cr);

/// 音声のバッファを表す構造体。
#[derive(Debug, Clone)]
pub struct AudioBuffer(pub Vec<u8>);

impl std::ops::Deref for AudioBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 音声データを `AudioBuffer` に変換するトレイト。
pub trait IntoAudio {
    fn into_audio(self) -> AudioBuffer;
}

impl<T: AsAudio> IntoAudio for T {
    fn into_audio(self) -> AudioBuffer {
        AudioBuffer(self.as_audio().into_owned())
    }
}

/// 音声データを `Cow<[u8]>` に変換するトレイト。
pub trait AsAudio {
    fn as_audio(&'_ self) -> Cow<'_, [u8]>;
}

impl AsAudio for AudioBuffer {
    fn as_audio(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
}
impl AsAudio for Vec<u8> {
    fn as_audio(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self)
    }
}
#[duplicate::duplicate_item(
    T;
    [Vec<u16>];
    [Vec<f32>];
)]
impl AsAudio for T {
    fn as_audio(&'_ self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.as_bytes())
    }
}

macro_rules! into_audio_impl_for_tuple {
    ($type:ty, $($name:ident),+) => {
        impl AsAudio for Vec<$type> {
            fn as_audio(&'_ self) -> Cow<'_, [u8]> {
                let mut audio_data = Vec::with_capacity(self.len() * std::mem::size_of::<$type>());
                for ($($name,)+) in self {
                    $(audio_data.extend_from_slice(&$name.to_le_bytes());)+
                }
                Cow::Owned(audio_data)
            }
        }
    };
}
into_audio_impl_for_tuple!((u16, u16), l, r);
into_audio_impl_for_tuple!((f32, f32), l, r);

/// 入力プラグインのトレイト。
/// このトレイトを実装し、[`crate::register_input_plugin!`] マクロを使用してプラグインを登録します。
pub trait InputPlugin: Send + Sync + Sized {
    /// 入力ハンドルの型。
    type InputHandle: std::any::Any + Send + Sync;

    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> InputPluginTable;

    /// 入力を開く。
    fn open(&self, file: std::path::PathBuf) -> AnyResult<Self::InputHandle>;
    /// 入力を閉じる。
    fn close(&self, handle: Self::InputHandle) -> AnyResult<()>;

    /// 動画・音声のトラック数を取得する。
    fn get_track_count(&self, handle: &mut Self::InputHandle) -> AnyResult<(u32, u32)> {
        let info = self.get_input_info(handle, 0, 0)?;
        let video_tracks = info.video.as_ref().map_or(0, |_| 1);
        let audio_tracks = info.audio.as_ref().map_or(0, |_| 1);
        Ok((video_tracks, audio_tracks))
    }

    /// 入力の情報を取得する。
    fn get_input_info(
        &self,
        handle: &mut Self::InputHandle,
        video_track: u32,
        audio_track: u32,
    ) -> AnyResult<InputInfo>;

    /// 動画・画像を読み込む。
    ///
    /// <div class="warning">
    ///
    /// [`InputPluginTable::concurrent`] が `true` の場合に呼ばれます。
    /// `false` の場合は [`Self::read_video_mut`] が呼ばれます。
    ///
    /// </div>
    fn read_video(
        &self,
        handle: &Self::InputHandle,
        frame: u32,
        returner: &mut ImageReturner,
    ) -> AnyResult<()> {
        let _ = (handle, frame, returner);
        Result::<(), anyhow::Error>::Err(anyhow::anyhow!(
            "read_video is not implemented for this plugin"
        ))
    }

    /// 動画・画像を読み込む。
    ///
    /// <div class="warning">
    ///
    /// [`InputPluginTable::concurrent`] が `false` の場合に呼ばれます。
    /// `true` の場合は [`Self::read_video`] が呼ばれます。
    ///
    /// </div>
    fn read_video_mut(
        &self,
        handle: &mut Self::InputHandle,
        frame: u32,
        returner: &mut ImageReturner,
    ) -> AnyResult<()> {
        self.read_video(handle, frame, returner)
    }

    /// 動画のトラックが利用可能かどうかを確認する。
    ///
    /// # Returns
    /// トラック番号を返します。基本的には `track` をそのまま返します。
    /// これがErrを返した場合、トラックの変更が失敗したものとして扱われます。
    fn can_set_video_track(&self, _handle: &mut Self::InputHandle, track: u32) -> AnyResult<u32> {
        Ok(track)
    }

    // TODO: これが他の関数と同時に呼ばれるかどうかは未検証なので、検証する（handleが `&mut` でいいかどうかに影響するため）
    /// 現在の時刻からフレーム数を取得する。
    /// [`VideoInputInfo::manual_frame_index`] が `true` の場合に使用されます。
    fn time_to_frame(
        &self,
        handle: &mut Self::InputHandle,
        track: u32,
        time: f64,
    ) -> AnyResult<u32> {
        const RESOLUTION: i32 = 1000; // ミリ秒単位での解像度
        let info = self.get_input_info(handle, track, 0)?;
        if let Some(video_info) = &info.video {
            Ok(
                (video_info.fps * Rational32::new((time * RESOLUTION as f64) as i32, RESOLUTION))
                    .to_integer() as u32,
            )
        } else {
            Err(anyhow::anyhow!("No video information available"))
        }
    }

    /// 音声を読み込む。
    ///
    /// <div class="warning">
    ///
    /// [`InputPluginTable::concurrent`] が `true` の場合に呼ばれます。
    /// `false` の場合は [`Self::read_audio_mut`] が呼ばれます。
    ///
    /// </div>
    fn read_audio(
        &self,
        handle: &Self::InputHandle,
        start: i32,
        length: i32,
        returner: &mut AudioReturner,
    ) -> AnyResult<()> {
        let _ = (handle, start, length, returner);
        Result::<(), anyhow::Error>::Err(anyhow::anyhow!(
            "read_audio is not implemented for this plugin"
        ))
    }

    /// 音声を読み込む。
    ///
    /// <div class="warning">
    ///
    /// [`InputPluginTable::concurrent`] が `false` の場合に呼ばれます。
    /// `true` の場合は [`Self::read_audio`] が呼ばれます。
    ///
    /// </div>
    fn read_audio_mut(
        &self,
        handle: &mut Self::InputHandle,
        start: i32,
        length: i32,
        returner: &mut AudioReturner,
    ) -> AnyResult<()> {
        self.read_audio(handle, start, length, returner)
    }

    /// 音声のトラックが利用可能かどうかを確認する。
    ///
    /// # Returns
    /// トラック番号を返します。基本的には `track` をそのまま返します。
    /// これがErrを返した場合、トラックの変更が失敗したものとして扱われます。
    fn can_set_audio_track(&self, _handle: &mut Self::InputHandle, track: u32) -> AnyResult<u32> {
        Ok(track)
    }

    /// 設定ダイアログを表示する。
    fn config(&self, _hwnd: Win32WindowHandle) -> AnyResult<()> {
        Ok(())
    }

    /// シングルトンインスタンスを参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance<R>(f: impl FnOnce(&Self) -> R) -> R
    where
        Self: crate::input::__bridge::InputSingleton,
    {
        <Self as crate::input::__bridge::InputSingleton>::with_instance(f)
    }

    /// シングルトンインスタンスを可変参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance_mut<R>(f: impl FnOnce(&mut Self) -> R) -> R
    where
        Self: crate::input::__bridge::InputSingleton,
    {
        <Self as crate::input::__bridge::InputSingleton>::with_instance_mut(f)
    }
}
