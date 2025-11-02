#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::common::LPCWSTR;
use crate::{
    filter2::FILTER_PLUGIN_TABLE, input2::INPUT_PLUGIN_TABLE, module2::SCRIPT_MODULE_TABLE,
    output2::OUTPUT_PLUGIN_TABLE,
};
use std::ffi::c_void;
use std::os::raw::c_char;

pub use windows_sys::Win32::Foundation::HWND;

pub type LPCSTR = *const c_char;

/// オブジェクトハンドル
pub type OBJECT_HANDLE = *mut c_void;

/// レイヤー・フレーム情報構造体
/// フレーム番号、レイヤー番号が0からの番号になります ※UI表示と異なります
#[repr(C)]
pub struct OBJECT_LAYER_FRAME {
    /// レイヤー番号
    pub layer: i32,
    /// 開始フレーム番号
    pub start: i32,
    /// 終了フレーム番号
    pub end: i32,
}

/// 冗長なので後で廃止します
#[repr(C)]
pub struct DEPRECATED_OBJECT_FRAME_INFO {
    pub start: i32,
    pub end: i32,
}

/// 編集情報構造体
/// フレーム番号、レイヤー番号が0からの番号になります ※UI表示と異なります
#[repr(C)]
pub struct EDIT_INFO {
    /// シーンの解像度
    pub width: i32,
    /// シーンの解像度
    pub height: i32,
    /// シーンのフレームレート
    pub rate: i32,
    /// シーンのフレームレート
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

/// 編集セクション構造体
/// メニュー選択やプロジェクト編集のコールバック関数内で利用出来ます
/// フレーム番号、レイヤー番号が0からの番号になります ※UI表示と異なります
#[repr(C)]
pub struct EDIT_SECTION {
    /// 編集情報
    pub info: *const EDIT_INFO,

    /// 指定の位置にオブジェクトエイリアスを作成します
    /// alias : オブジェクトエイリアスデータ(UTF-8)へのポインタ
    ///      オブジェクトエイリアスファイルと同じフォーマットになります
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数 ※エイリアスデータにフレーム情報が無い場合に利用します
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    ///      既に存在するオブジェクトに重なったり、エイリアスデータが不正な場合に失敗します
    pub create_object_from_alias:
        unsafe extern "C" fn(alias: LPCSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 指定のフレーム番号以降にあるオブジェクトを検索します
    /// layer : 検索対象のレイヤー番号
    /// frame : 検索を開始するフレーム番号
    /// 戻り値 : 検索したオブジェクトのハンドル (見つからない場合はnullptrを返却)
    pub find_object: unsafe extern "C" fn(layer: i32, frame: i32) -> OBJECT_HANDLE,

    /// 冗長なので後で廃止します
    pub deprecated_get_object_frame_info:
        unsafe extern "C" fn(object: OBJECT_HANDLE) -> DEPRECATED_OBJECT_FRAME_INFO,

    /// オブジェクトのレイヤー・フレーム情報を取得します
    /// object : オブジェクトのハンドル
    /// 戻り値 : オブジェクトのレイヤー・フレーム情報
    pub get_object_layer_frame: unsafe extern "C" fn(object: OBJECT_HANDLE) -> OBJECT_LAYER_FRAME,

    /// オブジェクトのエイリアスデータを取得します
    /// object : オブジェクトのハンドル
    /// 戻り値 : オブジェクトエイリアスデータ(UTF-8)へのポインタ (取得出来ない場合はnullptrを返却)
    ///      オブジェクトエイリアスファイルと同じフォーマットになります
    ///      ※次に文字列返却の関数を使うかコールバック処理の終了まで有効
    pub get_object_alias: unsafe extern "C" fn(object: OBJECT_HANDLE) -> LPCSTR,

    /// オブジェクトの設定項目の値を文字列で取得します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    ///          同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    ///          get_object_item_value(object, L"ぼかし:1", L"範囲"); // 2個目のぼかしを対象とする
    /// item  : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// 戻り値 : 取得した設定値(UTF8)へのポインタ (取得出来ない場合はnullptrを返却)
    ///      エイリアスファイルの設定値と同じフォーマットになります
    ///      ※次に文字列返却の関数を使うかコールバック処理の終了まで有効
    pub get_object_item_value:
        unsafe extern "C" fn(object: OBJECT_HANDLE, effect: LPCWSTR, item: LPCWSTR) -> LPCSTR,

    /// オブジェクトの設定項目の値を文字列で設定します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    ///          同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    ///          get_object_item_value(object, L"ぼかし:1", L"範囲"); // 2個目のぼかしを対象とする
    /// item  : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// value : 設定値(UTF8)
    ///      エイリアスファイルの設定値と同じフォーマットになります
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub set_object_item_value: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        item: LPCWSTR,
        value: LPCSTR,
    ) -> bool,

    /// オブジェクトを移動します
    /// object : オブジェクトのハンドル
    /// layer : 移動先のレイヤー番号
    /// frame : 移動先のフレーム番号
    /// 戻り値 : 移動した場合はtrue (移動先にオブジェクトが存在する場合は失敗します)
    pub move_object: unsafe extern "C" fn(object: OBJECT_HANDLE, layer: i32, frame: i32) -> bool,

    /// オブジェクトを削除します
    /// object : オブジェクトのハンドル
    pub delete_object: unsafe extern "C" fn(object: OBJECT_HANDLE),

    /// オブジェクト設定ウィンドウで選択されているオブジェクトのハンドルを取得します
    /// 戻り値 : オブジェクトのハンドル (未選択の場合はnullptrを返却)
    pub get_focus_object: unsafe extern "C" fn() -> OBJECT_HANDLE,

