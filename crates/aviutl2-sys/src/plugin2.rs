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

/// 汎用プラグイン構造体
#[repr(C)]
pub struct COMMON_PLUGIN_TABLE {
    /// プラグインの名前
    pub name: LPCWSTR,
    /// プラグインの情報
    pub information: LPCWSTR,
}

/// オブジェクトハンドル
pub type OBJECT_HANDLE = *mut c_void;

/// エフェクトハンドル
pub type EFFECT_HANDLE = *mut c_void;

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

/// トラックバー情報構造体
#[repr(C)]
pub struct TRACK_INFO {
    /// トラックバーの移動モードの名称 ※移動無しの場合はnullptr
    pub mode: LPCWSTR,
    /// トラックバーの設定値の配列へのポインタ ※設定値が無い場合はnullptr
    pub param: *mut f64,
    /// トラックバーの設定値の数
    pub param_num: i32,
    /// トラックバーの加速度が有効か？
    pub accelerate: bool,
    /// トラックバーの減速度が有効か？
    pub decelerate: bool,
    /// トラックバーの中間点無視が有効か？
    pub twopoint: bool,
    /// トラックバーの時間制御が有効か？
    pub timecontrol: bool,
    /// 所属グループのトラックバーの数 ※グループ化されていない場合は1
    pub group_num: i32,
    /// 所属グループ内のインデックス
    pub group_index: i32,
    /// 所属グループの名称 ※グループ化されていない場合はnullptr
    pub group_name: LPCWSTR,
}

/// パレット情報構造体
#[repr(C)]
pub struct PALETTE_INFO {
    pub color: [PALETTE_INFO_COLOR; Self::PALETTE_NUM],
}

impl PALETTE_INFO {
    pub const PALETTE_NUM: usize = 64;
}

