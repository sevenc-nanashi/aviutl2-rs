mod audio;
mod video;

pub use audio::*;
pub use video::*;

use super::config;
use crate::common::Rational32;

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

    /// 対応している機能のフラグ。
    pub flags: FilterPluginFlags,

    /// 設定項目。
    pub config_items: Vec<config::FilterConfigItem>,
}

define_bitflag! {
    /// フィルタプラグインのフラグ。
    ///
    /// # See Also
    ///
    /// - [`crate::bitflag!`]
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    pub struct FilterPluginFlags: i32 {
        /// 画像フィルタをサポートするかどうか。
        video: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_VIDEO,

        /// 音声フィルタをサポートするかどうか。
        audio: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_AUDIO,

        /// 入力として動作するかどうか。
        /// `true` の場合、カスタムオブジェクトとして動作します。
        /// `false` の場合、フィルタ効果として動作します。
        input: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_INPUT,

        /// フィルタオブジェクトをサポートするかどうか。
        /// `true` の場合、フィルタオブジェクトとして使えるようになります。
        filter: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_FILTER,
    }
}

/// フィルタプラグインのトレイト。
/// このトレイトを実装し、[`crate::register_filter_plugin!`] マクロを使用してプラグインを登録します。
pub trait FilterPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: crate::common::AviUtl2Info) -> crate::common::AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> crate::filter::FilterPluginTable;

    /// 画像フィルタ処理関数。
    ///
    /// # Note
    ///
    /// フィルタオブジェクトの場合、画像サイズは変更できません。
    fn proc_video(
        &self,
        _config: &[crate::filter::FilterConfigItem],
        _video: &mut crate::filter::FilterProcVideo,
    ) -> crate::common::AnyResult<()> {
        anyhow::bail!("proc_video is not implemented");
    }

    /// 音声フィルタ処理関数。
    fn proc_audio(
        &self,
        _config: &[crate::filter::FilterConfigItem],
        _audio: &mut crate::filter::FilterProcAudio,
    ) -> crate::common::AnyResult<()> {
        anyhow::bail!("proc_audio is not implemented");
    }

    /// シングルトンインスタンスを参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance<R>(f: impl FnOnce(&Self) -> R) -> R
    where
        Self: crate::filter::__bridge::FilterSingleton,
    {
        <Self as crate::filter::__bridge::FilterSingleton>::with_instance(f)
    }

    /// シングルトンインスタンスを可変参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance_mut<R>(f: impl FnOnce(&mut Self) -> R) -> R
    where
        Self: crate::filter::__bridge::FilterSingleton,
    {
        <Self as crate::filter::__bridge::FilterSingleton>::with_instance_mut(f)
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
    /// 描画対象のオブジェクトの固有ID。
    /// アプリ起動ごとの固有IDです。
    pub id: i64,
    /// オブジェクトの内の対象エフェクトのID。
    /// アプリ起動ごとの固有IDです。
    pub effect_id: i64,
    /// オブジェクトのレイヤー番号。
    pub layer: u32,
    /// オブジェクトの現在のフレーム番号。
    pub frame: u32,
    /// オブジェクトの総フレーム数。
    pub frame_total: u32,
    /// オブジェクトの現在の時間（秒）。
    pub time: f64,
    /// オブジェクトの総時間（秒）。
    pub time_total: f64,
    /// オブジェクトがフィルタオブジェクトかどうか。
    pub is_filter_object: bool,
    /// シーン基準のオブジェクトの開始フレーム。
    pub frame_s: u32,
    /// シーン基準のオブジェクトの終了フレーム。
    pub frame_e: u32,
}

impl ObjectInfo {
    /// シーン基準のオブジェクトのフレーム範囲。
    pub fn frame_range(&self) -> std::ops::RangeInclusive<u32> {
        self.frame_s..=self.frame_e
    }
}

/// フィルタ処理のエラー。
#[derive(Debug, thiserror::Error)]
pub enum FilterProcError {
    #[error("api call failed")]
    ApiCallFailed,
    #[error("input string contains null byte")]
    InputCwstrContainsNull(#[from] crate::common::NullByteError),
    #[error("value is out of range")]
    ValueOutOfRange,
}

pub type FilterProcResult<T> = Result<T, FilterProcError>;
