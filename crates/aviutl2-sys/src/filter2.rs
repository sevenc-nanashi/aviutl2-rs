#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::{ffi::c_void, mem::MaybeUninit};

use crate::{common::LPCWSTR, plugin2::EDIT_SECTION};

/// オブジェクトハンドル
pub type OBJECT_HANDLE = *mut c_void;

#[repr(C)]
pub union FILTER_ITEM {
    pub track: FILTER_ITEM_TRACK,
    pub track_group: FILTER_ITEM_TRACK_GROUP,
    pub checkbox: FILTER_ITEM_CHECKBOX,
    pub check_section: FILTER_ITEM_CHECK_SECTION,
    pub color: FILTER_ITEM_COLOR,
    pub select: FILTER_ITEM_SELECT,
    pub file: FILTER_ITEM_FILE,
    pub data: FILTER_ITEM_DATA,
    pub group: FILTER_ITEM_GROUP,
    pub button: FILTER_ITEM_BUTTON,
    pub string: FILTER_ITEM_STRING,
    pub text: FILTER_ITEM_TEXT,
    pub folder: FILTER_ITEM_FOLDER,
    pub separator: FILTER_ITEM_SEPARATOR,
}

/// トラックバー項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_TRACK {
    /// 設定の種別（L"track2"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: f64,
    /// 設定値の最小
    pub s: f64,
    /// 設定値の最大
    pub e: f64,
    /// 設定値の単位
    pub step: f64,
    /// ゼロ値名称 (設定値が0の時にトラックバーに表示する文字列)
    pub zero_display: LPCWSTR,
    /// 操作倍率 (設定値の範囲に対してのトラックバー操作範囲の倍率)
    pub slider_ratio: f64,
}

/// トラックバーグループ項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_TRACK_GROUP {
    /// 設定の種別（L"trackgroup"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// トラックバー項目グループ
    pub tracks: *mut *mut FILTER_ITEM_TRACK,
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

pub type FILTER_ITEM_CHECK = FILTER_ITEM_CHECKBOX;

/// チェックボックス(セクション毎)項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_CHECK_SECTION {
    /// 設定の種別（L"checksection"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値（フィルタ処理の呼び出し時に現在の値に更新されます）
    pub value: bool,
    /// セクション毎設定の初期値
    pub multi_section: bool,
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

/// セパレーター項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_SEPARATOR {
    /// 設定の種別（L"separator"）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
}