/// パレット色情報構造体
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PALETTE_INFO_COLOR {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// BPM情報構造体
#[repr(C)]
pub struct BPM_INFO {
    /// テンポ
    pub tempo: f32,
    /// 拍子
    pub beat: i32,
    /// 開始位置(秒)
    pub start: f64,
    /// 拍子オフセット(秒)
    pub offset: f32,
}

/// イベント種別
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EVENT_TYPE {
    /// オブジェクト情報の更新
    UPDATE_OBJECT = 1,
    /// 現在の編集フレームの移動
    CHANGE_EDIT_FRAME = 2,
    /// 現在の編集シーンの変更 ※シーン情報の更新も含まれる
    CHANGE_EDIT_SCENE = 3,
    /// 選択されているオブジェクトの変更
    CHANGE_FOCUS_OBJECT = 4,
    /// 編集状態の変更(プレビュー再生・ファイル出力の開始終了時)
    CHANGE_EDIT_STATE = 5,
}

/// オブジェクトフラグ種別
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OBJECT_FLAG_TYPE {
    /// グループ制御対象の有効・無効
    ENABLE_GROUP = 1,
    /// カメラ制御対象の有効・無効
    ENABLE_CAMERA = 2,
    /// クリッピングオブジェクトの有効・無効
    CLIPPING_OBJECT = 3,
    /// 上のオブジェクトでクリッピングの有効・無効
    CLIPPING_UPPER_OBJECT = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EDIT_INFO_COLOR {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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
    /// グリッド(BPM)のテンポ ※先頭のBPM情報
    pub grid_bpm_tempo: f32,
    /// グリッド(BPM)の拍子 ※先頭のBPM情報
    pub grid_bpm_beat: i32,
    /// グリッド(BPM)の拍子オフセット ※先頭のBPM情報
    pub grid_bpm_offset: f32,
    /// シーンのID
    pub scene_id: i32,
    /// シーンの背景色
    pub background: EDIT_INFO_COLOR,
}

/// 編集セクション構造体
/// メニュー選択やプロジェクト編集のコールバック関数内で利用出来ます
/// フレーム番号、レイヤー番号が0からの番号になります ※UI表示と異なります
#[repr(C)]
pub struct EDIT_SECTION {
    /// 編集情報 (call_read_section利用不可)
    pub info: *mut EDIT_INFO,

    /// 指定の位置にオブジェクトエイリアスを作成します (call_read_section利用不可)
    /// alias : オブジェクトエイリアスデータ(UTF-8)へのポインタ
    /// オブジェクトエイリアスファイル(.object)と同じフォーマットになります
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数 ※エイリアスデータにフレーム情報がある場合はフレーム情報から長さが設定されます
    /// フレーム数に0を指定した場合は長さや追加位置が自動調整されます
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    /// 既に存在するオブジェクトに重なったり、エイリアスデータが不正な場合に失敗します
    /// 複数オブジェクトのエイリアスデータの場合は先頭のオブジェクトのハンドルが返却されます ※オブジェクトは全て作成されます
    pub create_object_from_alias:
        unsafe extern "C" fn(alias: LPCSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 指定のフレーム番号以降にあるオブジェクトを検索します
    /// ※フィルタプラグインから呼び出した場合は処理対象のシーンのオブジェクトを検索します
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
    /// オブジェクトエイリアスファイルと同じフォーマットになります
    /// ※次に同一スレッドで文字列返却の関数を使うかコールバック処理の終了まで有効
    pub get_object_alias: unsafe extern "C" fn(object: OBJECT_HANDLE) -> LPCSTR,

    /// オブジェクトの設定項目の値を文字列で取得します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// get_object_item_value(object, L"ぼかし:1", L"範囲"); // 2個目のぼかしを対象とする
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// 戻り値 : 取得した設定値(UTF8)へのポインタ (取得出来ない場合はnullptrを返却)
    /// エイリアスファイルの設定値と同じフォーマットになります
    /// ※次に同一スレッドで文字列返却の関数を使うかコールバック処理の終了まで有効
    pub get_object_item_value:
        unsafe extern "C" fn(object: OBJECT_HANDLE, effect: LPCWSTR, item: LPCWSTR) -> LPCSTR,

    /// オブジェクトの設定項目の値を文字列で設定します (call_read_section利用不可)
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// set_object_item_value(object, L"ぼかし:1", L"範囲", "1"); // 2個目のぼかしを対象とする
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// value : 設定値(UTF8)
    /// エイリアスファイルの設定値と同じフォーマットになります
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub set_object_item_value: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        item: LPCWSTR,
        value: LPCSTR,
    ) -> bool,

    /// オブジェクトを移動します (call_read_section利用不可)
    /// object : オブジェクトのハンドル
    /// layer : 移動先のレイヤー番号
    /// frame : 移動先のフレーム番号
    /// 戻り値 : 移動した場合はtrue (移動先にオブジェクトが存在する場合は失敗します)
    pub move_object: unsafe extern "C" fn(object: OBJECT_HANDLE, layer: i32, frame: i32) -> bool,

    /// オブジェクトを削除します (call_read_section利用不可)
    /// ※同一編集セクション内で作成したオブジェクトは削除しないようにすること
    /// object : オブジェクトのハンドル
    pub delete_object: unsafe extern "C" fn(object: OBJECT_HANDLE),

    /// オブジェクト設定ウィンドウで選択されているオブジェクトのハンドルを取得します
    /// 戻り値 : オブジェクトのハンドル (未選択の場合はnullptrを返却)
    pub get_focus_object: unsafe extern "C" fn() -> OBJECT_HANDLE,

    /// オブジェクト設定ウィンドウで選択するオブジェクトを設定します (call_read_section利用不可)
    /// ※コールバック処理の終了時に設定されます
    /// object : オブジェクトのハンドル (nullptrを指定すると選択を解除します)
    pub set_focus_object: unsafe extern "C" fn(object: OBJECT_HANDLE),

    /// プロジェクトファイルのポインタを取得します (call_read_section利用不可)
    /// EDIT_HANDLE : 編集ハンドル
    /// 戻り値 : プロジェクトファイル構造体へのポインタ
    /// ※コールバック処理の終了まで有効
    pub get_project_file: unsafe extern "C" fn(edit: *mut EDIT_HANDLE) -> *mut PROJECT_FILE,

    /// 選択中オブジェクトのハンドルを取得します
    /// index : 選択中オブジェクトのインデックス(0〜)
    /// 戻り値 : 指定インデックスのオブジェクトのハンドル (インデックスが範囲外の場合はnullptrを返却)
    pub get_selected_object: unsafe extern "C" fn(index: i32) -> OBJECT_HANDLE,

    /// 選択中オブジェクトの数を取得します
    /// 戻り値 : 選択中オブジェクトの数
    pub get_selected_object_num: unsafe extern "C" fn() -> i32,

    /// マウス座標のレイヤー・フレーム位置を取得します (call_read_section利用不可)
    /// 最後のマウス移動のウィンドウメッセージの座標から計算します
    /// ファイルD&D時のコールバック関数内で取得した場合はドロップ位置になります
    /// layer : レイヤー番号の格納先
    /// frame : フレーム番号の格納先
    /// 戻り値 : マウス座標がレイヤー編集上の場合はtrue
    pub get_mouse_layer_frame: unsafe extern "C" fn(layer: *mut i32, frame: *mut i32) -> bool,

    /// 指定のスクリーン座標のレイヤー・フレーム位置を取得します (call_read_section利用不可)
    /// x,y : 対象のスクリーン座標
    /// layer : レイヤー番号の格納先
    /// frame : フレーム番号の格納先
    /// 戻り値 : スクリーン座標がレイヤー編集上の場合はtrue
    pub pos_to_layer_frame:
        unsafe extern "C" fn(x: i32, y: i32, layer: *mut i32, frame: *mut i32) -> bool,

    /// 指定のメディアファイルがサポートされているかを確認します
    /// file : メディアファイルのパス
    /// strict : trueの場合は実際に読み込めるかを確認します
    /// falseの場合は拡張子が対応しているかを確認します
    /// 戻り値 : サポートされている場合はtrue
    pub is_support_media_file: unsafe extern "C" fn(file: LPCWSTR, strict: bool) -> bool,

    /// 指定のメディアファイルの情報を取得します ※動画、音声、画像ファイル以外では取得出来ません
    /// file : メディアファイルのパス
    /// info : メディア情報の格納先へのポインタ
    /// info_size : メディア情報の格納先のサイズ ※MEDIA_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue
    pub get_media_info:
        unsafe extern "C" fn(file: LPCWSTR, info: *mut MEDIA_INFO, info_size: i32) -> bool,

    /// 指定の位置にメディアファイルからオブジェクトを作成します (call_read_section利用不可)
    /// file : メディアファイルのパス
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数
    /// フレーム数に0を指定した場合は長さや追加位置が自動調整されます
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    /// 既に存在するオブジェクトに重なったり、メディアファイルに対応していない場合は失敗します
    pub create_object_from_media_file:
        unsafe extern "C" fn(file: LPCWSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 指定の位置にオブジェクトを作成します (call_read_section利用不可)
    /// effect : エフェクト名 (エイリアスファイルのeffect.nameの値)
    /// layer : 作成するレイヤー番号
    /// frame : 作成するフレーム番号
    /// length : オブジェクトのフレーム数
    /// フレーム数に0を指定した場合は長さや追加位置が自動調整されます
    /// 戻り値 : 作成したオブジェクトのハンドル (失敗した場合はnullptrを返却)
    /// 既に存在するオブジェクトに重なったり、指定エフェクトに対応していない場合は失敗します
    pub create_object:
        unsafe extern "C" fn(effect: LPCWSTR, layer: i32, frame: i32, length: i32) -> OBJECT_HANDLE,

    /// 現在のレイヤー・フレーム位置を設定します (call_read_section利用不可)
    /// ※設定出来る範囲に調整されます
    /// layer : レイヤー番号
    /// frame : フレーム番号
    pub set_cursor_layer_frame: unsafe extern "C" fn(layer: i32, frame: i32),

    /// レイヤー編集のレイヤー・フレームの表示開始位置を設定します (call_read_section利用不可)
    /// ※設定出来る範囲に調整されます
    /// layer : 表示開始レイヤー番号
    /// frame : 表示開始フレーム番号
    pub set_display_layer_frame: unsafe extern "C" fn(layer: i32, frame: i32),

    /// フレーム範囲選択を設定します (call_read_section利用不可)
    /// ※設定出来る範囲に調整されます
    /// start,end : 開始終了フレーム番号
    /// 開始終了フレームの両方に-1を指定すると選択を解除します
    pub set_select_range: unsafe extern "C" fn(start: i32, end: i32),

    /// グリッド(BPM)を設定します (call_read_section利用不可)
    /// tempo : テンポ
    /// beat : 拍子
    /// offset : 基準時間
    pub set_grid_bpm: unsafe extern "C" fn(tempo: f32, beat: i32, offset: f32),

    /// オブジェクト名を取得します
    /// object : オブジェクトのハンドル
    /// 戻り値 : オブジェクト名へのポインタ (標準の名前の場合はnullptrを返却)
    /// ※オブジェクトの編集をするかコールバック処理の終了まで有効
    pub get_object_name: unsafe extern "C" fn(object: OBJECT_HANDLE) -> LPCWSTR,

    /// オブジェクト名を設定します (call_read_section利用不可)
    /// object : オブジェクトのハンドル
    /// name : オブジェクト名 (nullptrか空文字を指定すると標準の名前になります)
    pub set_object_name: unsafe extern "C" fn(object: OBJECT_HANDLE, name: LPCWSTR),

    /// レイヤー名を取得します
    /// layer : レイヤー番号
    /// 戻り値 : レイヤー名へのポインタ (標準の名前の場合はnullptrを返却)
    /// ※レイヤーの編集をするかコールバック処理の終了まで有効
    pub get_layer_name: unsafe extern "C" fn(layer: i32) -> LPCWSTR,

    /// レイヤー名を設定します (call_read_section利用不可)
    /// layer : レイヤー番号
    /// name : レイヤー名 (nullptrか空文字を指定すると標準の名前になります)
    pub set_layer_name: unsafe extern "C" fn(layer: i32, name: LPCWSTR),

    /// シーン名を取得します
    /// 戻り値 : シーン名へのポインタ
    /// ※シーンの編集をするかコールバック処理の終了まで有効
    pub get_scene_name: unsafe extern "C" fn() -> LPCWSTR,

    /// シーン名を設定します (call_read_section利用不可)
    /// ※シーンの操作は現状Undoに非対応です
    /// name : シーン名
    /// ※シーン名は必須になります (nullptrや空文字の場合は変更しません)
    pub set_scene_name: unsafe extern "C" fn(name: LPCWSTR),

    /// シーンの解像度を設定します (call_read_section利用不可)
    /// ※シーンの操作は現状Undoに非対応です
    /// width : 横のサイズ
    /// height : 縦のサイズ
    pub set_scene_size: unsafe extern "C" fn(width: i32, height: i32),

    /// シーンのフレームレートを設定します (call_read_section利用不可)
    /// ※シーンの操作は現状Undoに非対応です
    /// rate : フレームレート
    /// scale : フレームレートのスケール
    pub set_scene_frame_rate: unsafe extern "C" fn(rate: i32, scale: i32),

    /// シーンのサンプリングレートを設定します (call_read_section利用不可)
    /// ※シーンの操作は現状Undoに非対応です
    /// sample_rate : サンプリングレート
    pub set_scene_sample_rate: unsafe extern "C" fn(sample_rate: i32),

    /// レイヤーの表示・非表示状態を取得します
    /// layer : レイヤー番号
    /// 戻り値 : レイヤーが表示状態の場合はtrue
    pub get_layer_enable: unsafe extern "C" fn(layer: i32) -> bool,

    /// レイヤーの表示・非表示状態を設定します (call_read_section利用不可)
    /// layer : レイヤー番号
    /// enable : 設定するレイヤーの表示状態
    pub set_layer_enable: unsafe extern "C" fn(layer: i32, enable: bool),

    /// レイヤーのロック状態を取得します
    /// layer : レイヤー番号
    /// 戻り値 : レイヤーがロック状態の場合はtrue
    pub get_layer_lock: unsafe extern "C" fn(layer: i32) -> bool,

    /// レイヤーのロック状態を設定します (call_read_section利用不可)
    /// layer : レイヤー番号
    /// lock : 設定するレイヤーのロック状態
    pub set_layer_lock: unsafe extern "C" fn(layer: i32, lock: bool),

    /// オブジェクトの区間の数を取得します
    /// object : オブジェクトのハンドル
    /// 戻り値 : 区間の数
    pub get_object_section_num: unsafe extern "C" fn(object: OBJECT_HANDLE) -> i32,

    /// 選択中オブジェクトの区間の位置を取得します
    /// 戻り値 : 区間の番号 (未選択の場合は-1を返却)
    pub get_focus_object_section: unsafe extern "C" fn() -> i32,

    /// オブジェクトの区間の開始フレーム番号を取得します
    /// object : オブジェクトのハンドル
    /// section : 区間の番号
    /// 戻り値 : 区間の開始フレーム番号 (取得出来ない場合は-1を返却)
    pub get_object_section_frame: unsafe extern "C" fn(object: OBJECT_HANDLE, section: i32) -> i32,

    /// 指定フレーム位置でのオブジェクトのトラックバー項目の値を取得します
    /// ※フィルタプラグインから呼び出した場合は処理対象のシーンのオブジェクトのみ取得出来ます
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// item : 対象のトラックバー項目の名称 (エイリアスファイルのキーの名称)
    /// frame : 取得対象のフレーム番号 ※小数部でフレーム間の位置を指定出来ます
    /// value : トラックバー項目の値の格納先へのポインタ
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_object_track_value: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        item: LPCWSTR,
        frame: f64,
        value: *mut f64,
    ) -> bool,

    /// 指定フレーム位置でのオブジェクトのチェックボックス(セクション毎含む)項目の値を取得します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// item : 対象のチェックボックス項目の名称 (エイリアスファイルのキーの名称)
    /// frame : 取得対象のフレーム番号 ※セクション毎チェックボックスの場合に利用
    /// value : チェックボックス項目の値の格納先へのポインタ
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_object_check_value: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        item: LPCWSTR,
        frame: i32,
        value: *mut bool,
    ) -> bool,

    /// オブジェクトのトラックバー項目の情報を取得します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// item : 対象のトラックバー項目の名称 (エイリアスファイルのキーの名称)
    /// info : トラックバー情報の格納先へのポインタ
    /// info_size : トラックバー情報の格納先のサイズ ※TRACK_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_object_track_info: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        item: LPCWSTR,
        info: *mut TRACK_INFO,
        info_size: i32,
    ) -> bool,

    /// 現在のパレット名を取得します
    /// ラベル付きの場合は[ラベル名.パレット名]のフォーマットになります
    /// 戻り値 : 現在のパレット名 ※コールバック処理の終了まで有効
    pub get_palette_name: unsafe extern "C" fn() -> LPCWSTR,

    /// 指定のパレットの情報を取得します
    /// name : パレット名
    /// info : パレット情報の格納先へのポインタ
    /// info_size : パレット情報の格納先のサイズ ※PALETTE_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_palette_info:
        unsafe extern "C" fn(name: LPCWSTR, info: *mut PALETTE_INFO, info_size: i32) -> bool,

    /// 登録されているフォントのDirectWriteのフォントのポインタを取得する (IDWriteFontのポインタを取得します)
    /// font : フォント名 ※アプリケーション内の登録名
    /// 戻り値 : IDWriteFontのポインタ (指定フォントが無い場合はnullptrを返却)
    pub get_font: unsafe extern "C" fn(font: LPCWSTR) -> *mut c_void,

    /// オブジェクトのトラックバーグループの所属アイテム名を取得します
    /// object : オブジェクトのハンドル
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// group_name : 対象のトラックバーグループ項目の名称 (エイリアスファイルのキーの名称)
    /// item_names : 所属アイテム名の格納先へのポインタ
    /// item_num : 所属アイテム名の格納先の数
    /// 戻り値 : 取得出来た所属アイテム名の数 (指定グループが無い場合は0を返却)
    /// item_namesがnullptrの場合は所属アイテム数を返却します
    pub get_object_track_group_names: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect: LPCWSTR,
        group_name: LPCWSTR,
        item_names: *mut LPCWSTR,
        item_num: i32,
    ) -> i32,

    /// 新しい関数に差し替えるので廃止します
    #[deprecated = "新しい関数に差し替えるので廃止します"]
    pub deprecated_get_grid_bpm_list:
        unsafe extern "C" fn(bpm_list: *mut BPM_INFO, bpm_num: i32) -> i32,

    /// 新しい関数に差し替えるので廃止します
    #[deprecated = "新しい関数に差し替えるので廃止します"]
    pub deprecated_set_grid_bpm_list: unsafe extern "C" fn(bpm_list: *mut BPM_INFO, bpm_num: i32),

    /// オブジェクトからエフェクトを検索します
    /// object : 検索対象のオブジェクトのハンドル
    /// effect : 検索するエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// 同じエフェクトが複数ある場合は":n"のサフィックスでインデックス指定出来ます (nは0からの番号)
    /// nullptrを指定すると先頭のエフェクトを取得します
    /// 戻り値 : 検索したエフェクトのハンドル (見つからない場合はnullptrを返却)
    /// ※エフェクトハンドルはエフェクトが破棄されるかコールバック処理の終了まで有効
    pub find_effect: unsafe extern "C" fn(object: OBJECT_HANDLE, effect: LPCWSTR) -> EFFECT_HANDLE,

    /// オブジェクトからエフェクト一覧を取得します
    /// object : オブジェクトのハンドル
    /// effect_list : エフェクトのハンドルリストの格納先へのポインタ
    /// effect_num : エフェクトのハンドルリストの格納先の数
    /// 戻り値 : 取得出来たエフェクトハンドルの数 (取得出来ない場合は0を返却)
    /// effect_listがnullptrの場合は所有しているエフェクトの数を返却します
    /// ※エフェクトハンドルはエフェクトが破棄されるかコールバック処理の終了まで有効
    pub get_effect_list: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        effect_list: *mut EFFECT_HANDLE,
        effect_num: i32,
    ) -> i32,

    /// エフェクト名を取得します
    /// effect : エフェクトのハンドル
    /// 戻り値 : エフェクト名へのポインタ (取得出来ない場合はnullptrを返却)
    pub get_effect_name: unsafe extern "C" fn(effect: EFFECT_HANDLE) -> LPCWSTR,

    /// エフェクトの有効・無効状態を取得します
    /// effect : エフェクトのハンドル
    /// 戻り値 : エフェクトが有効状態の場合はtrue
    pub get_effect_enable: unsafe extern "C" fn(effect: EFFECT_HANDLE) -> bool,

    /// エフェクトの有効・無効状態を設定します (call_read_section利用不可)
    /// effect : エフェクトのハンドル
    /// enable : 設定するエフェクトの有効・無効状態
    /// ※エフェクトが出力項目(標準描画等)の場合は変更出来ません (常に有効状態)
    pub set_effect_enable: unsafe extern "C" fn(effect: EFFECT_HANDLE, enable: bool),

    /// エフェクトのロック状態を取得します
    /// effect : エフェクトのハンドル
    /// 戻り値 : エフェクトがロック状態の場合はtrue
    pub get_effect_lock: unsafe extern "C" fn(effect: EFFECT_HANDLE) -> bool,

    /// エフェクトのロック状態を設定します (call_read_section利用不可)
    /// effect : エフェクトのハンドル
    /// enable : 設定するエフェクトのロック状態
    /// ※エフェクトが音声の場合は変更出来ません
    /// ※エフェクトが出力項目(標準描画等)の場合は変更出来ません (入力項目に同期)
    pub set_effect_lock: unsafe extern "C" fn(effect: EFFECT_HANDLE, lock: bool),

    /// エフェクトの設定項目の値を文字列で取得します
    /// effect : エフェクトのハンドル
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// 戻り値 : 取得した設定値(UTF8)へのポインタ (取得出来ない場合はnullptrを返却)
    /// エイリアスファイルの設定値と同じフォーマットになります
    /// ※次に同一スレッドで文字列返却の関数を使うかコールバック処理の終了まで有効
    pub get_effect_item_value: unsafe extern "C" fn(effect: EFFECT_HANDLE, item: LPCWSTR) -> LPCSTR,

    /// エフェクトの設定項目の値を文字列で設定します (call_read_section利用不可)
    /// effect : エフェクトのハンドル
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// value : 設定値(UTF8)
    /// エイリアスファイルの設定値と同じフォーマットになります
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub set_effect_item_value:
        unsafe extern "C" fn(effect: EFFECT_HANDLE, item: LPCWSTR, value: LPCSTR) -> bool,

    /// 指定フレーム位置でのエフェクトのトラックバー項目の値を取得します
    /// ※フィルタプラグインから呼び出した場合は処理対象のシーンのオブジェクトのみ取得出来ます
    /// effect : エフェクトのハンドル
    /// item : 対象のトラックバー項目の名称 (エイリアスファイルのキーの名称)
    /// frame : 取得対象のフレーム番号 ※小数部でフレーム間の位置を指定出来ます
    /// value : トラックバー項目の値の格納先へのポインタ
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_effect_track_value: unsafe extern "C" fn(
        effect: EFFECT_HANDLE,
        item: LPCWSTR,
        frame: f64,
        value: *mut f64,
    ) -> bool,

    /// 指定フレーム位置でのエフェクトのチェックボックス(セクション毎含む)項目の値を取得します
    /// effect : エフェクトのハンドル
    /// item : 対象のチェックボックス項目の名称 (エイリアスファイルのキーの名称)
    /// frame : 取得対象のフレーム番号 ※セクション毎チェックボックスの場合に利用
    /// value : チェックボックス項目の値の格納先へのポインタ
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_effect_check_value: unsafe extern "C" fn(
        effect: EFFECT_HANDLE,
        item: LPCWSTR,
        frame: i32,
        value: *mut bool,
    ) -> bool,

    /// エフェクトのトラックバー項目の情報を取得します
    /// effect : エフェクトのハンドル
    /// item : 対象のトラックバー項目の名称 (エイリアスファイルのキーの名称)
    /// info : トラックバー情報の格納先へのポインタ
    /// info_size : トラックバー情報の格納先のサイズ ※TRACK_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue (対象が見つからない場合は失敗します)
    pub get_effect_track_info: unsafe extern "C" fn(
        effect: EFFECT_HANDLE,
        item: LPCWSTR,
        info: *mut TRACK_INFO,
        info_size: i32,
    ) -> bool,

    /// グリッド(BPM)のBPM情報一覧を取得します
    /// bpm_list : BPM情報リストの格納先へのポインタ
    /// bpm_num : BPM情報リストの格納先の数
    /// bpm_size : BPM情報構造体のサイズ ※BPM_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来たBPM情報の数
    /// bpm_listがnullptrの場合はグリッド(BPM)に設定されているBPM情報の数を返却します
    pub get_grid_bpm_list:
        unsafe extern "C" fn(bpm_list: *mut BPM_INFO, bpm_num: i32, bpm_size: i32) -> i32,

    /// グリッド(BPM)のBPM情報一覧を設定します (call_read_section利用不可)
    /// bpm_list : 設定するBPM情報リストへのポインタ
    /// bpm_num : 設定するBPM情報リストの要素数
    /// bpm_size : BPM情報構造体のサイズ ※BPM_INFOと異なる場合はサイズ分のみ設定されます
    pub set_grid_bpm_list:
        unsafe extern "C" fn(bpm_list: *mut BPM_INFO, bpm_num: i32, bpm_size: i32),

    /// オブジェクトにエフェクトを追加します (call_read_section利用不可)
    /// object : エフェクトを追加するオブジェクトのハンドル
    /// effect : 追加するエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// : ※エフェクトが入出力項目(図形や標準描画等)の場合は差し替えになります
    /// 戻り値 : 追加したエフェクトのハンドル (追加出来ない場合はnullptrを返却)
    /// ※エフェクトハンドルはエフェクトが破棄されるかコールバック処理の終了まで有効
    pub create_effect:
        unsafe extern "C" fn(object: OBJECT_HANDLE, effect: LPCWSTR) -> EFFECT_HANDLE,

    /// オブジェクトからエフェクトを削除します (call_read_section利用不可)
    /// object : エフェクトを削除するオブジェクトのハンドル
    /// effect : 削除するエフェクトのハンドル
    /// 戻り値 : 削除出来た場合はtrue
    pub delete_effect: unsafe extern "C" fn(object: OBJECT_HANDLE, effect: EFFECT_HANDLE) -> bool,

    /// オブジェクトに中間点(区間)を追加します (call_read_section利用不可)
    /// object : 中間点を追加するオブジェクトのハンドル
    /// frame : 中間点を追加するフレーム番号
    /// 戻り値 : 追加出来た場合はtrue
    pub create_object_section: unsafe extern "C" fn(object: OBJECT_HANDLE, frame: i32) -> bool,

    /// オブジェクトの中間点(区間)を削除します (call_read_section利用不可)
    /// object : 中間点を削除するオブジェクトのハンドル
    /// section : 削除する中間点の区間の番号 (開始位置が中間点の区間番号)
    /// 戻り値 : 削除出来た場合はtrue
    pub delete_object_section: unsafe extern "C" fn(object: OBJECT_HANDLE, section: i32) -> bool,

    /// オブジェクトの区間の開始位置を移動します (call_read_section利用不可)
    /// ※移動する区間番号(section)が区間数(最終区間+1)の場合は終了点を移動します
    /// object : 区間を移動するオブジェクトのハンドル
    /// section : 移動する区間の番号 (0〜区間数の値)
    /// frame : 移動先のフレーム番号 ※区間を跨ぐ移動は出来ません
    /// 戻り値 : 移動出来た場合はtrue
    pub move_object_section:
        unsafe extern "C" fn(object: OBJECT_HANDLE, section: i32, frame: i32) -> bool,

    /// エフェクトの順序を移動します (call_read_section利用不可)
    /// ※エフェクト種別がフィルタ効果の場合に順序を移動出来ます
    /// object : エフェクトの順序を移動するオブジェクトのハンドル
    /// effect : 順序を移動するエフェクトのハンドル
    /// index : 移動目標の順序のインデックス
    /// 戻り値 : 移動処理後の順序のインデックス (対象が見つからない場合は-1を返却)
    pub move_effect:
        unsafe extern "C" fn(object: OBJECT_HANDLE, effect: EFFECT_HANDLE, index: i32) -> i32,

    /// エフェクトの汎用データ項目の値を取得します
    /// effect : エフェクトのハンドル
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// data : 汎用データの格納先へのポインタ
    /// size : 汎用データの格納先のサイズ ※実際のサイズと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た汎用データのサイズ (取得出来ない場合は0を返却)
    /// dataがnullptrの場合は汎用データのサイズを返却します
    pub get_effect_data_value: unsafe extern "C" fn(
        effect: EFFECT_HANDLE,
        item: LPCWSTR,
        data: *mut c_void,
        size: i32,
    ) -> i32,

    /// エフェクトの汎用データ項目の値を設定します (call_read_section利用不可)
    /// effect : エフェクトのハンドル
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// data : 設定する汎用データへのポインタ
    /// size : 設定する汎用データのサイズ
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub set_effect_data_value: unsafe extern "C" fn(
        effect: EFFECT_HANDLE,
        item: LPCWSTR,
        data: *mut c_void,
        size: i32,
    ) -> bool,

    /// 編集データを編集済み状態に設定する ※通常は自動的に設定されます
    pub set_edited_state: unsafe extern "C" fn(),

    /// マークされているフレームの一覧を取得します
    /// frame_list : フレーム番号リストの格納先へのポインタ
    /// frame_num : フレーム番号リストの格納先の数
    /// 戻り値 : 取得出来たフレーム番号の数
    /// frame_listがnullptrの場合はマークされているフレームの数を返却します
    pub get_mark_frame_list: unsafe extern "C" fn(frame_list: *mut i32, frame_num: i32) -> i32,

    /// 指定フレームのマークのメモを取得します
    /// frame : マークのメモを取得するフレームの番号
    /// 戻り値 : マークのメモへのポインタ (取得出来ない場合はnullptrを返却)
    /// ※マークを編集するかコールバック処理の終了まで有効
    pub get_mark_frame_memo: unsafe extern "C" fn(frame: i32) -> LPCWSTR,

    /// 指定フレームをマークします (call_read_section利用不可)
    /// 既にマークされている場合はメモを更新します
    /// frame : マークを設定するフレームの番号
    /// memo : マークのメモ (nullptrを指定すると空を設定します)
    pub set_mark_frame: unsafe extern "C" fn(frame: i32, memo: LPCWSTR),

    /// 指定フレームのマークを解除します (call_read_section利用不可)
    /// frame : マークを解除するフレームの番号
    pub clear_mark_frame: unsafe extern "C" fn(frame: i32),

    /// 指定フレームのマークを移動します (call_read_section利用不可)
    /// frame : マークを移動するフレームの番号
    /// frame_to : マークの移動先のフレームの番号
    /// 戻り値 : 移動定出来た場合はtrue (移動元がマーク未設定、移動先がマーク済みの場合は失敗します)
    pub move_mark_frame: unsafe extern "C" fn(frame: i32, frame_to: i32) -> bool,

    /// 指定のパレットの情報を設定します (call_read_section利用不可)
    /// ※パレットファイルの保存をします
    /// name : パレット名
    /// info : パレット情報へのポインタ
    /// info_size : パレット情報のサイズ ※PALETTE_INFOのサイズ
    /// 戻り値 : 設定出来た場合はtrue (対象が見つからない場合は失敗します)
    pub set_palette_info:
        unsafe extern "C" fn(name: LPCWSTR, info: *mut PALETTE_INFO, info_size: i32) -> bool,

    /// 指定のオブジェクトフラグの状態を取得します
    /// object : フラグを取得するオブジェクトのハンドル
    /// type : オブジェクトフラグの種別
    /// 戻り値 : フラグが有効か？ (取得出来ない場合はfalseを返却)
    pub get_object_flag:
        unsafe extern "C" fn(object: OBJECT_HANDLE, r#type: OBJECT_FLAG_TYPE) -> bool,

    /// 指定のオブジェクトフラグの状態を設定します (call_read_section利用不可)
    /// object : フラグを設定するオブジェクトのハンドル
    /// type : オブジェクトフラグの種別
    /// flag : フラグの状態
    pub set_object_flag:
        unsafe extern "C" fn(object: OBJECT_HANDLE, r#type: OBJECT_FLAG_TYPE, flag: bool),

    /// オブジェクトIDを取得します
    /// object : IDを取得するオブジェクトのハンドル
    /// 戻り値 : オブジェクトID (取得出来ない場合は0を返却)
    pub get_object_id: unsafe extern "C" fn(object: OBJECT_HANDLE) -> i64,

    /// エフェクトIDを取得します
    /// effect : IDを取得するエフェクトのハンドル
    /// 戻り値 : エフェクトID (取得出来ない場合は0を返却)
    pub get_effect_id: unsafe extern "C" fn(effect: EFFECT_HANDLE) -> i64,
}

/// 編集ハンドル構造体
/// get_host_app_window()以外はRegisterPlugin処理内から利用出来ません
#[repr(C)]
pub struct EDIT_HANDLE {
    /// プロジェクトデータの編集をする為のコールバック関数(func_proc_edit)を呼び出します
    /// 編集情報を排他制御する為に更新ロック状態のコールバック関数内で編集処理をする形になります
    /// コールバック関数内で編集したオブジェクトは纏めてUndoに登録されます
    /// コールバック関数はメインスレッドから呼ばれます
    /// func_proc_edit : 編集処理のコールバック関数
    /// 戻り値 : trueなら成功
    /// 編集が出来ない場合(出力中等)に失敗します
    pub call_edit_section:
        unsafe extern "C" fn(func_proc_edit: unsafe extern "C" fn(edit: *mut EDIT_SECTION)) -> bool,

    /// call_edit_section()に引数paramを渡せるようにした関数です
    /// param : 任意のユーザーデータのポインタ
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

    /// エフェクト名の一覧をコールバック関数(func_proc_enum_effect)で取得します
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

    /// モジュール情報の一覧をコールバック関数(func_proc_enum_module)で取得します
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_module : モジュール情報の取得処理のコールバック関数
    pub enum_module_info: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_enum_module: unsafe extern "C" fn(param: *mut c_void, info: *mut MODULE_INFO),
    ),

    /// ホストアプリケーションのメインウィンドウのハンドルを取得します
    pub get_host_app_window: unsafe extern "C" fn() -> HWND,

    /// 編集状態を取得します
    pub get_edit_state: unsafe extern "C" fn() -> i32,

    /// プロジェクトデータを参照する為のコールバック関数(func_proc_read_section)を呼び出します
    /// 参照中にデータが更新されないように参照ロック状態のコールバック関数内で処理をする形になります
    /// EDIT_SECTIONの更新系の関数等は利用出来ません ※EDIT_SECTIONの各項目に記載しています
    /// コールバック関数は呼び出し元と同じスレッドで呼ばれます
    /// func_proc_read_section : コールバック関数
    /// 戻り値 : trueなら成功
    /// 参照が出来ない場合(出力中等)に失敗します
    pub call_read_section: unsafe extern "C" fn(
        func_proc_read_section: unsafe extern "C" fn(edit: *mut EDIT_SECTION),
    ) -> bool,

    /// call_read_section()に引数paramを渡せるようにした関数です
    /// param : 任意のユーザーデータのポインタ
    pub call_read_section_param: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_read_section: unsafe extern "C" fn(param: *mut c_void, edit: *mut EDIT_SECTION),
    ) -> bool,

    /// エフェクトの設定項目の一覧をコールバック関数(func_proc_enum_effect_item)で取得します
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_effect_item : エフェクトの設定項目の取得処理のコールバック関数
    /// 戻り値 : 取得出来た場合はtrue (対象が見つからない場合は失敗します)
    pub enum_effect_item: unsafe extern "C" fn(
        effect: LPCWSTR,
        param: *mut c_void,
        func_proc_enum_effect_item: unsafe extern "C" fn(
            param: *mut c_void,
            name: LPCWSTR,
            r#type: i32,
        ),
    ) -> bool,

    /// 現在のシーンの映像のレンダリングをします
    /// この関数はレンダリングのタスクを追加するのみで完了します
    /// レンダリング完了時はイベント通知スレッドからコールバック関数が呼ばれます
    /// frame : レンダリング対象のフレーム番号
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_rendering_video : レンダリング完了時に呼ばれるコールバック関数
    /// buffer : レンダリングした画像データへのポインタ ※PIXEL_RGBA形式
    /// width,height : レンダリングした画像サイズ
    /// pitch : レンダリングした画像データの横1ラインのバイト数
    /// 戻り値 : レンダリング要求が成功した場合はtrue (出力中等は失敗します)
    pub rendering_scene_video: unsafe extern "C" fn(
        frame: i32,
        param: *mut c_void,
        func_proc_rendering_video: unsafe extern "C" fn(
            param: *mut c_void,
            frame: i32,
            buffer: *const c_void,
            width: i32,
            height: i32,
            pitch: i32,
        ),
    ) -> bool,

    /// 現在のシーンの音声のレンダリングをします
    /// この関数はレンダリングのタスクを追加するのみで完了します
    /// レンダリング完了時はイベント通知スレッドからコールバック関数が呼ばれます
    /// frame : レンダリング対象のフレーム番号
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_rendering_audio : レンダリング完了時に呼ばれるコールバック関数
    /// buffer0 : レンダリングした音声データ(左チャンネル)へのポインタ ※PCM(float)32bit形式
    /// buffer1 : レンダリングした音声データ(右チャンネル)へのポインタ ※PCM(float)32bit形式
    /// sample_num : レンダリングした音声のサンプル数
    /// 戻り値 : レンダリング要求が成功した場合はtrue (出力中等は失敗します)
    pub rendering_scene_audio: unsafe extern "C" fn(
        frame: i32,
        param: *mut c_void,
        func_proc_rendering_audio: unsafe extern "C" fn(
            param: *mut c_void,
            frame: i32,
            buffer0: *const f32,
            buffer1: *const f32,
            sample_num: i32,
        ),
    ) -> bool,

    /// レンダリング中のタスクが全て完了するまで待機します
    /// ※参照ロック、編集ロック状態で呼び出すとデッドロックする可能性があります
    pub wait_rendering_task: unsafe extern "C" fn(),

    /// フォント名の一覧をコールバック関数(func_proc_enum_font)で取得します
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_font : フォント名の取得処理のコールバック関数
    pub enum_font_name: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_enum_font: unsafe extern "C" fn(param: *mut c_void, name: LPCWSTR),
    ),

    /// パレット名の一覧をコールバック関数(func_proc_enum_palette)で取得します
    /// パレット情報を排他制御する為に参照ロックします。※同一スレッドで既にロック状態の場合はそのまま取得します。
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_palette : パレット名の取得処理のコールバック関数
    pub enum_palette_name: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_enum_palette: unsafe extern "C" fn(param: *mut c_void, name: LPCWSTR),
    ),

    /// 指定のオブジェクトの映像のレンダリングをします
    /// この関数はレンダリングのタスクを追加するのみで完了します
    /// レンダリング完了時はイベント通知スレッドからコールバック関数が呼ばれます
    /// object : レンダリング対象のオブジェクトのハンドル
    /// frame : レンダリング対象のフレーム番号
    /// apply_effect : 追加のフィルタ効果を反映するか？ ※グループ制御の追加効果は反映されません
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_rendering_video : レンダリング完了時に呼ばれるコールバック関数
    /// buffer : レンダリングした画像データへのポインタ ※PIXEL_RGBA形式
    /// width,height : レンダリングした画像サイズ
    /// pitch : レンダリングした画像データの横1ラインのバイト数
    /// 戻り値 : レンダリング要求が成功した場合はtrue (対象外のオブジェクトや出力中等は失敗します)
    pub rendering_object_video: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        frame: i32,
        apply_effect: bool,
        param: *mut c_void,
        func_proc_rendering_video: unsafe extern "C" fn(
            param: *mut c_void,
            frame: i32,
            buffer: *const c_void,
            width: i32,
            height: i32,
            pitch: i32,
        ),
    ) -> bool,

    /// 指定のオブジェクトの音声のレンダリングをします
    /// この関数はレンダリングのタスクを追加するのみで完了します
    /// レンダリング完了時はイベント通知スレッドからコールバック関数が呼ばれます
    /// object : レンダリング対象のオブジェクトのハンドル
    /// frame : レンダリング対象のフレーム番号
    /// apply_effect : 追加のフィルタ効果を反映するか？ ※グループ制御の追加効果は反映されません
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_rendering_audio : レンダリング完了時に呼ばれるコールバック関数
    /// buffer0 : レンダリングした音声データ(左チャンネル)へのポインタ ※PCM(float)32bit形式
    /// buffer1 : レンダリングした音声データ(右チャンネル)へのポインタ ※PCM(float)32bit形式
    /// sample_num : レンダリングした音声のサンプル数
    /// 戻り値 : レンダリング要求が成功した場合はtrue (出力中等は失敗します)
    pub rendering_object_audio: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        frame: i32,
        apply_effect: bool,
        param: *mut c_void,
        func_proc_rendering_audio: unsafe extern "C" fn(
            param: *mut c_void,
            frame: i32,
            buffer0: *const f32,
            buffer1: *const f32,
            sample_num: i32,
        ),
    ) -> bool,

    /// 指定の設定項目が所属するグループの所属アイテム名を取得します
    /// グループ項目を直接指定することも出来ます (item_indexにはnullptrを指定します)
    /// effect : 対象のエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// item : 対象の設定項目の名称 (エイリアスファイルのキーの名称)
    /// item_names : グループの所属アイテム名の格納先へのポインタ
    /// item_num : グループの所属アイテム名の格納先の数
    /// item_index : 対象の設定項目のグループ内のインデックスの格納先へのポインタ (nullptrの場合は格納しません)
    /// 戻り値 : 取得出来た所属アイテム名の数 (グループに所属していない場合は0を返却)
    /// item_namesがnullptrの場合は所属アイテム数を返却します
    pub get_effect_item_group_names: unsafe extern "C" fn(
        effect: LPCWSTR,
        item: LPCWSTR,
        item_names: *mut LPCWSTR,
        item_num: i32,
        item_index: *mut i32,
    ) -> i32,

    /// シーン名の一覧をコールバック関数(func_proc_enum_scene)で取得します
    /// シーン情報を排他制御する為に参照ロックします。※同一スレッドで既にロック状態の場合はそのまま取得します。
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_enum_scene : シーン名の取得処理のコールバック関数
    pub enum_scene_name: unsafe extern "C" fn(
        param: *mut c_void,
        func_proc_enum_scene: unsafe extern "C" fn(param: *mut c_void, name: LPCWSTR, scene_id: i32),
    ),

    /// 指定のシーンに切り替えます
    /// 参照ロック、編集ロック状態では利用出来ません
    /// scene_id : シーンのID
    /// 戻り値 : 成功した場合はtrue (シーンが存在しない場合や出力中等は失敗します)
    pub select_scene: unsafe extern "C" fn(scene_id: i32) -> bool,

    /// シーンを作成します (作成したシーンに切り替わります)
    /// 参照ロック、編集ロック状態では利用出来ません
    /// name : シーン名
    /// label : シーンのラベル (nullptrか空文字を指定するとラベル無し)
    /// width, height : シーンの解像度
    /// rate, scale : シーンのフレームレート
    /// sample_rate : シーンのサンプリングレート
    /// background : シーンの背景色 (background.aが255以外の場合は透明色)
    /// 戻り値 : 成功した場合はtrue (出力中等は失敗します)
    pub create_scene: unsafe extern "C" fn(
        name: LPCWSTR,
        label: LPCWSTR,
        width: i32,
        height: i32,
        rate: i32,
        scale: i32,
        sample_rate: i32,
        background: EDIT_INFO_COLOR,
    ) -> bool,

    /// プロジェクトを新規作成します
    /// 参照ロック、編集ロック状態では利用出来ません
    /// width, height : シーンの解像度
    /// rate, scale : シーンのフレームレート
    /// sample_rate : シーンのサンプリングレート
    /// background : シーンの背景色 (background.aが255以外の場合は透明色)
    /// show_confirm : 現在のプロジェクトの保存・キャンセルの確認ダイアログを表示する
    /// 戻り値 : 成功した場合はtrue (出力中等は失敗します)
    pub create_project: unsafe extern "C" fn(
        width: i32,
        height: i32,
        rate: i32,
        scale: i32,
        sample_rate: i32,
        background: EDIT_INFO_COLOR,
        show_confirm: bool,
    ) -> bool,

    /// 指定のプロジェクトファイルを開きます
    /// 参照ロック、編集ロック状態では利用出来ません
    /// file : プロジェクトファイルのパス
    /// show_confirm : 現在のプロジェクトの保存・キャンセルの確認ダイアログを表示する
    /// 戻り値 : 成功した場合はtrue (出力中等は失敗します)
    pub open_project_file: unsafe extern "C" fn(file: LPCWSTR, show_confirm: bool) -> bool,

    /// 指定のプロジェクトファイルへ保存します (自動バックアップと同じ処理で保存されます)
    /// 参照ロック、編集ロック状態では利用出来ません
    /// file : プロジェクトファイルのパス
    /// 戻り値 : 成功した場合はtrue (出力中等は失敗します)
    pub save_project_file: unsafe extern "C" fn(file: LPCWSTR) -> bool,

    /// 現在のシーンを出力プラグインでファイル出力します
    /// この関数はファイル出力を開始するのみで完了します
    /// 参照ロック、編集ロック状態では利用出来ません
    /// file : 出力ファイルのパス
    /// output_plugin : 出力プラグイン名
    /// func_project_config : 出力開始時にプロジェクトのファイル出力設定を反映させるコールバック関数 (nullptrなら呼ばれません)
    /// ※出力プラグインのFLAG_PROJECT_CONFIGが有効の場合にfunc_save_project_config()と同じ設定をすることで反映出来ます
    /// 戻り値 : 成功した場合はtrue (出力中等は失敗します)
    pub output_file: unsafe extern "C" fn(
        file: LPCWSTR,
        output_plugin: LPCWSTR,
        param: *mut c_void,
        func_project_config: Option<
            unsafe extern "C" fn(param: *mut c_void, project: *mut PROJECT_FILE),
        >,
    ) -> bool,
}

