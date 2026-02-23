#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::{ffi::c_void, mem::MaybeUninit};

use crate::{common::LPCWSTR, plugin2::EDIT_SECTION};

#[repr(C)]
pub union FILTER_ITEM {
    pub track: FILTER_ITEM_TRACK,
    pub checkbox: FILTER_ITEM_CHECKBOX,
    pub color: FILTER_ITEM_COLOR,
    pub select: FILTER_ITEM_SELECT,
    pub file: FILTER_ITEM_FILE,
    pub data: FILTER_ITEM_DATA,
    pub group: FILTER_ITEM_GROUP,
    pub button: FILTER_ITEM_BUTTON,
    pub string: FILTER_ITEM_STRING,
    pub text: FILTER_ITEM_TEXT,
    pub folder: FILTER_ITEM_FOLDER,
}

/// トラックバー項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_TRACK {
    /// 設定の種別（L"track"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: f64,
    /// 設定値の最小
    pub s: f64,
    /// 設定値の最大
    pub e: f64,
    /// 設定値の単位（1.0 / 0.1 / 0.01 / 0.001）
    pub step: f64,
}

/// チェックボックス項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_CHECKBOX {
    /// 設定の種別（L"check"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: bool,
}

/// 色選択項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_COLOR {
    /// 設定の種別（L"color"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: FILTER_ITEM_COLOR_VALUE,
}

/// 色選択項目の設定値の色
#[repr(C)]
#[derive(Clone, Copy)]
pub union FILTER_ITEM_COLOR_VALUE {
    pub code: u32,
    pub bgrx: [u8; 4],
}

/// 選択リスト項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_SELECT {
    /// 設定の種別（L"select"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: i32,
    /// 選択肢リスト (FILTER_ITEM_SELECT_ITEMを列挙して名前がnullのFILTER_ITEM_SELECT_ITEMで終端したリストへのポインタ)
    pub items: *const FILTER_ITEM_SELECT_ITEM,
}

/// 選択リスト項目構造体の選択肢項目
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_SELECT_ITEM {
    /// 選択肢の名前
    pub name: LPCWSTR,
    /// 選択肢の値
    pub value: i32,
}

/// ファイル選択項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_FILE {
    /// 設定の種別（L"file"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: LPCWSTR,
    /// ファイルフィルタ
    pub filefilter: LPCWSTR,
}

/// 汎用データ項目構造体
/// `default_value` は最大1024バイトまでの任意のデータを格納できます。
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_DATA {
    /// 設定の種別（L"data"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値のポインタに更新されます）
    pub value: *mut c_void,
    /// 汎用データのサイズ（1024バイト以下）
    pub size: i32,
    /// デフォルト値（sizeで指定した長さまで有効）
    pub default_value: [MaybeUninit<u8>; 1024],
}

/// 設定グループ項目構造体
/// 自身以降の設定項目をグループ化することが出来ます
/// ※設定名を空にするとグループの終端を定義することが出来ます
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_GROUP {
    /// 設定の種別（L"group"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// デフォルトの表示状態
    pub default_visible: bool,
}

/// ボタン項目構造体
/// ボタンを押すとコールバック関数が呼ばれます
/// ※plugin2.hの編集のコールバック関数と同様な形になります
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_BUTTON {
    /// 設定の種別（L"button"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// コールバック関数 (呼び出し時に各設定項目の設定値が更新されます)
    pub callback: extern "C" fn(edit_section: *mut EDIT_SECTION),
}

/// 文字列項目構造体
/// ※1行の文字列
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_STRING {
    /// 設定の種別（L"string"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値のポインタに更新されます）
    pub value: LPCWSTR,
}

/// テキスト項目構造体
/// ※複数行の文字列
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_TEXT {
    /// 設定の種別（L"text"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値のポインタに更新されます）
    pub value: LPCWSTR,
}

/// フォルダ選択項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_FOLDER {
    /// 設定の種別（L"folder"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値のポインタに更新されます）
    pub value: LPCWSTR,
}

/// RGBA32bit構造体
#[repr(C)]
pub struct PIXEL_RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// シーン情報構造体
#[repr(C)]
pub struct SCENE_INFO {
    /// シーンの解像度（幅）
    pub width: i32,
    /// シーンの解像度（高さ）
    pub height: i32,
    /// シーンのフレームレート
    pub rate: i32,
    /// シーンのフレームレート（スケール）
    pub scale: i32,
    /// シーンのサンプリングレート
    pub sample_rate: i32,
}

