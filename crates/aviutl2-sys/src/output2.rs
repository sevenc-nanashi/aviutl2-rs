#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::{common::LPCWSTR, plugin2::PROJECT_FILE};
use std::ffi::c_void;

pub use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::BITMAPINFOHEADER,
    Media::{Audio::WAVE_FORMAT_PCM, Multimedia::WAVE_FORMAT_IEEE_FLOAT},
};
/// 出力情報構造体
#[repr(C)]
pub struct OUTPUT_INFO {
    /// フラグ
    pub flag: i32,
    /// 縦横サイズ
    pub w: i32,
    /// 縦横サイズ
    pub h: i32,
    /// フレームレート、スケール
    pub rate: i32,
    /// フレームレート、スケール
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
    /// frame : フレーム番号
    /// format : 画像フォーマット
    /// 0(BI_RGB) = RGB24bit / 'P''A''6''4' = PA64 / 'H''F''6''4' = HF64 / 'Y''U''Y''2' = YUY2 / 'Y''C''4''8' = YC48
    /// ※PA64はDXGI_FORMAT_R16G16B16A16_UNORM(乗算済みα)です
    /// ※HF64はDXGI_FORMAT_R16G16B16A16_FLOAT(乗算済みα)です(内部フォーマット)
    /// ※YC48は互換対応のフォーマットです
    /// 戻り値 : データへのポインタ
    /// 画像データポインタの内容は次に外部関数を使うかメインに処理を戻すまで有効
    pub func_get_video: Option<extern "C" fn(frame: i32, format: u32) -> *mut c_void>,
    /// PCM形式の音声データへのポインタを取得します
    /// start : 開始サンプル番号
    /// length : 読み込むサンプル数
    /// readed : 読み込まれたサンプル数
    /// format : 音声フォーマット
    /// 1(WAVE_FORMAT_PCM) = PCM16bit / 3(WAVE_FORMAT_IEEE_FLOAT) = PCM(float)32bit
    /// 戻り値 : データへのポインタ
    /// 音声データポインタの内容は次に外部関数を使うかメインに処理を戻すまで有効
    pub func_get_audio: Option<
        extern "C" fn(start: i32, length: i32, readed: *mut i32, format: u32) -> *mut c_void,
    >,
    /// 中断するか調べます
    /// 戻り値 : trueなら中断
    pub func_is_abort: Option<extern "C" fn() -> bool>,
    /// 残り時間を表示させます
    /// now : 処理しているフレーム番号
    /// total : 処理する総フレーム数
    /// 戻り値 : trueなら成功
    pub func_rest_time_disp: Option<extern "C" fn(now: i32, total: i32)>,
    /// データ取得のバッファ数(フレーム数)を設定します ※標準は4になります
    /// バッファ数の半分のデータを先読みリクエストするようになります
    /// video : 画像データのバッファ数
    /// audio : 音声データのバッファ数
    pub func_set_buffer_size: Option<extern "C" fn(video_size: i32, audio_size: i32)>,
}

impl OUTPUT_INFO {
    /// 画像データあり
    pub const FLAG_VIDEO: i32 = 1;
    /// 画像データあり
    pub const FLAG_AUDIO: i32 = 2;
}

/// 出力プラグイン構造体
#[repr(C)]
pub struct OUTPUT_PLUGIN_TABLE {
    /// フラグ
    pub flag: i32,
    /// プラグインの名前
    pub name: LPCWSTR,
    /// ファイルのフィルタ
    pub filefilter: LPCWSTR,
    /// プラグインの情報
    pub information: LPCWSTR,
    /// 出力時に呼ばれる関数へのポインタ
    /// 戻り値 : 成功時はtrueを返却
    pub func_output: Option<extern "C" fn(oip: *mut OUTPUT_INFO) -> bool>,
    /// 出力設定のダイアログを要求された時に呼ばれる関数へのポインタ (nullptrなら呼ばれません)
    /// 戻り値 : 成功時はtrueを返却
    pub func_config: Option<extern "C" fn(hwnd: HWND, dll_hinst: HINSTANCE) -> bool>,
    /// 出力設定のテキスト情報を取得する時に呼ばれる関数へのポインタ (nullptrなら呼ばれません)
    /// 戻り値 : 出力設定のテキスト情報へのポインタ (次に関数が呼ばれるまで内容を有効にしておく)
    pub func_get_config_text: Option<extern "C" fn() -> LPCWSTR>,
    /// プロジェクトファイル側から出力設定の読み込み要求時に呼ばれる関数へのポインタ (FLAG_PROJECT_CONFIGが有効の時のみ呼ばれます)
    /// project : プロジェクトファイル構造体へのポインタ
    /// 戻り値 : 成功時はtrueを返却
    pub func_load_project_config: Option<extern "C" fn(project: *mut PROJECT_FILE) -> bool>,
    /// プロジェクトファイル側への出力設定の書き込み要求時に呼ばれる関数へのポインタ (FLAG_PROJECT_CONFIGが有効の時のみ呼ばれます)
    /// project : プロジェクトファイル構造体へのポインタ
    /// 戻り値 : 成功時はtrueを返却
    pub func_save_project_config: Option<extern "C" fn(project: *mut PROJECT_FILE) -> bool>,
}

impl OUTPUT_PLUGIN_TABLE {
    /// 画像をサポートする
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声をサポートする
    pub const FLAG_AUDIO: i32 = 2;
    /// 静止画出力のみサポートする (OUTPUT_INFOが1フレーム出力になります)
    /// ※静止画出力では出力完了時の通知やサウンド再生をしません
    pub const FLAG_IMAGE: i32 = 4;
    /// プロジェクトファイルの設定保持をサポートする
    /// ※プロジェクトファイル側に出力設定を保持する場合に指定します
    pub const FLAG_PROJECT_CONFIG: i32 = 8;
}
