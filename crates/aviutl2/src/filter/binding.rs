use std::borrow::Cow;
use zerocopy::IntoBytes;

use super::config;
use crate::common::{AnyResult, AviUtl2Info, FileFilter, Rational32, Win32WindowHandle, Yc48, f16};

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
    pub input_type: FilterType,

    /// オブジェクトの初期入力をするかどうか。（メディアオブジェクトの場合）
    pub wants_initial_input: bool,

    /// 設定項目。
    pub config_items: Vec<config::FilterConfigItem>,
}
/// 動画・画像と音声の入力情報をまとめた構造体。
#[derive(Debug, Clone)]
pub struct InputInfo {
    // /// 動画・画像のフォーマット。
    // pub video: Option<VideoInputInfo>,
    // /// 音声のフォーマット。
    // pub audio: Option<AudioInputInfo>,
}

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
            FilterType::Video => 1,
            FilterType::Audio => 2,
            FilterType::Both => 3,
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
}
