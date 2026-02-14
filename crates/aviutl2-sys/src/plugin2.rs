#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::common::LPCWSTR;
use crate::{
    filter2::FILTER_PLUGIN_TABLE, input2::INPUT_PLUGIN_TABLE, module2::SCRIPT_MODULE_TABLE,
    output2::OUTPUT_PLUGIN_TABLE,
};
use std::ffi::c_void;
use std::os::raw::c_char;

pub use windows_sys::Win32::Foundation::{HINSTANCE, HWND};

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

/// メディア情報構造体
#[repr(C)]
pub struct MEDIA_INFO {
    /// Videoトラック数 ※0ならVideo無し
    pub video_track_num: i32,
    /// Audioトラック数 ※0ならAudio無し
    pub audio_track_num: i32,
    /// 総時間 ※静止画の場合は0
    pub total_time: f64,
    /// 解像度
    pub width: i32,
    /// 解像度
    pub height: i32,
}

/// モジュール情報構造体
#[repr(C)]
pub struct MODULE_INFO {
    pub r#type: i32,
    pub name: LPCWSTR,
    pub information: LPCWSTR,
}

impl MODULE_INFO {
    /// フィルタスクリプト
    pub const TYPE_SCRIPT_FILTER: i32 = 1;
    /// オブジェクトスクリプト
    pub const TYPE_SCRIPT_OBJECT: i32 = 2;
    /// カメラスクリプト
    pub const TYPE_SCRIPT_CAMERA: i32 = 3;
    /// トラックバースクリプト
    pub const TYPE_SCRIPT_TRACK: i32 = 4;
    /// スクリプトモジュール
    pub const TYPE_SCRIPT_MODULE: i32 = 5;
    /// 入力プラグイン
    pub const TYPE_PLUGIN_INPUT: i32 = 6;
    /// 出力プラグイン
    pub const TYPE_PLUGIN_OUTPUT: i32 = 7;
    /// フィルタプラグイン
    pub const TYPE_PLUGIN_FILTER: i32 = 8;
    /// 汎用プラグイン
    pub const TYPE_PLUGIN_COMMON: i32 = 9;
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
    /// 現在の選択レイヤー番号
    pub layer: i32,
    /// オブジェクトが存在する最大のフレーム番号
    pub frame_max: i32,
    /// オブジェクトが存在する最大のレイヤー番号
    pub layer_max: i32,
    /// レイヤー編集で表示されているフレームの開始番号
    pub display_frame_start: i32,
    /// レイヤー編集で表示されているレイヤーの開始番号
    pub display_layer_start: i32,
    /// レイヤー編集で表示されているフレーム数 ※厳密ではないです
    pub display_frame_num: i32,
    /// レイヤー編集で表示されているレイヤー数 ※厳密ではないです
    pub display_layer_num: i32,
    /// フレーム範囲選択の開始フレーム番号 ※未選択の場合は-1
    pub select_range_start: i32,
    /// フレーム範囲選択の終了フレーム番号 ※未選択の場合は-1
    pub select_range_end: i32,
    /// グリッド(BPM)のテンポ
    pub grid_bpm_tempo: f32,
    /// グリッド(BPM)の拍子
    pub grid_bpm_beat: i32,
    /// グリッド(BPM)の基準時間
    pub grid_bpm_offset: f32,
    /// シーンのID
    pub scene_id: i32,
}

/// 編集セクション構造体
/// メニュー選択やプロジェクト編集のコールバック関数内で利用出来ます
/// フレーム番号、レイヤー番号が0からの番号になります ※UI表示と異なります
#[repr(C)]
pub struct EDIT_SECTION {
    /// 編集情報
    pub info: *mut EDIT_INFO,