impl EDIT_HANDLE {
    /// エフェクト種別 ※今後追加される可能性があります
    /// フィルタ効果
    pub const EFFECT_TYPE_FILTER: i32 = 1;
    /// メディア入力
    pub const EFFECT_TYPE_INPUT: i32 = 2;
    /// シーンチェンジ
    pub const EFFECT_TYPE_TRANSITION: i32 = 3;
    /// オブジェクト制御
    pub const EFFECT_TYPE_CONTROL: i32 = 4;
    /// メディア出力
    /// エフェクトフラグ ※今後追加される可能性があります
    pub const EFFECT_TYPE_OUTPUT: i32 = 5;
    /// 画像をサポート
    pub const EFFECT_FLAG_VIDEO: i32 = 1;
    /// 音声をサポート
    pub const EFFECT_FLAG_AUDIO: i32 = 2;
    /// フィルタオブジェクトをサポート
    pub const EFFECT_FLAG_FILTER: i32 = 4;
    /// カメラ効果をサポート
    pub const EFFECT_FLAG_CAMERA: i32 = 8;
    /// 編集中
    pub const EDIT_STATE_EDIT: i32 = 0;
    /// プレビュー再生中
    pub const EDIT_STATE_PLAY: i32 = 1;
    /// ファイル出力中
    pub const EDIT_STATE_SAVE: i32 = 2;
    /// 設定項目種別 ※今後追加される可能性があります
    /// 整数
    pub const EFFECT_ITEM_TYPE_INTEGER: i32 = 1;
    /// 数値(トラックバー)
    pub const EFFECT_ITEM_TYPE_NUMBER: i32 = 2;
    /// チェックボックス
    pub const EFFECT_ITEM_TYPE_CHECK: i32 = 3;
    /// テキスト
    pub const EFFECT_ITEM_TYPE_TEXT: i32 = 4;
    /// 文字列
    pub const EFFECT_ITEM_TYPE_STRING: i32 = 5;
    /// ファイル
    pub const EFFECT_ITEM_TYPE_FILE: i32 = 6;
    /// 色
    pub const EFFECT_ITEM_TYPE_COLOR: i32 = 7;
    /// リスト選択
    pub const EFFECT_ITEM_TYPE_SELECT: i32 = 8;
    /// シーン
    pub const EFFECT_ITEM_TYPE_SCENE: i32 = 9;
    /// レイヤー範囲
    pub const EFFECT_ITEM_TYPE_RANGE: i32 = 10;
    /// リストと文字の複合
    pub const EFFECT_ITEM_TYPE_COMBO: i32 = 11;
    /// マスク
    pub const EFFECT_ITEM_TYPE_MASK: i32 = 12;
    /// フォント
    pub const EFFECT_ITEM_TYPE_FONT: i32 = 13;
    /// 図形
    pub const EFFECT_ITEM_TYPE_FIGURE: i32 = 14;
    /// データ
    pub const EFFECT_ITEM_TYPE_DATA: i32 = 15;
    /// フォルダ
    pub const EFFECT_ITEM_TYPE_FOLDER: i32 = 16;
    /// 数値(トラックバー)グループ
    pub const EFFECT_ITEM_TYPE_NUMBER_GROUP: i32 = 17;
    /// 設定グループ(明示的なグループのみ) ※設定値無し
    pub const EFFECT_ITEM_TYPE_GROUP: i32 = 18;
    /// セパレーター ※設定値無し
    pub const EFFECT_ITEM_TYPE_SEPARATOR: i32 = 19;
}