/// 頂点データ構造体(描画色)
#[repr(C)]
pub struct VERTEX_COLOR {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// 頂点データ構造体(描画色、法線)
#[repr(C)]
pub struct VERTEX_COLOR_NORM {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

/// 頂点データ構造体(テクスチャ)
#[repr(C)]
pub struct VERTEX_TEXTURE {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    pub a: f32,
}

/// 頂点データ構造体(テクスチャ、法線)
#[repr(C)]
pub struct VERTEX_TEXTURE_NORM {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    pub a: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

/// 頂点リストの種別
#[repr(i32)]
pub enum VERTEX_TYPE {
    /// 三角形の `VERTEX_COLOR` のリスト（頂点数は3の倍数になる）
    TRIANGLE_COLOR = 1,
    /// 三角形の `VERTEX_COLOR_NORM` のリスト（頂点数は3の倍数になる）
    TRIANGLE_COLOR_NORM = 2,
    /// 三角形の `VERTEX_TEXTURE` のリスト（頂点数は3の倍数になる）
    TRIANGLE_TEXTURE = 3,
    /// 三角形の `VERTEX_TEXTURE_NORM` のリスト（頂点数は3の倍数になる）
    TRIANGLE_TEXTURE_NORM = 4,
    /// 四角形の `VERTEX_COLOR` のリスト（頂点数は4の倍数になる）
    QUAD_COLOR = 5,
    /// 四角形の `VERTEX_COLOR_NORM` のリスト（頂点数は4の倍数になる）
    QUAD_COLOR_NORM = 6,
    /// 四角形の `VERTEX_TEXTURE` のリスト（頂点数は4の倍数になる）
    QUAD_TEXTURE = 7,
    /// 四角形の `VERTEX_TEXTURE_NORM` のリスト（頂点数は4の倍数になる）
    QUAD_TEXTURE_NORM = 8,
}

/// サンプラーの種別
#[repr(i32)]
pub enum SAMPLER_MODE {
    /// 領域外は透明色
    CLIP = 0,
    /// 領域外は一番外側の色
    CLAMP = 1,
    /// 領域外はループ
    LOOP = 2,
    /// 領域外は領域を反転しながらループ
    MIRROR = 3,
    /// 拡大縮小補間をしない（領域外は透明色）
    DOT = 4,
}

/// 合成モードの種別
#[repr(i32)]
pub enum BLEND_MODE {
    /// 通常
    NONE = 0,
    /// 加算
    ADD = 1,
    /// 減算
    SUB = 2,
    /// 乗算
    MUL = 3,
    /// スクリーン
    SCREEN = 4,
    /// オーバーレイ
    OVERLAY = 5,
    /// 比較（明）
    LIGHT = 6,
    /// 比較（暗）
    DARK = 7,
    /// 輝度
    BRIGHTNESS = 8,
    /// 色差
    CHROMA = 9,
    /// 陰影
    SHADOW = 10,
    /// 明暗
    LIGHT_DARK = 11,
    /// 差分
    DIFF = 12,
}

/// ビルボードの種別
#[repr(i32)]
pub enum BILLBOARD_MODE {
    /// 標準の向き（何もしない）
    NONE = 0,
    /// 横方向のみカメラに向ける
    SIDE = 1,
    /// 縦横方向のみカメラに向ける
    DIRECTION = 2,
    /// カメラに向ける
    CAMERA = 3,
}

/// 出力ブレンド(BlendState)の種別
#[repr(i32)]
pub enum BLEND_STATE_MODE {
    /// 出力をそのままコピー
    COPY = 0,
    /// α値のみを乗算 ※RGB値は利用されません
    MASK = 1,
    /// 出力をアルファブレンド
    DRAW = 2,
    /// 出力を加算合成
    ADD = 3,
}

/// 画像入力のピクセルフォーマット種別
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum INPUT_PIXEL_FORMAT {
    /// DXGI_FORMAT_R8G8B8A8_UNORM ※PIXEL_RGBA
    RGBA = 28,
    /// DXGI_FORMAT_B8G8R8A8_UNORM
    BGRA = 87,
    /// DXGI_FORMAT_B8G8R8X8_UNORM
    BGR = 88,
    /// DXGI_FORMAT_R16G16B16A16_UNORM
    PA64 = 11,
    /// DXGI_FORMAT_R16G16B16A16_FLOAT
    HF64 = 10,
    /// DXGI_FORMAT_YUY2
    YUY2 = 107,
    /// DXGI_FORMAT_R16G16B16A16_SNORM ※互換対応
    YC48 = 13,
}

/// 画像出力のピクセルフォーマット種別
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OUTPUT_PIXEL_FORMAT {
    /// DXGI_FORMAT_R8G8B8A8_UNORM ※PIXEL_RGBA
    RGBA = 28,
    /// DXGI_FORMAT_R16G16B16A16_UNORM
    PA64 = 11,
    /// DXGI_FORMAT_R16G16B16A16_FLOAT
    HF64 = 10,
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
    /// フラグ
    pub flag: i32,
    /// オブジェクトの現在のレイヤー番号
    pub layer: i32,
    /// 複数オブジェクト時の現在の対象番号
    pub index: i32,
    /// 複数オブジェクト時の対象数 (1 = 単体オブジェクト / 0 = 不定)
    pub num: i32,
    /// 全体(シーン)基準のオブジェクトの開始フレーム(0からの番号)
    pub frame_s: i32,
    /// 全体(シーン)基準のオブジェクトの終了フレーム(0からの番号)
    pub frame_e: i32,
}

impl OBJECT_INFO {
    /// フィルタオブジェクトか？
    pub const FLAG_FILTER_OBJECT: i32 = 1;
}

/// オブジェクトの画像パラメータ構造体
#[repr(C)]
pub struct OBJECT_IMAGE_PARAM {
    /// 基準座標
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// 回転角度 (360.0で1回転)
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    /// 拡大率 (1.0=等倍)
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
    /// 中心座標 (基準座標からの相対)
    pub cx: f32,
    pub cy: f32,
    pub cz: f32,
    /// 不透明度 (0.0〜1.0/0.0=透明/1.0=不透明)
    pub alpha: f32,
}

/// オブジェクトの音声パラメータ構造体
#[repr(C)]
pub struct OBJECT_AUDIO_PARAM {
    /// 音量倍率 (1.0=等倍)
    pub vol_l: f32,
    pub vol_r: f32,
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

