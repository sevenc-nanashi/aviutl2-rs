#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code
)]

use std::ffi::c_void;

pub use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::{BI_RGB, BITMAPINFOHEADER},
    Media::{Audio::WAVE_FORMAT_PCM, Multimedia::WAVE_FORMAT_IEEE_FLOAT},
};

macro_rules! fourcc {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24))
    };
}

/// YUY2（YUV 4:2:2）フォーマット
pub const BI_YUY2: u32 = fourcc!('Y', 'U', 'Y', '2');
/// PA64（DXGI_FORMAT_R16G16B16A16_UNORM、乗算済みα）フォーマット
pub const BI_PA64: u32 = fourcc!('P', 'A', '6', '4');
/// YC48（互換対応の旧内部フォーマット）フォーマット
pub const BI_YC48: u32 = fourcc!('Y', 'C', '4', '8');
/// HF64（DXGI_FORMAT_R16G16B16A16_FLOAT、乗算済みα）フォーマット
pub const BI_HF64: u32 = fourcc!('H', 'F', '6', '4');

pub type LPCWSTR = *const u16;

/// 出力情報構造体
#[repr(C)]
pub struct OUTPUT_INFO {
    /// フラグ
    pub flag: i32,
    /// 縦横サイズ
    pub w: i32,
    /// 縦横サイズ
    pub h: i32,
    /// フレームレート
    pub rate: i32,
    /// フレームレート（スケール）
    pub scale: i32,
    /// フレーム数
    pub n: i32,
    /// 音声サンプリングレート
    pub audio_rate: i32,
    /// 音声チャンネル数
    pub audio_ch: i32,
    /// 音声サンプリング数
    pub audio_n: i32,
    /// セーブファイル名へのポインタ
    pub savefile: LPCWSTR,
    /// DIB形式の画像データを取得します
    ///
    /// # Safety
    /// 画像データポインタの内容は次に外部関数を使うかメインに処理を戻すまで有効
    ///
    /// # See Also
    /// [`BI_RGB`]
    /// [`BI_YUY2`]
    /// [`BI_PA64`]
    /// [`BI_YC48`]
    pub func_get_video: Option<extern "C" fn(frame: i32, format: u32) -> *mut c_void>,
    /// PCM形式の音声データへのポインタを取得します
    ///
    /// # Safety
    /// 音声データポインタの内容は次に外部関数を使うかメインに処理を戻すまで有効
    ///
    /// # See Also
    /// [`WAVE_FORMAT_PCM`]
    /// [`WAVE_FORMAT_IEEE_FLOAT`]
    pub func_get_audio: Option<
        extern "C" fn(start: i32, length: i32, readed: *mut i32, format: u32) -> *mut c_void,
    >,
    /// 中断するか調べます
    ///
    /// # Returns
    /// `true`なら中断
    pub func_is_abort: Option<extern "C" fn() -> bool>,
    /// 残り時間を表示させます
    ///
    /// # Args
    /// - `now`: 処理しているフレーム番号
    /// - `total`: 処理する総フレーム数
    pub func_rest_time_disp: Option<extern "C" fn(now: i32, total: i32)>,
    /// データ取得のバッファ数(フレーム数)を設定します ※標準は4になります
    ///
    /// バッファ数の半分のデータを先読みリクエストするようになります
    ///
    /// # Args
    /// - `video_size`: 画像データのバッファ数
    /// - `audio_size`: 音声データのバッファ数
    pub func_set_buffer_size: Option<extern "C" fn(video_size: i32, audio_size: i32)>,
}

impl OUTPUT_INFO {
    /// 画像データあり
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声データあり
    pub const FLAG_AUDIO: i32 = 2;
}

/// 出力プラグイン構造体
#[repr(C)]
pub struct OUTPUT_PLUGIN_TABLE {
    /// フラグ ※未使用
    pub flag: i32,
    /// プラグインの名前
    pub name: LPCWSTR,
    /// ファイルのフィルタ
    pub filefilter: LPCWSTR,
    /// プラグインの情報
    pub information: LPCWSTR,
    /// 出力時に呼ばれる関数へのポインタ
    pub func_output: Option<extern "C" fn(oip: *mut OUTPUT_INFO) -> bool>,
    /// 出力設定のダイアログを要求された時に呼ばれる関数へのポインタ (nullなら呼ばれません)
    pub func_config: Option<extern "C" fn(hwnd: HWND, dll_hinst: HINSTANCE) -> bool>,
    /// 出力設定のテキスト情報を取得する時に呼ばれる関数へのポインタ (nullなら呼ばれません)
    ///
    /// # Returns
    /// 出力設定のテキスト情報(次に関数が呼ばれるまで内容を有効にしておく)
    pub func_get_config_text: Option<extern "C" fn() -> LPCWSTR>,
}

impl OUTPUT_PLUGIN_TABLE {
    /// 画像をサポートする
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声をサポートする
    pub const FLAG_AUDIO: i32 = 2;
}