/// プロジェクトファイル構造体
/// プロジェクトファイルのロード、セーブのコールバックや編集のコールバック関数内で利用出来ます
/// プロジェクトの保存データはプラグイン毎のデータ領域になります
#[repr(C)]
pub struct PROJECT_FILE {
    /// プロジェクトに保存されている文字列(UTF-8)を取得します
    /// key : キー名(UTF-8)
    /// 戻り値 : 取得した文字列へのポインタ (未設定の場合はnullptr)
    /// ※コールバック処理の終了まで有効
    pub get_param_string: unsafe extern "C" fn(key: LPCSTR) -> LPCSTR,
    /// プロジェクトに文字列(UTF-8)を保存します
    /// key : キー名(UTF-8)
    /// value : 保存する文字列(UTF-8)
    pub set_param_string: unsafe extern "C" fn(key: LPCSTR, value: LPCSTR),
    /// プロジェクトに保存されているバイナリデータを取得します
    /// key : キー名(UTF-8)
    /// data : 取得するデータの格納先へのポインタ
    /// size : 取得するデータのサイズ (保存されているサイズと異なる場合は失敗します)
    /// 戻り値 : 正しく取得出来た場合はtrue
    pub get_param_binary: unsafe extern "C" fn(key: LPCSTR, data: *mut c_void, size: i32) -> bool,
    /// プロジェクトにバイナリデータを保存します
    /// key : キー名(UTF-8)
    /// data : 保存するデータへのポインタ
    /// size : 保存するデータのサイズ (4096バイト以下)
    pub set_param_binary: unsafe extern "C" fn(key: LPCSTR, data: *mut c_void, size: i32),
    /// プロジェクトに保存されているデータを全て削除します
    pub clear_params: unsafe extern "C" fn(),

