#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::common::LPCWSTR;
use std::ffi::c_void;

pub use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::BITMAPINFOHEADER,
    Media::Audio::WAVEFORMATEX,
};

/// 入力ファイル情報構造体
/// 画像フォーマットはRGB24bit,RGBA32bit,PA64,HF64,YUY2,YC48が対応しています
/// 音声フォーマットはPCM16bit,PCM(float)32bitが対応しています
/// ※PA64はDXGI_FORMAT_R16G16B16A16_UNORM(乗算済みα)です
/// ※HF64はDXGI_FORMAT_R16G16B16A16_FLOAT(乗算済みα)です(内部フォーマット)
/// ※YC48は互換対応のフォーマットです
#[repr(C)]
pub struct INPUT_INFO {
    /// フラグ
    pub flag: i32,
    /// フレームレート、スケール
    pub rate: i32,
    /// フレームレート、スケール
    pub scale: i32,
    /// フレーム数
    pub n: i32,
    /// 画像フォーマットへのポインタ(次に関数が呼ばれるまで内容を有効にしておく)
    pub format: *const BITMAPINFOHEADER,
    /// 画像フォーマットのサイズ
    pub format_size: i32,
    /// 音声サンプル数
    pub audio_n: i32,
    /// 音声フォーマットへのポインタ(次に関数が呼ばれるまで内容を有効にしておく)
    pub audio_format: *const WAVEFORMATEX,
    /// 音声フォーマットのサイズ
    pub audio_format_size: i32,
}

impl INPUT_INFO {
    /// 画像データあり
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声データあり
    pub const FLAG_AUDIO: i32 = 2;
    /// フレーム番号を時間から算出する ※func_time_to_frame()が呼ばれるようになる
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
    /// file : ファイル名
    /// 戻り値 : 入力ファイルハンドル ※失敗時はnullptrを返却
    pub func_open: Option<unsafe extern "C" fn(file: LPCWSTR) -> INPUT_HANDLE>,
    /// 入力ファイルをクローズする関数へのポインタ
    /// ih : 入力ファイルハンドル
    /// 戻り値 : 成功時はtrueを返却
    pub func_close: Option<extern "C" fn(ih: INPUT_HANDLE) -> bool>,
    /// 入力ファイルの情報を取得する関数へのポインタ
    /// ih : 入力ファイルハンドル
    /// iip : 入力ファイル情報構造体へのポインタ
    /// 戻り値 : 成功時はtrueを返却
    pub func_info_get: Option<extern "C" fn(ih: INPUT_HANDLE, iip: *mut INPUT_INFO) -> bool>,
    /// 画像データを読み込む関数へのポインタ
    /// ih : 入力ファイルハンドル
    /// frame : 読み込むフレーム番号
    /// buf : データを読み込むバッファへのポインタ
    /// 戻り値 : 読み込んだデータサイズ
    pub func_read_video:
        Option<extern "C" fn(ih: INPUT_HANDLE, frame: i32, buf: *mut c_void) -> i32>,
    /// 音声データを読み込む関数へのポインタ
    /// ih : 入力ファイルハンドル
    /// start : 読み込み開始サンプル番号
    /// length : 読み込むサンプル数
    /// buf : データを読み込むバッファへのポインタ
    /// 戻り値 : 読み込んだサンプル数
    pub func_read_audio:
        Option<extern "C" fn(ih: INPUT_HANDLE, start: i32, length: i32, buf: *mut c_void) -> i32>,
    /// 入力設定のダイアログを要求された時に呼ばれる関数へのポインタ (nullptrなら呼ばれません)
    /// hwnd : ウィンドウハンドル
    /// dll_hinst : インスタンスハンドル
    /// 戻り値 : 成功時はtrueを返却
    pub func_config: Option<extern "C" fn(hwnd: HWND, dll_hinst: HINSTANCE) -> bool>,
    /// 入力ファイルの読み込み対象トラックを設定する関数へのポインタ (FLAG_MULTI_TRACKが有効の時のみ呼ばれます)
    /// func_open()の直後にトラック数取得、トラック番号設定が呼ばれます。※オープン直後の設定以降は呼ばれません
    /// ih : 入力ファイルハンドル
    /// type : メディア種別 ( 0 = 映像 / 1 = 音声 )
    /// index : トラック番号 ( -1 が指定された場合はトラック数の取得 )
    /// 戻り値 : 設定したトラック番号 (失敗した場合は -1 を返却)
    /// トラック数の取得の場合は設定可能なトラックの数 (メディアが無い場合は 0 を返却)
    pub func_set_track:
        Option<extern "C" fn(ih: INPUT_HANDLE, track_type: i32, track_index: i32) -> i32>,
    /// 映像の時間から該当フレーム番号を算出する時に呼ばれる関数へのポインタ (FLAG_TIME_TO_FRAMEが有効の時のみ呼ばれます)
    /// 画像データを読み込む前に呼び出され、結果のフレーム番号で読み込むようになります。
    /// ※FLAG_TIME_TO_FRAMEを利用する場合のINPUT_INFOのrate,scale情報は平均フレームレートを表す値を設定してください
    /// ih : 入力ファイルハンドル
    /// time : 映像の時間(秒)
    /// 戻り値 : 映像の時間に対応するフレーム番号
    pub func_time_to_frame: Option<extern "C" fn(ih: INPUT_HANDLE, time: f64) -> i32>,
}

impl INPUT_PLUGIN_TABLE {
    /// 画像をサポートする
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声をサポートする
    pub const FLAG_AUDIO: i32 = 2;
    /// データの同時取得をサポートする
    /// ※同一ハンドルで画像と音声の取得関数が同時に呼ばれる
    /// ※異なるハンドルで各関数が同時に呼ばれる
    pub const FLAG_CONCURRENT: i32 = 16;
    /// マルチトラックをサポートする ※func_set_track()が呼ばれるようになる
    pub const FLAG_MULTI_TRACK: i32 = 32;

    pub const TRACK_TYPE_VIDEO: i32 = 0;
    pub const TRACK_TYPE_AUDIO: i32 = 1;
}