    /// オブジェクト設定ウィンドウで選択するオブジェクトを設定します (コールバック処理の終了時に設定されます)
    /// object : オブジェクトのハンドル
    pub set_focus_object: unsafe extern "C" fn(object: OBJECT_HANDLE),

    /// アプリケーションのログを出力します
    /// message : ログメッセージ
    pub output_log: unsafe extern "C" fn(message: LPCWSTR),
}

/// 編集ハンドル構造体
#[repr(C)]
pub struct EDIT_HANDLE {
    /// プロジェクトデータの編集をする為のコールバック関数(func_proc_edit)を呼び出します
    /// 編集情報を排他制御する為にコールバック関数内で編集処理をする形になります
    /// コールバック関数内で編集したオブジェクトは纏めてUndoに登録されます
    /// コールバック関数はメインスレッドから呼ばれます
    /// func_proc_edit : 編集処理のコールバック関数
    /// 戻り値   : trueなら成功
    ///        編集が出来ない場合(出力中等)に失敗します
    pub call_edit_section:
        unsafe extern "C" fn(func_proc_edit: unsafe extern "C" fn(edit: *mut EDIT_SECTION)) -> bool,
}

/// プロジェクトファイル構造体
/// プロジェクトファイルのロード、セーブ時のコールバック関数内で利用出来ます
/// プロジェクトの保存データはプラグイン毎のデータ領域になります
#[repr(C)]
pub struct PROJECT_FILE {
    /// プロジェクトに保存されている文字列(UTF-8)を取得します
    /// key  : キー名(UTF-8)
    /// 戻り値 : 取得した文字列へのポインタ (未設定の場合はnullptr)
    pub get_param_string: unsafe extern "C" fn(key: LPCSTR) -> LPCSTR,
    /// プロジェクトに文字列(UTF-8)を保存します
    /// key  : キー名(UTF-8)
    /// value : 保存する文字列(UTF-8)
    pub set_param_string: unsafe extern "C" fn(key: LPCSTR, value: LPCSTR),
    /// プロジェクトに保存されているバイナリデータを取得します
    /// key  : キー名(UTF-8)
    /// data  : 取得するデータの格納先へのポインタ
    /// size  : 取得するデータのサイズ (保存されているサイズと異なる場合は失敗します)
    /// 戻り値 : 正しく取得出来た場合はtrue
    pub get_param_binary: unsafe extern "C" fn(key: LPCSTR, data: *mut c_void, size: i32) -> bool,
    /// プロジェクトにバイナリデータを保存します
    /// key  : キー名(UTF-8)
    /// data  : 保存するデータへのポインタ
    /// size  : 保存するデータのサイズ (4096バイト以下)
    pub set_param_binary: unsafe extern "C" fn(key: LPCSTR, data: *mut c_void, size: i32),
    /// プロジェクトに保存されているデータを全て削除します
    pub clear_params: unsafe extern "C" fn(),
}

/// ホストアプリケーション構造体
#[repr(C)]
pub struct HOST_APP_TABLE {
    /// プラグインの情報を設定する
    /// information : プラグインの情報
    pub set_plugin_information: unsafe extern "C" fn(information: LPCWSTR),

    /// 入力プラグインを登録する
    /// input_plugin_table : 入力プラグイン構造体
    pub register_input_plugin: unsafe extern "C" fn(input_plugin_table: *mut INPUT_PLUGIN_TABLE),
    /// 出力プラグインを登録する
    /// output_plugin_table : 出力プラグイン構造体
    pub register_output_plugin: unsafe extern "C" fn(output_plugin_table: *mut OUTPUT_PLUGIN_TABLE),
    /// フィルタプラグインを登録する
    /// filter_plugin_table : フィルタプラグイン構造体
    pub register_filter_plugin: unsafe extern "C" fn(filter_plugin_table: *mut FILTER_PLUGIN_TABLE),
    /// スクリプトモジュールを登録する
    /// script_module_table : スクリプトモジュール構造体
    pub register_script_module: unsafe extern "C" fn(script_module_table: *mut SCRIPT_MODULE_TABLE),

    /// インポートメニューを登録する
    /// name    : インポートメニューの名称
    /// func_proc_import : インポートメニュー選択時のコールバック関数
    pub register_import_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_import: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),
    /// エクスポートメニューを登録する
    /// name    : エクスポートメニューの名称
    /// func_proc_export : エクスポートメニュー選択時のコールバック関数
    pub register_export_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_export: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),

    /// ウィンドウクライアントを登録する
    /// name  : ウィンドウの名称
    /// hwnd  : ウィンドウハンドル
    /// ウィンドウにはWS_CHILDが追加され親ウィンドウが設定されます ※WS_POPUPは削除されます
    pub register_window_client: unsafe extern "C" fn(name: LPCWSTR, hwnd: HWND),

    /// プロジェクトデータ編集用のハンドルを取得します
    /// 戻り値 : 編集ハンドル
    pub create_edit_handle: unsafe extern "C" fn() -> *mut EDIT_HANDLE,

    /// プロジェクトファイルをロードした直後に呼ばれる関数を登録する ※プロジェクトの初期化時にも呼ばれます
    /// func_project_load : プロジェクトファイルのロード時のコールバック関数
    pub register_project_load_handler:
        unsafe extern "C" fn(func_project_load: unsafe extern "C" fn(*mut PROJECT_FILE)),
    /// プロジェクトファイルをセーブする直前に呼ばれる関数を登録する
    /// func_project_save : プロジェクトファイルのセーブ時のコールバック関数
    pub register_project_save_handler:
        unsafe extern "C" fn(func_project_save: unsafe extern "C" fn(*mut PROJECT_FILE)),
}