    /// プロジェクトファイルのパスを取得します
    /// key : キー名(UTF-8)
    /// 戻り値 : プロジェクトファイルパスへのポインタ (ファイルパスは未設定の場合があります)
    /// ※コールバック処理の終了まで有効
    pub get_project_file_path: unsafe extern "C" fn() -> LPCWSTR,
}

/// ホストアプリケーション構造体
#[repr(C)]
pub struct HOST_APP_TABLE {
    /// プラグインの情報を設定する
    /// information : プラグインの情報
    /// ※現在はGetCommonPluginTable()を利用する方法が推奨になります
    #[deprecated(note = "現在はGetCommonPluginTable()を利用する方法が推奨になります")]
    pub set_plugin_information: unsafe extern "C" fn(information: LPCWSTR),

    /// 入力プラグインを登録する
    /// input_plugin_table : 入力プラグイン構造体
    pub register_input_plugin: unsafe extern "C" fn(input_plugin_table: *mut INPUT_PLUGIN_TABLE),
    /// 出力プラグインを登録する
    /// output_plugin_table : 出力プラグイン構造体
    pub register_output_plugin: unsafe extern "C" fn(output_plugin_table: *mut OUTPUT_PLUGIN_TABLE),
    /// フィルタプラグインを登録する
    /// filter_plugin_table : フィルタプラグイン構造体
    /// ※FLAG_USERDATAを利用する場合は編集リソースが破棄されてからUninitializePlugin()が呼ばれます
    pub register_filter_plugin: unsafe extern "C" fn(filter_plugin_table: *mut FILTER_PLUGIN_TABLE),
    /// スクリプトモジュールを登録する
    /// script_module_table : スクリプトモジュール構造体
    pub register_script_module: unsafe extern "C" fn(script_module_table: *mut SCRIPT_MODULE_TABLE),