    /// 指定の位置にオブジェクトエイリアスを作成します
    /// alias : オブジェクトエイリアスデータ(UTF-8)へのポインタ
    ///      オブジェクトエイリアスファイルと同じフォーマットになります
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数 ※エイリアスデータにフレーム情報がある場合はフレーム情報から長さが設定されます
    ///      フレーム数に0を指定した場合は長さや追加位置が自動調整されます
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    ///      既に存在するオブジェクトに重なったり、エイリアスデータが不正な場合に失敗します
    ///      複数オブジェクトのエイリアスデータの場合は先頭のオブジェクトのハンドルが返却されます ※オブジェクトは全て作成されます
    pub create_object_from_alias:
        unsafe extern "C" fn(alias: LPCSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 指定のフレーム番号以降にあるオブジェクトを検索します
    /// layer : 検索対象のレイヤー番号
    /// frame : 検索を開始するフレーム番号
    /// 戻り値 : 検索したオブジェクトのハンドル (見つからない場合はnullptrを返却)
    pub find_object: unsafe extern "C" fn(layer: i32, frame: i32) -> OBJECT_HANDLE,

    /// オブジェクトに対象エフェクトが何個存在するかを取得します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 戻り値 : 対象エフェクトの数 ※存在しない場合は0
    pub count_object_effect: unsafe extern "C" fn(object: OBJECT_HANDLE, effect: LPCWSTR) -> i32,

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
    ///          set_object_item_value(object, L"ぼかし:1", L"範囲", "1"); // 2個目のぼかしを対象とする
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

    /// プロジェクトファイルのポインタを取得します
    /// EDIT_HANDLE : 編集ハンドル
    /// 戻り値 : プロジェクトファイル構造体へのポインタ ※コールバック処理の終了まで有効
    pub get_project_file: unsafe extern "C" fn(edit: *mut EDIT_HANDLE) -> *mut PROJECT_FILE,

    /// 選択中オブジェクトのハンドルを取得します
    /// index : 選択中オブジェクトのインデックス(0〜)
    /// 戻り値 : 指定インデックスのオブジェクトのハンドル (インデックスが範囲外の場合はnullptrを返却)
    pub get_selected_object: unsafe extern "C" fn(index: i32) -> OBJECT_HANDLE,

    /// 選択中オブジェクトの数を取得します
    /// 戻り値 : 選択中オブジェクトの数
    pub get_selected_object_num: unsafe extern "C" fn() -> i32,

    /// マウス座標のレイヤー・フレーム位置を取得します
    /// 最後のマウス移動のウィンドウメッセージの座標から計算します
    /// layer : レイヤー番号の格納先
    /// frame : フレーム番号の格納先
    /// 戻り値 : マウス座標がレイヤー編集上の場合はtrue
    pub get_mouse_layer_frame: unsafe extern "C" fn(layer: *mut i32, frame: *mut i32) -> bool,

    /// 指定のスクリーン座標のレイヤー・フレーム位置を取得します
    /// x,y   : 対象のスクリーン座標
    /// layer : レイヤー番号の格納先
    /// frame : フレーム番号の格納先
    /// 戻り値 : スクリーン座標がレイヤー編集上の場合はtrue
    pub pos_to_layer_frame:
        unsafe extern "C" fn(x: i32, y: i32, layer: *mut i32, frame: *mut i32) -> bool,

    /// 指定のメディアファイルがサポートされているかを確認します
    /// file   : メディアファイルのパス
    /// strict : trueの場合は実際に読み込めるかを確認します
    ///      falseの場合は拡張子が対応しているかを確認します
    /// 戻り値 : サポートされている場合はtrue
    pub is_support_media_file: unsafe extern "C" fn(file: LPCWSTR, strict: bool) -> bool,

    /// 指定のメディアファイルの情報を取得します ※動画、音声、画像ファイル以外では取得出来ません
    /// file : メディアファイルのパス
    /// info : メディア情報の格納先へのポインタ
    /// info_size : メディア情報の格納先のサイズ ※MEDIA_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue
    pub get_media_info:
        unsafe extern "C" fn(file: LPCWSTR, info: *mut MEDIA_INFO, info_size: i32) -> bool,

    /// 指定の位置にメディアファイルからオブジェクトを作成します
    /// file  : メディアファイルのパス
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数
    ///      フレーム数に0を指定した場合は長さや追加位置が自動調整されます
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    ///      既に存在するオブジェクトに重なったり、メディアファイルに対応していない場合は失敗します
    pub create_object_from_media_file:
        unsafe extern "C" fn(file: LPCWSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 指定の位置にオブジェクトを作成します
    /// effect : エフェクト名 (エイリアスファイルのeffect.nameの値)
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数
    ///      フレーム数に0を指定した場合は長さや追加位置が自動調整されます
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    ///      既に存在するオブジェクトに重なったり、指定エフェクトに対応していない場合は失敗します
    pub create_object:
        unsafe extern "C" fn(effect: LPCWSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 現在のレイヤー・フレーム位置を設定します ※設定出来る範囲に調整されます
    /// layer : レイヤー番号
    /// frame : フレーム番号
    pub set_cursor_layer_frame: unsafe extern "C" fn(layer: i32, frame: i32),

    /// レイヤー編集のレイヤー・フレームの表示開始位置を設定します ※設定出来る範囲に調整されます
    /// layer : 表示開始レイヤー番号
    /// frame : 表示開始フレーム番号
    pub set_display_layer_frame: unsafe extern "C" fn(layer: i32, frame: i32),

    /// フレーム範囲選択を設定します ※設定出来る範囲に調整されます
    /// start,end : 開始終了フレーム番号
    ///      開始終了フレームの両方に-1を指定すると選択を解除します
    pub set_select_range: unsafe extern "C" fn(start: i32, end: i32),

    /// グリッド(BPM)を設定します
    /// tempo : テンポ
    /// beat  : 拍子
    /// offset : 基準時間
    pub set_grid_bpm: unsafe extern "C" fn(tempo: f32, beat: i32, offset: f32),

    /// オブジェクト名を取得します
    /// object : オブジェクトのハンドル
    /// 戻り値 : オブジェクト名へのポインタ (標準の名前の場合はnullptrを返却)
    ///      ※オブジェクトの編集をするかコールバック処理の終了まで有効
    pub get_object_name: unsafe extern "C" fn(object: OBJECT_HANDLE) -> LPCWSTR,

    /// オブジェクト名を設定します
    /// object : オブジェクトのハンドル
    /// name : オブジェクト名 (nullptrか空文字を指定すると標準の名前になります)
    pub set_object_name: unsafe extern "C" fn(object: OBJECT_HANDLE, name: LPCWSTR),

    /// レイヤー名を取得します
    /// layer : レイヤー番号
    /// 戻り値 : レイヤー名へのポインタ (標準の名前の場合はnullptrを返却)
    ///     ※レイヤーの編集をするかコールバック処理の終了まで有効
    pub get_layer_name: unsafe extern "C" fn(layer: i32) -> LPCWSTR,

    /// レイヤー名を設定します
    /// layer : レイヤー番号
    /// name : レイヤー名 (nullptrか空文字を指定すると標準の名前になります)
    pub set_layer_name: unsafe extern "C" fn(layer: i32, name: LPCWSTR),

    /// シーン名を取得します
    /// 戻り値 : シーン名へのポインタ
    ///     ※シーンの編集をするかコールバック処理の終了まで有効
    pub get_scene_name: unsafe extern "C" fn() -> LPCWSTR,

    /// シーン名を設定します ※シーンの操作は現状Undoに非対応です
    /// name : シーン名 (nullptrや空文字の場合は変更しません)
    pub set_scene_name: unsafe extern "C" fn(name: LPCWSTR),

    /// シーンの解像度を設定します ※シーンの操作は現状Undoに非対応です
    /// width : 横のサイズ
    /// height : 縦のサイズ
    pub set_scene_size: unsafe extern "C" fn(width: i32, height: i32),

    /// シーンのフレームレートを設定します ※シーンの操作は現状Undoに非対応です
    /// rate : フレームレート
    /// scale : フレームレートのスケール
    pub set_scene_frame_rate: unsafe extern "C" fn(rate: i32, scale: i32),

    /// シーンのサンプリングレートを設定します ※シーンの操作は現状Undoに非対応です
    /// sample_rate : サンプリングレート
    pub set_scene_sample_rate: unsafe extern "C" fn(sample_rate: i32),
}

/// 編集ハンドル構造体
#[repr(C)]
pub struct EDIT_HANDLE {
    /// プロジェクトデータの編集をする為のコールバック関数(func_proc_edit)を呼び出します
    /// 編集情報を排他制御する為に更新ロック状態のコールバック関数内で編集処理をする形になります
    /// コールバック関数内で編集したオブジェクトは纏めてUndoに登録されます
    /// コールバック関数はメインスレッドから呼ばれます
    /// func_proc_edit : 編集処理のコールバック関数
    /// 戻り値   : trueなら成功
    ///        編集が出来ない場合(出力中等)に失敗します
    pub call_edit_section:
        unsafe extern "C" fn(func_proc_edit: unsafe extern "C" fn(edit: *mut EDIT_SECTION)) -> bool,

    /// call_edit_section()に引数paramを渡せるようにした関数です
    pub call_edit_section_param: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_edit: unsafe extern "C" fn(param: *mut c_void, edit: *mut EDIT_SECTION),
    ) -> bool,

    /// 編集情報を取得します
    /// 編集情報を排他制御する為に参照ロックします。※同一スレッドで既にロック状態の場合はそのまま取得します。
    /// info : 編集情報の格納先へのポインタ
    /// info_size : 編集情報の格納先のサイズ ※EDIT_INFOと異なる場合はサイズ分のみ取得されます
    pub get_edit_info: unsafe extern "C" fn(info: *mut EDIT_INFO, info_size: i32),

    /// ホストアプリケーションを再起動します
    pub restart_host_app: unsafe extern "C" fn(),

    /// エフェクト名の一覧をコールバック関数（func_proc_enum_effect）で取得します
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_effect : エフェクト名の取得処理のコールバック関数
    pub enum_effect_name: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_enum_effect: unsafe extern "C" fn(
            param: *mut c_void,
            name: LPCWSTR,
            r#type: i32,
            flag: i32,
        ),
    ),

    /// モジュール情報の一覧をコールバック関数（func_proc_enum_module）で取得します
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_module : モジュール情報の取得処理のコールバック関数
    pub enum_module_info: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_enum_module: unsafe extern "C" fn(param: *mut c_void, info: *mut MODULE_INFO),
    ),