    /// 編集セクション関数
    pub edit: *mut EDIT_SECTION,

    /// 現在のオブジェクトの画像パラメータ情報
    pub param: *mut OBJECT_IMAGE_PARAM,

    /// 指定オブジェクトの画像出力項目のパラメータを取得する
    pub get_output_image_param: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        offset: f64,
        param: *mut OBJECT_IMAGE_PARAM,
        param_size: i32,
    ) -> bool,

    /// 指定のレイヤーにある画像オブジェクトを取得します
    pub get_image_object: unsafe extern "C" fn(layer: i32, offset: f64) -> OBJECT_HANDLE,

    /// 指定の画像リソースをフレームバッファに描画します
    pub draw_image: unsafe extern "C" fn(
        image: LPCWSTR,
        x: f32,
        y: f32,
        z: f32,
        rx: f32,
        ry: f32,
        rz: f32,
        sx: f32,
        sy: f32,
        sz: f32,
        alpha: f32,
    ) -> bool,

    /// 指定の頂点リストのポリゴンをフレームバッファに描画します
    pub draw_poly: unsafe extern "C" fn(
        vertex_type: VERTEX_TYPE,
        vertex_list: *const c_void,
        vertex_num: i32,
        image: LPCWSTR,
    ) -> bool,

    /// 標準のアンカー枠を設定します
    pub set_default_anchor: unsafe extern "C" fn(width: i32, height: i32),

    /// 描画時の合成モードを設定します
    pub set_blend_mode: unsafe extern "C" fn(blend: BLEND_MODE),

    /// 描画時の光沢度を設定します
    pub set_material_shine: unsafe extern "C" fn(shine: f32),

    /// 描画時のサンプラーを設定します
    pub set_sampler_mode: unsafe extern "C" fn(sampler: SAMPLER_MODE),

    /// 描画時に裏面を非表示にするかを設定します
    pub set_culling_state: unsafe extern "C" fn(culling: bool),

    /// 描画時にオブジェクトをカメラの方向に向けるかを設定します
    pub set_billboard_mode: unsafe extern "C" fn(billboard: BILLBOARD_MODE),

    /// 画像リソースを作成する
    pub create_image_resource:
        unsafe extern "C" fn(image: LPCWSTR, buffer: *const PIXEL_RGBA, width: i32, height: i32),

    /// 指定の画像リソースのD3D画像リソースのポインタを取得する
    pub get_image_resource_texture2d: unsafe extern "C" fn(resource: LPCWSTR) -> *mut c_void,

    /// 画像リソースをコピーする
    pub copy_image_resource:
        unsafe extern "C" fn(dst_resource: LPCWSTR, src_resource: LPCWSTR) -> bool,

    /// 画像リソースをクリアする
    pub clear_image_resource: unsafe extern "C" fn(resource: LPCWSTR, color: PIXEL_RGBA) -> bool,

    /// 指定の画像リソースを描画先の画像リソースに描画します
    pub draw_image_to_resource: unsafe extern "C" fn(
        dst_resource: LPCWSTR,
        src_resource: LPCWSTR,
        x: f32,
        y: f32,
        z: f32,
        rx: f32,
        ry: f32,
        rz: f32,
        sx: f32,
        sy: f32,
        sz: f32,
        alpha: f32,
    ) -> bool,

