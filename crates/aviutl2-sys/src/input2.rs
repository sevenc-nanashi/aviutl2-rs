#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code
)]

use std::ffi::c_void;

pub use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::{BI_BITFIELDS, BI_RGB, BITMAPINFOHEADER},
    Media::{
        Audio::{WAVE_FORMAT_PCM, WAVEFORMATEX},
        Multimedia::WAVE_FORMAT_IEEE_FLOAT,
    },
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

/// 入力ファイル情報構造体
#[repr(C)]
pub struct INPUT_INFO {
    /// フラグ
    pub flag: i32,
    /// フレームレート
    pub rate: i32,
    /// フレームレート（スケール）
    pub scale: i32,
    /// フレーム数
    pub n: i32,
    /// 画像フォーマットへのポインタ
    ///
    /// # Safety
    /// 次に関数が呼ばれるまで内容を有効にしておく
    ///
    /// # See Also
    /// [`BITMAPINFOHEADER`]
    /// [`BI_RGB`]
    /// [`BI_YUY2`]
    /// [`BI_PA64`]
    /// [`BI_YC48`]
    pub format: *mut BITMAPINFOHEADER,
    /// 画像フォーマットのサイズ
    pub format_size: i32,
    /// 音声サンプル数
    pub audio_n: i32,
    /// 音声フォーマットへのポインタ
    ///
    /// # Safety
    /// 次に関数が呼ばれるまで内容を有効にしておく
    pub audio_format: *mut WAVEFORMATEX,
    /// 音声フォーマットのサイズ
    pub audio_format_size: i32,
}

impl INPUT_INFO {
    /// 画像データあり
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声データあり
    pub const FLAG_AUDIO: i32 = 2;
    /// フレーム番号を時間から算出する
    /// （func_time_to_frame()が呼ばれるようになる）
    pub const FLAG_TIME_TO_FRAME: i32 = 16;
}

/// 入力ファイルハンドル
pub type INPUT_HANDLE = *mut c_void;

/// 入力プラグイン構造体
#[repr(C)]
pub struct INPUT_PLUGIN_TABLE {
    /// フラグ
    pub flag: i32,
    /// プラグインの名前
    pub name: LPCWSTR,
    /// 入力ファイルフィルタ
    pub filefilter: LPCWSTR,
    /// プラグインの情報
    pub information: LPCWSTR,
    /// 入力ファイルをオープンする関数へのポインタ
    ///
    /// # Args
    /// - `file`: ファイル名
    ///
    /// # Returns
    /// `INPUT_HANDLE`
    pub func_open: Option<extern "C" fn(file: LPCWSTR) -> INPUT_HANDLE>,
    /// 入力ファイルをクローズする関数へのポインタ
    ///
    /// # Args
    /// - `ih`: 入力ファイルハンドル
    ///
    /// # Returns
    /// `true`なら成功
    pub func_close: Option<extern "C" fn(ih: INPUT_HANDLE) -> bool>,
    /// 入力ファイルの情報を取得する関数へのポインタ
    ///
    /// # Args
    /// - `ih`: 入力ファイルハンドル
    /// - `iip`: 入力ファイル情報構造体へのポインタ
    ///
    /// # Returns
    /// `true`なら成功
    pub func_info_get: Option<extern "C" fn(ih: INPUT_HANDLE, iip: *mut INPUT_INFO) -> bool>,
    /// 画像データを読み込む関数へのポインタ
    ///
    /// # Args
    /// - `ih`: 入力ファイルハンドル
    /// - `frame`: 読み込むフレーム番号
    /// - `buf`: データを読み込むバッファへのポインタ
    ///
    /// # Returns
    /// 読み込んだデータサイズ
    pub func_read_video:
        Option<extern "C" fn(ih: INPUT_HANDLE, frame: i32, buf: *mut c_void) -> i32>,
    /// 音声データを読み込む関数へのポインタ
    ///
    /// # Args
    /// - `ih`: 入力ファイルハンドル
    /// - `start`: 読み込み開始サンプル番号
    /// - `length`: 読み込むサンプル数
    /// - `buf`: データを読み込むバッファへのポインタ
    ///
    /// # Returns
    /// 読み込んだサンプル数
    pub func_read_audio:
        Option<extern "C" fn(ih: INPUT_HANDLE, start: i32, length: i32, buf: *mut c_void) -> i32>,
    /// 入力設定のダイアログを要求された時に呼ばれる関数へのポインタ (nullなら呼ばれません)
    ///
    /// # Args
    /// - `hwnd`: ウィンドウハンドル
    /// - `dll_hinst`: インスタンスハンドル
    ///
    /// # Returns
    /// `true`なら成功
    pub func_config: Option<extern "C" fn(hwnd: HWND, dll_hinst: HINSTANCE) -> bool>,
    /// 入力ファイルの読み込み対象トラックを設定する関数へのポインタ (FLAG_MULTI_TRACKが有効の時のみ呼ばれます)
    ///
    /// `func_open()`の直後にトラック数取得、トラック番号設定が呼ばれます。※オープン直後の設定以降は呼ばれません
    ///
    /// # Args
    /// - `ih`: 入力ファイルハンドル
    /// - `track_type`: メディア種別 ( 0 = 映像 / 1 = 音声 )
    /// - `track_index`: トラック番号 ( -1 が指定された場合はトラック数の取得 )
    ///
    /// # Returns
    /// 設定したトラック番号 (失敗した場合は -1 を返却)
    /// トラック数の取得の場合は設定可能なトラックの数 (メディアが無い場合は 0 を返却)
    ///
    /// # See Also
    /// [`INPUT_PLUGIN_TABLE::FLAG_MULTI_TRACK`]
    /// [`INPUT_PLUGIN_TABLE::TRACK_TYPE_VIDEO`]
    /// [`INPUT_PLUGIN_TABLE::TRACK_TYPE_AUDIO`]
    pub func_set_track:
        Option<extern "C" fn(ih: INPUT_HANDLE, track_type: i32, track_index: i32) -> i32>,
    /// 映像の時間から該当フレーム番号を算出する時に呼ばれる関数へのポインタ (FLAG_TIME_TO_FRAMEが有効の時のみ呼ばれます)
    ///
    /// 画像データを読み込む前に呼び出され、結果のフレーム番号で読み込むようになります。
    ///
    /// # Remarks
    /// FLAG_TIME_TO_FRAMEを利用する場合のINPUT_INFOのrate,scale情報は平均フレームレートを表す値を設定してください
    ///
    /// # Args
    /// - `ih`: 入力ファイルハンドル
    /// - `time`: 映像の時間(秒)
    ///
    /// # Returns
    /// 映像の時間に対応するフレーム番号
    pub func_time_to_frame: Option<extern "C" fn(ih: INPUT_HANDLE, time: f64) -> i32>,
}

impl INPUT_PLUGIN_TABLE {
    /// 画像をサポートする
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声をサポートする
    pub const FLAG_AUDIO: i32 = 2;
    /// 画像・音声データの同時取得をサポートする ※画像と音声取得関数が同時に呼ばれる
    pub const FLAG_CONCURRENT: i32 = 16;
    /// マルチトラックをサポートする ※func_set_track()が呼ばれるようになる
    pub const FLAG_MULTI_TRACK: i32 = 32;

    pub const TRACK_TYPE_VIDEO: i32 = 0;
    pub const TRACK_TYPE_AUDIO: i32 = 1;
}
