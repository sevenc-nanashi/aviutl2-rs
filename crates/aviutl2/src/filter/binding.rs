use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use super::config;
use crate::common::{AnyResult, AviUtl2Info, Rational32};

/// 入力プラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct FilterPluginTable {
    /// プラグインの名前。
    pub name: String,
    /// ラベルの初期値。
    /// Noneの場合、デフォルトのラベルになります
    pub label: Option<String>,
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,

    /// 入力の種類。
    pub filter_type: FilterType,

    /// カスタムオブジェクトにするかどうか。
    /// `true` の場合、カスタムオブジェクトとして動作します。
    /// `false` の場合、フィルタ効果として動作します。
    pub as_object: bool,

    /// 設定項目。
    pub config_items: Vec<config::FilterConfigItem>,
}
/// 動画・画像と音声の入力情報をまとめた構造体。
/// 入力の種類を表す列挙型。
#[derive(Debug, Clone)]
pub enum FilterType {
    /// 動画のみ。
    Video,
    /// 音声のみ。
    Audio,
    /// 動画と音声の両方。
    Both,
}

impl FilterType {
    pub(crate) fn to_bits(&self) -> i32 {
        match self {
            FilterType::Video => aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_VIDEO,
            FilterType::Audio => aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_AUDIO,
            FilterType::Both => {
                aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_VIDEO
                    | aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_AUDIO
            }
        }
    }
}

/// フィルタプラグインのトレイト。
/// このトレイトを実装し、[`crate::register_filter_plugin!`] マクロを使用してプラグインを登録します。
pub trait FilterPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> FilterPluginTable;

    /// 画像フィルタ処理関数。
    fn proc_video(
        &self,
        _config: &[config::FilterConfigItem],
        _video: &FilterProcVideo,
    ) -> AnyResult<()> {
        anyhow::bail!("proc_video is not implemented");
    }

    /// 音声フィルタ処理関数。
    fn proc_audio(
        &self,
        _config: &[config::FilterConfigItem],
        _audio: &FilterProcAudio,
    ) -> AnyResult<()> {
        anyhow::bail!("proc_audio is not implemented");
    }
}

/// シーン情報。
#[derive(Debug, Clone, Copy)]
pub struct SceneInfo {
    /// 解像度（幅）。
    pub width: u32,
    /// 解像度（高さ）。
    pub height: u32,
    /// フレームレート。
    pub frame_rate: Rational32,
    /// サンプリングレート。
    pub sample_rate: u32,
}

/// オブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct ObjectInfo {
    /// ID。
    pub id: i64,
    /// オブジェクトの現在のフレーム番号。
    pub frame: u32,
    /// オブジェクトの総フレーム数。
    pub frame_total: u32,
    /// オブジェクトの現在の時間（秒）。
    pub time: f64,
    /// オブジェクトの総時間（秒）。
    pub time_total: f64,
}

/// 画像フィルタのオブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct VideoObjectInfo {
    /// オブジェクトの現在の画像サイズの幅。
    pub width: u32,
    /// オブジェクトの現在の画像サイズの高さ。
    pub height: u32,
}

/// 音声フィルタのオブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct AudioObjectInfo {
    /// オブジェクトの現在の音声サンプル位置。
    pub sample_index: u64,
    /// オブジェクトの総サンプル数。
    pub sample_total: u64,
    /// オブジェクトの現在の音声サンプル数。
    pub sample_num: u32,
    /// オブジェクトの現在の音声チャンネル数。
    /// 通常2になります。
    pub channel_num: u32,
}

/// RGBAのピクセル。
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct RgbaPixel {
    /// 赤。
    pub r: u8,
    /// 緑。
    pub g: u8,
    /// 青。
    pub b: u8,
    /// アルファ。
    pub a: u8,
}

/// 画像フィルタ処理のための構造体。
#[derive(Debug)]
pub struct FilterProcVideo {
    /// シーン情報。
    pub scene: SceneInfo,
    /// オブジェクト情報。
    pub object: ObjectInfo,
    /// 画像フィルタ特有のオブジェクト情報。
    pub video_object: VideoObjectInfo,

    pub(crate) inner: *const aviutl2_sys::filter2::FILTER_PROC_VIDEO,
}
unsafe impl Send for FilterProcVideo {}
unsafe impl Sync for FilterProcVideo {}

impl FilterProcVideo {
    /// 現在の画像のデータを取得する。
    /// RGBA32bit で取得されます。
    ///
    /// # Note
    ///
    /// [`FilterPluginTable::as_object`] が `true` の場合、この関数は空の配列を返します。
    pub fn get_image_data<T: Clone + FromBytes + Immutable>(&self) -> Vec<T> {
        if self.video_object.width == 0 || self.video_object.height == 0 {
            log::warn!("width or height is 0, perhaps the filter plugin is an object");
            return vec![];
        }
        let width = self.video_object.width as usize;
        let height = self.video_object.height as usize;
        let mut buffer: Vec<u8> = vec![0; width * height * 4];
        let inner = unsafe { &*self.inner };
        (inner.get_image_data)(buffer.as_mut_ptr() as *mut aviutl2_sys::filter2::PIXEL_RGBA);
        let pixels: &[T] = <[T]>::ref_from_bytes(&buffer).unwrap();

        pixels.to_vec()
    }

    /// 現在の画像のデータを設定する。
    ///
    /// # Panics
    ///
    /// `data` をバイト列に変換した際の長さが `width * height * 4` と一致しない場合、パニックします。
    pub fn set_image_data<T: IntoBytes + Immutable>(&self, data: &[T], width: u32, height: u32) {
        let bytes = &data.as_bytes();
        assert!(bytes.len() == (width * height * 4) as usize);
        let inner = unsafe { &*self.inner };
        (inner.set_image_data)(
            bytes.as_ptr() as *const aviutl2_sys::filter2::PIXEL_RGBA,
            width as i32,
            height as i32,
        );
    }
}

/// 音声フィルタ処理のための構造体。
#[derive(Debug)]
pub struct FilterProcAudio {
    /// シーン情報。
    pub scene: SceneInfo,
    /// オブジェクト情報。
    pub object: ObjectInfo,
    /// 音声フィルタ特有のオブジェクト情報。
    pub audio_object: AudioObjectInfo,

    pub(crate) inner: *const aviutl2_sys::filter2::FILTER_PROC_AUDIO,
}

unsafe impl Send for FilterProcAudio {}
unsafe impl Sync for FilterProcAudio {}

impl FilterProcAudio {
    /// 現在の音声のデータを取得する。
    /// `channel` は 0 が左チャンネル、1 が右チャンネルです。
    pub fn get_sample_data(&self, channel: u32) -> Vec<f32> {
        let sample_num = self.audio_object.sample_num as usize;
        let mut buffer: Vec<f32> = vec![0.0; sample_num];
        let inner = unsafe { &*self.inner };
        (inner.get_sample_data)(buffer.as_mut_ptr(), channel as i32);
        buffer
    }

    /// 現在の音声のデータを設定する。
    /// `channel` は 0 が左チャンネル、1 が右チャンネルです。
    pub fn set_sample_data(&self, data: &[f32], channel: u32) {
        let sample_num = self.audio_object.sample_num as usize;
        assert!(data.len() == sample_num);
        let inner = unsafe { &*self.inner };
        (inner.set_sample_data)(data.as_ptr(), channel as i32);
    }
}