    /// 指定の頂点リストのポリゴンを描画先の画像リソースに描画します
    pub draw_poly_to_resource: unsafe extern "C" fn(
        dst_resource: LPCWSTR,
        vertex_type: VERTEX_TYPE,
        vertex_list: *const c_void,
        vertex_num: i32,
        src_resource: LPCWSTR,
    ) -> bool,

    /// ピクセルシェーダーを実行します
    pub exec_pixelshader_file: unsafe extern "C" fn(
        cso_file: LPCWSTR,
        target: LPCWSTR,
        resource_list: *mut LPCWSTR,
        resource_num: i32,
        constant: *mut c_void,
        constant_size: i32,
        blend_state: *mut c_void,
        sampler_state: *mut c_void,
    ) -> bool,

    /// コンピュートシェーダーを実行します
    pub exec_computeshader_file: unsafe extern "C" fn(
        cso_file: LPCWSTR,
        target_list: *mut LPCWSTR,
        target_num: i32,
        resource_list: *mut LPCWSTR,
        resource_num: i32,
        constant: *mut c_void,
        constant_size: i32,
        count_x: i32,
        count_y: i32,
        count_z: i32,
        sampler_state: *mut c_void,
    ) -> bool,

    /// 定義済みのD3Dの出力ブレンドのリソースのポインタを取得する
    pub get_blend_state: unsafe extern "C" fn(blend: BLEND_STATE_MODE) -> *mut c_void,

    /// 定義済みのD3Dのサンプラーのリソースのポインタを取得する
    pub get_sampler_state: unsafe extern "C" fn(sampler: SAMPLER_MODE) -> *mut c_void,

    /// ピクセルシェーダーを実行します
    pub exec_pixelshader_data: unsafe extern "C" fn(
        data: *const u8,
        data_size: i32,
        target: LPCWSTR,
        resource_list: *mut LPCWSTR,
        resource_num: i32,
        constant: *mut c_void,
        constant_size: i32,
        blend_state: *mut c_void,
        sampler_state: *mut c_void,
    ) -> bool,

    /// コンピュートシェーダーを実行します
    pub exec_computeshader_data: unsafe extern "C" fn(
        data: *const u8,
        data_size: i32,
        target_list: *mut LPCWSTR,
        target_num: i32,
        resource_list: *mut LPCWSTR,
        resource_num: i32,
        constant: *mut c_void,
        constant_size: i32,
        count_x: i32,
        count_y: i32,
        count_z: i32,
        sampler_state: *mut c_void,
    ) -> bool,

    /// 指定の画像リソースのサイズを取得する
    pub get_image_resource_size:
        unsafe extern "C" fn(resource: LPCWSTR, width: *mut i32, height: *mut i32) -> bool,

    /// 画像リソースから指定フォーマットの画像データを取得する
    pub get_image_resource_data: unsafe extern "C" fn(
        resource: LPCWSTR,
        buffer: *mut c_void,
        width: i32,
        height: i32,
        pitch: i32,
        format: OUTPUT_PIXEL_FORMAT,
    ) -> bool,

    /// 画像リソースに指定フォーマットの画像データを設定する
    pub set_image_resource_data: unsafe extern "C" fn(
        resource: LPCWSTR,
        buffer: *const c_void,
        width: i32,
        height: i32,
        pitch: i32,
        format: INPUT_PIXEL_FORMAT,
    ) -> bool,

    /// 登録されているフォントのDirectWriteのフォントのポインタを取得する
    /// font: フォント名 ※アプリケーション内の登録名
    pub get_font: unsafe extern "C" fn(font: LPCWSTR) -> *mut c_void,
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

    /// 編集セクション関数
    pub edit: *mut EDIT_SECTION,

    /// 現在のオブジェクトの音声パラメータ情報
    pub param: *mut OBJECT_AUDIO_PARAM,

    /// 指定オブジェクトの音声出力項目のパラメータを取得する
    pub get_output_audio_param: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        offset: f64,
        param: *mut OBJECT_AUDIO_PARAM,
        param_size: i32,
    ) -> bool,

    /// 指定のレイヤー位置にある音声オブジェクトを取得します
    pub get_audio_object: unsafe extern "C" fn(layer: i32, offset: f64) -> OBJECT_HANDLE,
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