/// オブジェクト情報構造体
#[repr(C)]
pub struct OBJECT_INFO {
    /// オブジェクトのID (アプリ起動毎の固有ID)
    /// ※描画対象のオブジェクトの固有ID
    pub id: i64,
    /// オブジェクトの現在のフレーム番号
    pub frame: i32,
    /// オブジェクトの総フレーム数
    pub frame_total: i32,
    /// オブジェクトの現在の時間(秒)
    pub time: f64,
    /// オブジェクトの総時間(秒)
    pub time_total: f64,
    /// オブジェクトの現在の画像サイズの幅 (画像フィルタのみ)
    pub width: i32,
    /// オブジェクトの現在の画像サイズの高さ (画像フィルタのみ)
    pub height: i32,
    /// オブジェクトの現在の音声サンプル位置 (音声フィルタのみ)
    pub sample_index: i64,
    /// オブジェクトの総サンプル数 (音声フィルタのみ)
    pub sample_total: i64,
    /// オブジェクトの現在の音声サンプル数 (音声フィルタのみ)
    pub sample_num: i32,
    /// オブジェクトの現在の音声チャンネル数 (音声フィルタのみ) ※通常2になります
    pub channel_num: i32,
    /// オブジェクトの内の対象エフェクトのID (アプリ起動毎の固有ID)
    /// ※処理対象のフィルタ効果、オブジェクト入出力の固有ID
    pub effect_id: i64,
}

/// 画像フィルタ処理用構造体
#[repr(C)]
pub struct FILTER_PROC_VIDEO {
    /// シーン情報
    pub scene: *const SCENE_INFO,

    /// オブジェクト情報
    pub object: *const OBJECT_INFO,

    /// 現在の画像のデータを取得する（VRAMからデータを取得します）
    /// buffer: 画像データの格納先へのポインタ
    pub get_image_data: unsafe extern "C" fn(buffer: *mut PIXEL_RGBA),

    /// 現在の画像のデータを設定します（VRAMへデータを書き込みます）
    /// buffer: 画像データへのポインタ
    /// width,height: 画像サイズ
    pub set_image_data: unsafe extern "C" fn(buffer: *const PIXEL_RGBA, width: i32, height: i32),

    // 現在のオブジェクトの画像データのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    // 戻り値		: オブジェクトの画像データへのポインタ
    //				  ※現在の画像が変更(set_image_data)されるかフィルタ処理の終了まで有効
    pub get_image_texture2d: unsafe extern "C" fn() -> *mut c_void,

    // 現在のフレームバッファの画像データのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    // 戻り値		: フレームバッファの画像データへのポインタ
    //				  ※フィルタ処理の終了まで有効
    pub get_framebuffer_texture2d: unsafe extern "C" fn() -> *mut c_void,
}

/// 音声フィルタ処理用構造体
#[repr(C)]
pub struct FILTER_PROC_AUDIO {
    /// シーン情報
    pub scene: *const SCENE_INFO,

    /// オブジェクト情報
    pub object: *const OBJECT_INFO,

    /// 現在の音声のデータを取得する
    /// buffer: 音声データの格納先へのポインタ ※音声データはPCM(float)32bit
    /// channel: 音声データのチャンネル ( 0 = 左チャンネル / 1 = 右チャンネル )
    pub get_sample_data: unsafe extern "C" fn(buffer: *mut f32, channel: i32),

    /// 現在の音声のデータを設定する
    /// buffer: 音声データへのポインタ ※音声データはPCM(float)32bit
    /// channel: 音声データのチャンネル ( 0 = 左チャンネル / 1 = 右チャンネル )
    pub set_sample_data: unsafe extern "C" fn(buffer: *const f32, channel: i32),
}

impl FILTER_PLUGIN_TABLE {
    /// 画像フィルタをサポートする
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声フィルタをサポートする
    pub const FLAG_AUDIO: i32 = 2;
    /// メディアオブジェクトの初期入力をする (メディアオブジェクトにする場合)
    pub const FLAG_INPUT: i32 = 4;
    /// フィルタオブジェクトをサポートする (フィルタオブジェクトに対応する場合)
    /// フィルタオブジェクトの場合は画像サイズの変更が出来ません
    pub const FLAG_FILTER: i32 = 8;
}

/// フィルタプラグイン構造体
#[repr(C)]
pub struct FILTER_PLUGIN_TABLE {
    /// フラグ
    /// 画像と音声のフィルタ処理は別々のスレッドで処理されます
    pub flag: i32,
    /// プラグインの名前
    pub name: LPCWSTR,
    /// ラベルの初期値 (nullptrならデフォルトのラベルになります)
    pub label: LPCWSTR,
    /// プラグインの情報
    pub information: LPCWSTR,

    /// 設定項目の定義 (FILTER_ITEM_XXXポインタを列挙してnull終端したリストへのポインタ)
    pub items: *const *const c_void,

    /// 画像フィルタ処理関数へのポインタ (FLAG_VIDEOが有効の時のみ呼ばれます)
    pub func_proc_video: Option<extern "C" fn(video: *mut FILTER_PROC_VIDEO) -> bool>,

    /// 音声フィルタ処理関数へのポインタ (FLAG_AUDIOが有効の時のみ呼ばれます)
    pub func_proc_audio: Option<extern "C" fn(audio: *mut FILTER_PROC_AUDIO) -> bool>,
}