    /// ホストアプリケーションのメインウィンドウのハンドルを取得します
    pub get_host_app_window: unsafe extern "C" fn() -> HWND,
}

impl EDIT_HANDLE {
    /// エフェクト種別：フィルタ効果 ※今後追加される可能性があります
    pub const EFFECT_TYPE_FILTER: i32 = 1;
    /// エフェクト種別：メディア入力 ※今後追加される可能性があります
    pub const EFFECT_TYPE_INPUT: i32 = 2;
    /// エフェクト種別：シーンチェンジ ※今後追加される可能性があります
    pub const EFFECT_TYPE_TRANSITION: i32 = 3;
    /// エフェクトフラグ：画像をサポート ※今後追加される可能性があります
    pub const EFFECT_FLAG_VIDEO: i32 = 1;
    /// エフェクトフラグ：音声をサポート ※今後追加される可能性があります
    pub const EFFECT_FLAG_AUDIO: i32 = 2;
    /// エフェクトフラグ：フィルタオブジェクトをサポート ※今後追加される可能性があります
    pub const EFFECT_FLAG_FILTER: i32 = 4;
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

    /// プロジェクトファイルのパスを取得します
    /// 戻り値 : プロジェクトファイルパスへのポインタ (ファイルパスは未設定の場合があります)
    ///      ※コールバック処理の終了まで有効
    pub get_project_file_path: unsafe extern "C" fn() -> LPCWSTR,
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

