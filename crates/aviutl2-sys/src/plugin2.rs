#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::common::LPCWSTR;
use crate::{filter2::FILTER_PLUGIN_TABLE, input2::INPUT_PLUGIN_TABLE, module2::SCRIPT_MODULE_TABLE, output2::OUTPUT_PLUGIN_TABLE};
use std::ffi::c_void;
use std::os::raw::c_char;

pub use windows_sys::Win32::Foundation::HWND;

/// Cの`LPCSTR`相当（UTF-8 のヌル終端文字列）
pub type LPCSTR = *const c_char;

/// オブジェクトハンドル（不透明ポインタ）
pub type OBJECT_HANDLE = *mut c_void;

/// レイヤー・フレーム情報構造体
#[repr(C)]
pub struct OBJECT_LAYER_FRAME {
    /// レイヤー番号
    pub layer: i32,
    /// 開始フレーム番号
    pub start: i32,
    /// 終了フレーム番号
    pub end: i32,
}

/// 冗長（非推奨）オブジェクトフレーム情報構造体
#[repr(C)]
pub struct DEPRECATED_OBJECT_FRAME_INFO {
    pub start: i32,
    pub end: i32,
}

/// 編集情報構造体
#[repr(C)]
pub struct EDIT_INFO {
    /// シーンの解像度
    pub width: i32,
    /// シーンの解像度
    pub height: i32,
    /// シーンのフレームレート
    pub rate: i32,
    /// シーンのフレームレート（スケール）
    pub scale: i32,
    /// シーンのサンプリングレート
    pub sample_rate: i32,
    /// 現在のカーソルのフレーム番号
    pub frame: i32,
    /// 現在のレイヤーの表示開始番号
    pub layer: i32,
    /// オブジェクトが存在する最大のフレーム番号
    pub frame_max: i32,
    /// オブジェクトが存在する最大のレイヤー番号
    pub layer_max: i32,
}

/// 編集セクション構造体（プロジェクト編集用の機能群）
#[repr(C)]
pub struct EDIT_SECTION {
    /// 編集情報
    pub info: *mut EDIT_INFO,

    /// 指定位置にオブジェクトエイリアスを作成
    pub create_object_from_alias:
        unsafe extern "C" fn(alias: LPCSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 指定フレーム以降でオブジェクトを検索
    pub find_object: unsafe extern "C" fn(layer: i32, frame: i32) -> OBJECT_HANDLE,

    /// 冗長（非推奨）フレーム情報の取得
    pub deprecated_get_object_frame_info:
        unsafe extern "C" fn(object: OBJECT_HANDLE) -> DEPRECATED_OBJECT_FRAME_INFO,

    /// レイヤー・フレーム情報の取得
    pub get_object_layer_frame:
        unsafe extern "C" fn(object: OBJECT_HANDLE) -> OBJECT_LAYER_FRAME,

    /// オブジェクトのエイリアスデータ取得（UTF-8）
    pub get_object_alias: unsafe extern "C" fn(object: OBJECT_HANDLE) -> LPCSTR,

    /// 設定項目の値を文字列（UTF-8）で取得
    pub get_object_item_value:
        unsafe extern "C" fn(object: OBJECT_HANDLE, effect: LPCWSTR, item: LPCWSTR) -> LPCSTR,

    /// 設定項目の値を文字列（UTF-8）で設定
    pub set_object_item_value: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        item: LPCWSTR,
        value: LPCSTR,
    ) -> bool,

    /// オブジェクトを移動
    pub move_object: unsafe extern "C" fn(object: OBJECT_HANDLE, layer: i32, frame: i32) -> bool,

    /// オブジェクトを削除
    pub delete_object: unsafe extern "C" fn(object: OBJECT_HANDLE),

    /// 設定ウィンドウで選択中のオブジェクト取得
    pub get_focus_object: unsafe extern "C" fn() -> OBJECT_HANDLE,

    /// 設定ウィンドウで選択するオブジェクトを設定
    pub set_focus_object: unsafe extern "C" fn(object: OBJECT_HANDLE),
}

/// 編集ハンドル構造体
#[repr(C)]
pub struct EDIT_HANDLE {
    /// プロジェクトデータの編集を行うコールバックを呼び出す
    pub call_edit_section:
        unsafe extern "C" fn(func_proc_edit: unsafe extern "C" fn(edit: *mut EDIT_SECTION)) -> bool,
}

/// プロジェクトファイル構造体
#[repr(C)]
pub struct PROJECT_FILE {
    /// 文字列(UTF-8)を取得
    pub get_param_string: unsafe extern "C" fn(key: LPCSTR) -> LPCSTR,
    /// 文字列(UTF-8)を保存
    pub set_param_string: unsafe extern "C" fn(key: LPCSTR, value: LPCSTR),
    /// バイナリを取得
    pub get_param_binary: unsafe extern "C" fn(key: LPCSTR, data: *mut c_void, size: i32) -> bool,
    /// バイナリを保存（サイズ上限 4096 バイト）
    pub set_param_binary: unsafe extern "C" fn(key: LPCSTR, data: *mut c_void, size: i32),
    /// すべての保存データを削除
    pub clear_params: unsafe extern "C" fn(),
}

/// ホストアプリケーション構造体
#[repr(C)]
pub struct HOST_APP_TABLE {
    /// プラグイン情報の設定
    pub set_plugin_information: unsafe extern "C" fn(information: LPCWSTR),

    /// 入力プラグインの登録
    pub register_input_plugin: unsafe extern "C" fn(input_plugin_table: *mut INPUT_PLUGIN_TABLE),
    /// 出力プラグインの登録
    pub register_output_plugin: unsafe extern "C" fn(output_plugin_table: *mut OUTPUT_PLUGIN_TABLE),
    /// フィルタプラグインの登録
    pub register_filter_plugin: unsafe extern "C" fn(filter_plugin_table: *mut FILTER_PLUGIN_TABLE),
    /// スクリプトモジュールの登録
    pub register_script_module:
        unsafe extern "C" fn(script_module_table: *mut SCRIPT_MODULE_TABLE),

    /// インポートメニューの登録
    pub register_import_menu:
        unsafe extern "C" fn(name: LPCWSTR, func_proc_import: unsafe extern "C" fn(*mut EDIT_SECTION)),
    /// エクスポートメニューの登録
    pub register_export_menu:
        unsafe extern "C" fn(name: LPCWSTR, func_proc_export: unsafe extern "C" fn(*mut EDIT_SECTION)),

    /// ウィンドウクライアントの登録
    pub register_window_client: unsafe extern "C" fn(name: LPCWSTR, hwnd: HWND),

    /// 編集ハンドルの作成
    pub create_edit_handle: unsafe extern "C" fn() -> *mut EDIT_HANDLE,

    /// プロジェクトロード直後のハンドラ登録
    pub register_project_load_handler:
        unsafe extern "C" fn(func_project_load: unsafe extern "C" fn(*mut PROJECT_FILE)),
    /// プロジェクトセーブ直前のハンドラ登録
    pub register_project_save_handler:
        unsafe extern "C" fn(func_project_save: unsafe extern "C" fn(*mut PROJECT_FILE)),
}