    /// インポートメニューを登録する (ウィンドウメニューのファイルに追加されます)
    /// name : インポートメニューの名称
    /// func_proc_import : インポートメニュー選択時のコールバック関数
    pub register_import_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_import: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),
    /// エクスポートメニューを登録する (ウィンドウメニューのファイルに追加されます)
    /// name : エクスポートメニューの名称
    /// func_proc_export : エクスポートメニュー選択時のコールバック関数
    pub register_export_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_export: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),

    /// ウィンドウクライアントを登録する
    /// name : ウィンドウの名称
    /// hwnd : ウィンドウハンドル
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
    /// name : レイヤーメニューの名称 ※名称に'\'を入れると表示を複数階層に出来ます
    /// func_proc_layer_menu : レイヤーメニュー選択時のコールバック関数
    pub register_layer_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        func_proc_layer_menu: unsafe extern "C" fn(*mut EDIT_SECTION),
    ),

    /// オブジェクトメニューを登録する (レイヤー編集でオブジェクト選択時の右クリックメニューに追加されます)
    /// name : オブジェクトメニューの名称 ※名称に'\'を入れると表示を複数階層に出来ます
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
    /// name : レイヤーメニューの名称 ※名称に'\'を入れると表示を複数階層に出来ます
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_layer_menu : レイヤーメニュー選択時のコールバック関数
    pub register_layer_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        param: *mut c_void,
        func_proc_layer_menu: unsafe extern "C" fn(param: *mut c_void),
    ),

    /// オブジェクトメニューを登録する (レイヤー編集でオブジェクト選択時の右クリックメニューに追加されます)
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : オブジェクトメニューの名称 ※名称に'\'を入れると表示を複数階層に出来ます
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

    /// ファイルをD&Dした時に呼ばれる関数を登録する
    /// name : ドラッグ時のツールチップや入力プラグインの設定で表示する名称
    /// filefilter : D&Dに対応するファイルフィルタ
    /// func_proc_file_drop : ファイルをD&Dした時のコールバック関数
    pub register_file_drop_handler: unsafe extern "C" fn(
        name: LPCWSTR,
        filefilter: LPCWSTR,
        func_proc_file_drop: unsafe extern "C" fn(edit_section: *mut EDIT_SECTION, file: LPCWSTR),
    ),

    /// ファイルをD&Dした時に呼ばれる関数を登録する
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : ドラッグ時のツールチップや入力プラグインの設定で表示する名称
    /// filefilter : D&Dに対応するファイルフィルタ
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_file_drop : ファイルをD&Dした時のコールバック関数
    pub register_file_drop_param_handler: unsafe extern "C" fn(
        name: LPCWSTR,
        filefilter: LPCWSTR,
        param: *mut c_void,
        func_proc_file_drop: unsafe extern "C" fn(param: *mut c_void, file: LPCWSTR),
    ),

    /// オブジェクト編集の設定項目メニューを登録する (オブジェクト編集の右クリックメニューに追加されます)
    /// name : 設定項目メニューの名称 ※名称に'\'を入れると表示を複数階層に出来ます
    /// allow_effect_only : エフェクトのみを許可するか? ※trueの場合はitemがnullptrで呼ばれるケースを許可します
    /// func_proc_item_menu : 設定項目メニュー選択時のコールバック関数
    /// ※コールバック関数の引数はget_object_item_value()の引数と同じ形式になります
    pub register_object_item_menu: unsafe extern "C" fn(
        name: LPCWSTR,
        allow_effect_only: bool,
        func_proc_item_menu: unsafe extern "C" fn(
            edit: *mut EDIT_SECTION,
            object: OBJECT_HANDLE,
            effect: LPCWSTR,
            item: LPCWSTR,
        ),
    ),

    /// オブジェクト編集の設定項目メニューを登録する (オブジェクト編集の右クリックメニューに追加されます)
    /// 引数paramを渡して編集セクションにしないでコールバックを呼び出します
    /// name : 設定項目メニューの名称 ※名称に'\'を入れると表示を複数階層に出来ます
    /// allow_effect_only : エフェクトのみを許可するか? ※trueの場合はitemがnullptrで呼ばれるケースを許可します
    /// param : 任意のユーザーデータのポインタ
    /// func_proc_item_menu : 設定項目メニュー選択時のコールバック関数
    /// ※コールバック関数の引数はget_object_item_value()の引数と同じ形式になります
    pub register_object_item_menu_param: unsafe extern "C" fn(
        name: LPCWSTR,
        allow_effect_only: bool,
        param: *mut c_void,
        func_proc_item_menu: unsafe extern "C" fn(
            param: *mut c_void,
            object: OBJECT_HANDLE,
            effect: LPCWSTR,
            item: LPCWSTR,
        ),
    ),

    /// スクリプトモジュールをモジュール名を指定して登録する
    /// script_module_table : スクリプトモジュール構造体
    /// module_name : モジュール名
    pub register_script_module_name:
        unsafe extern "C" fn(script_module_table: *mut SCRIPT_MODULE_TABLE, module_name: LPCWSTR),

    /// フォントコレクションを登録する
    /// collection : フォントコレクション (IDWriteFontCollectionのポインタ)
    /// ※DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)から作成したものが利用出来ると思います
    pub register_font_collection: unsafe extern "C" fn(collection: *mut std::ffi::c_void),

    /// 指定のイベントのコールバック関数を登録する
    /// コールバック関数はイベント通知スレッドから呼ばれます
    /// イベント処理からcall_edit_section()は利用出来ません
    /// event : イベント種別
    /// func_proc_event : イベント処理のコールバック関数
    pub register_event_listener: unsafe extern "C" fn(
        r#type: EVENT_TYPE,
        param: *mut c_void,
        func_proc_event: unsafe extern "C" fn(param: *mut c_void),
    ),
}