    /// レイヤーメニューを登録する (レイヤー編集でオブジェクト未選択時の右クリックメニューに追加されます)
    /// name : レイヤーメニューの名称
    /// func_proc_layer_menu : レイヤーメニュー選択時のコールバック関数
    pub register_layer_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_layer_menu: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),

    /// オブジェクトメニューを登録する (レイヤー編集でオブジェクト選択時の右クリックメニューに追加されます)
    /// name : オブジェクトメニューの名称
    /// func_proc_object_menu : オブジェクトメニュー選択時のコールバック関数
    pub register_object_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_object_menu: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),

    /// 設定メニューを登録する
    /// 設定メニューの登録後にウィンドウクライアントを登録するとシステムメニューに「設定」が追加されます
    /// name : 設定メニューの名称
    /// func_config : 設定メニュー選択時のコールバック関数
    pub register_config_menu:
        unsafe extern "C" fn(name: LPCWSTR, func_config: unsafe extern "C" fn(HWND, HINSTANCE)),

    /// 編集メニューを登録する
    /// name : 編集メニューの名称 ※名称に'\'を入れると表示を階層に出来ます
    /// func_proc_edit_menu : 編集メニュー選択時のコールバック関数
    pub register_edit_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_edit_menu: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),

    /// キャッシュを破棄の操作時に呼ばれる関数を登録する
    /// func_proc_clear_cache : キャッシュの破棄時のコールバック関数
    pub register_clear_cache_handler:
        unsafe extern "C" fn(func_proc_clear_cache: unsafe extern "C" fn(*mut EDIT_SECTION)),

    /// シーンを変更した直後に呼ばれる関数を登録する ※シーンの設定情報が更新された時にも呼ばれます
    /// func_proc_change_scene : シーン変更時のコールバック関数
    pub register_change_scene_handler:
        unsafe extern "C" fn(func_proc_change_scene: unsafe extern "C" fn(*mut EDIT_SECTION)),

    /// インポートメニューを登録する (ウィンドウメニューのファイルに追加されます)
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : インポートメニューの名称
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_import : インポートメニュー選択時のコールバック関数
    pub register_import_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        param: *mut c_void,
        func_proc_import: unsafe extern "C" fn(param: *mut c_void),
    ),

    /// エクスポートメニューを登録する (ウィンドウメニューのファイルに追加されます)
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : エクスポートメニューの名称
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_export : エクスポートメニュー選択時のコールバック関数
    pub register_export_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        param: *mut c_void,
        func_proc_export: unsafe extern "C" fn(param: *mut c_void),
    ),

    /// レイヤーメニューを登録する (レイヤー編集でオブジェクト未選択時の右クリックメニューに追加されます)
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : レイヤーメニューの名称
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_layer_menu : レイヤーメニュー選択時のコールバック関数
    pub register_layer_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        param: *mut c_void,
        func_proc_layer_menu: unsafe extern "C" fn(param: *mut c_void),
    ),

    /// オブジェクトメニューを登録する (レイヤー編集でオブジェクト選択時の右クリックメニューに追加されます)
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : オブジェクトメニューの名称
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_object_menu : オブジェクトメニュー選択時のコールバック関数
    pub register_object_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        param: *mut c_void,
        func_proc_object_menu: unsafe extern "C" fn(param: *mut c_void),
    ),

    /// 編集メニューを登録する
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : 編集メニューの名称 ※名称に'\'を入れると表示を階層に出来ます
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_edit_menu : 編集メニュー選択時のコールバック関数
    pub register_edit_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        param: *mut c_void,
        func_proc_edit_menu: unsafe extern "C" fn(param: *mut c_void),
    ),
}
