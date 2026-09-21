#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::{ffi::c_void, mem::MaybeUninit};

use crate::{
    common::LPCWSTR,
    plugin2::{EDIT_SECTION, LPCSTR},
};

/// オブジェクトハンドル
pub type OBJECT_HANDLE = *mut c_void;

#[repr(C)]
pub union FILTER_ITEM {
    pub track: FILTER_ITEM_TRACK,
    pub track_group: FILTER_ITEM_TRACK_GROUP,
    pub check: FILTER_ITEM_CHECK,
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
    pub hide_rule: FILTER_ITEM_HIDE_RULE,
}

/// トラックバー項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_TRACK {
    /// 設定の種別（`L"track2"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値に更新されます)
    pub value: f64,
    /// 設定値の最小、最大
    pub s: f64,
    /// 設定値の最小、最大
    pub e: f64,
    /// 設定値の単位( 1.0 / 0.1 / 0.01 / 0.001 ) ※0.0001以下も指定出来ますが最大最小値の範囲に応じて調整されます
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
    /// 設定の種別（`L"trackgroup"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// トラックバー項目グループ (FILTER_ITEM_TRACKポインタを列挙してnull終端したリストへのポインタ) ※2か3項目のみ
    pub tracks: *mut *mut FILTER_ITEM_TRACK,
}

/// チェックボックス項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_CHECK {
    /// 設定の種別（`L"check"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値に更新されます)
    pub value: bool,
}

/// チェックボックス(セクション毎)項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_CHECK_SECTION {
    /// 設定の種別（`L"checksection2"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値に更新されます)
    pub value: bool,
    /// セクション毎設定の初期値
    pub multi_section: bool,
}

/// 色選択項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_COLOR {
    /// 設定の種別（`L"color"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値に更新されます)
    pub value: FILTER_ITEM_COLOR_VALUE,
}

/// 設定値の色
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
    /// 設定の種別（`L"select"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値に更新されます)
    pub value: i32,
    /// 選択肢リスト (ITEMを列挙して名前がnullのITEMで終端したリストへのポインタ)
    pub items: *const FILTER_ITEM_SELECT_ITEM,
}

/// 選択肢項目
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
    /// 設定の種別（`L"file"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値のポインタに更新されます)
    pub value: LPCWSTR,
    /// ファイルフィルタ
    pub filefilter: LPCWSTR,
}

/// 汎用データ項目構造体 (設定が表示されない項目になります)
/// フィルタ処理関数内でvalueの参照先データを更新することが出来ます
/// ※Undoポイントの作成や編集済みフラグの設定はされません
/// ※保存済みの汎用データが空の場合はデフォルト値に初期化されます
/// フィルタ処理関数内のset_filter_item_data_size()関数でデータのサイズを変更出来ます
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_DATA {
    /// 設定の種別（`L"data"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 汎用データのポインタ (フィルタ処理の呼び出し時に現在の汎用データのポインタに更新されます)
    pub value: *mut c_void,
    /// 汎用データのサイズ (フィルタ処理の呼び出し時に現在の汎用データのサイズに更新されます) ※16KB以下
    pub size: i32,
    /// デフォルト値 (Tの定義でデフォルト値を指定しておく)
    pub default_value: [MaybeUninit<u8>; 16 * 1024],
}

/// 設定グループ項目構造体
/// 自身以降の設定項目をグループ化することが出来ます
/// ※設定名を空にするとグループの終端を定義することが出来ます
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_GROUP {
    /// 設定の種別（`L"group"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// デフォルトの表示状態
    pub default_visible: bool,
}

/// ボタン項目構造体
/// ボタンを押すとコールバック関数が呼ばれます ※plugin2.hの編集のコールバック関数と同様な形になります
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_BUTTON {
    /// 設定の種別（`L"button"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// ボタンを押した時のコールバック関数 (呼び出し時に各設定項目の設定値が更新されます)
    pub callback: Option<extern "C" fn(edit_section: *mut EDIT_SECTION)>,
    /// 同上
    pub callback2: Option<
        extern "C" fn(
            edit: *mut EDIT_SECTION,
            object: OBJECT_HANDLE,
            effect: LPCWSTR,
            item: LPCWSTR,
        ),
    >,
}

/// 文字列項目構造体 ※1行の文字列
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_STRING {
    /// 設定の種別（`L"string"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値のポインタに更新されます)
    pub value: LPCWSTR,
}

/// テキスト項目構造体 ※複数行の文字列
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_TEXT {
    /// 設定の種別（`L"text"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値のポインタに更新されます)
    pub value: LPCWSTR,
}

/// フォルダ選択項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_FOLDER {
    /// 設定の種別（`L"folder"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
    /// 設定値 (フィルタ処理の呼び出し時に現在の値のポインタに更新されます)
    pub value: LPCWSTR,
}

/// セパレーター項目構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_SEPARATOR {
    /// 設定の種別（`L"separator"`）
    pub r#type: LPCWSTR,
    /// 設定名
    pub name: LPCWSTR,
}

/// 非表示条件項目構造体
/// 非表示条件を満たした項目を非表示にすることが出来ます
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILTER_ITEM_HIDE_RULE {
    /// 設定の種別（`L"hiderule"`）
    pub r#type: LPCWSTR,
    /// 非表示にする設定名 ※設定値を持つ項目のみ
    pub name: LPCWSTR,
    /// 非表示の条件の設定名
    /// チェックボックス項目,リスト選択項目,ファイル選択項目,フォルダ選択項目
    /// ※セクション毎のチェックボックスはセクション毎が有効の場合は2を返却(0/1/2)
    /// ※ファイル,フォルダ選択項目は選択されているかを返却(0/1)
    /// ※nullptrを指定した場合は常に非表示
    /// ※"filter"を指定した場合はフィルタオブジェクトかを返却(0/1)
    pub condition_name: LPCWSTR,
    /// 非表示の条件の比較種別
    pub condition_operator: FILTER_ITEM_HIDE_RULE_OPERATOR,
    /// 非表示の条件の比較値
    pub condition_value: i32,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FILTER_ITEM_HIDE_RULE_OPERATOR {
    /// == (等しい)
    EQUAL = 0,
    /// != (等しくない)
    NOT_EQUAL = 1,
    /// > (より大きい)
    GREATER = 2,
    /// < (より小さい)
    LESS = 3,
}

/// 頂点データ構造体(描画色)
/// { x, y, z, r, g, b, a }
#[repr(C)]
pub struct VERTEX_COLOR {
    /// 頂点座標
    pub x: f32,
    /// 頂点座標
    pub y: f32,
    /// 頂点座標
    pub z: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub r: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub g: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub b: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub a: f32,
}

/// 頂点データ構造体(描画色、法線)
/// { x, y, z, r, g, b, a, vx, vy, vz }
#[repr(C)]
pub struct VERTEX_COLOR_NORM {
    /// 頂点座標
    pub x: f32,
    /// 頂点座標
    pub y: f32,
    /// 頂点座標
    pub z: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub r: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub g: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub b: f32,
    /// 頂点の色(0.0〜1.0の乗算済みα)
    pub a: f32,
    /// 法線ベクトル
    pub vx: f32,
    /// 法線ベクトル
    pub vy: f32,
    /// 法線ベクトル
    pub vz: f32,
}

/// 頂点データ構造体(テクスチャ)
/// { x, y, z, u, v, a }
#[repr(C)]
pub struct VERTEX_TEXTURE {
    /// 頂点座標
    pub x: f32,
    /// 頂点座標
    pub y: f32,
    /// 頂点座標
    pub z: f32,
    /// テクスチャー座標(0.0〜1.0の正規化座標)
    pub u: f32,
    /// テクスチャー座標(0.0〜1.0の正規化座標)
    pub v: f32,
    /// 頂点のα値
    pub a: f32,
}

/// 頂点データ構造体(テクスチャ、法線)
/// { x, y, z, u, v, a, vx, vy, vz }
#[repr(C)]
pub struct VERTEX_TEXTURE_NORM {
    /// 頂点座標
    pub x: f32,
    /// 頂点座標
    pub y: f32,
    /// 頂点座標
    pub z: f32,
    /// テクスチャー座標(0.0〜1.0の正規化座標)
    pub u: f32,
    /// テクスチャー座標(0.0〜1.0の正規化座標)
    pub v: f32,
    /// 頂点のα値
    pub a: f32,
    /// 法線ベクトル
    pub vx: f32,
    /// 法線ベクトル
    pub vy: f32,
    /// 法線ベクトル
    pub vz: f32,
}

/// 頂点リストの種別
#[repr(i32)]
pub enum VERTEX_TYPE {
    /// 三角形のVERTEX_COLORのリスト (頂点数は3の倍数になる)
    TRIANGLE_COLOR = 1,
    /// 三角形のVERTEX_COLOR_NORMのリスト (頂点数は3の倍数になる)
    TRIANGLE_COLOR_NORM = 2,
    /// 三角形のVERTEX_TEXTUREのリスト (頂点数は3の倍数になる)
    TRIANGLE_TEXTURE = 3,
    /// 三角形のVERTEX_TEXTURE_NORMのリスト (頂点数は3の倍数になる)
    TRIANGLE_TEXTURE_NORM = 4,
    /// 四角形のVERTEX_COLORのリスト (頂点数は4の倍数になる)
    QUAD_COLOR = 5,
    /// 四角形のVERTEX_COLOR_NORMのリスト (頂点数は4の倍数になる)
    QUAD_COLOR_NORM = 6,
    /// 四角形のVERTEX_TEXTUREのリスト (頂点数は4の倍数になる)
    QUAD_TEXTURE = 7,
    /// 四角形のVERTEX_TEXTURE_NORMのリスト (頂点数は4の倍数になる)
    QUAD_TEXTURE_NORM = 8,
}

/// サンプラー(SampleState)の種別
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
    /// 拡大縮小補間をしない(領域外は透明色)
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
    /// 比較(明)
    LIGHT = 6,
    /// 比較(暗)
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
    /// 標準の向き(何もしない)
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
    /// オブジェクトの現在の画像サイズ (画像フィルタのみ)
    pub width: i32,
    /// オブジェクトの現在の画像サイズ (画像フィルタのみ)
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
    /// オブジェクトの現在のレイヤー番号 ※描画対象のオブジェクトのレイヤー番号
    pub layer: i32,
    /// 複数オブジェクト時の現在の対象番号 ※個別オブジェクト用
    pub index: i32,
    /// 複数オブジェクト時の対象数 (1 = 単体オブジェクト / 0 = 不定) ※個別オブジェクト用
    pub num: i32,
    /// 全体(シーン)基準のオブジェクトの開始フレーム(0からの番号)
    pub frame_s: i32,
    /// 全体(シーン)基準のオブジェクトの終了フレーム(0からの番号)
    pub frame_e: i32,
    /// 対象エフェクトの現在のレイヤー番号 ※自身のオブジェクトのレイヤー番号
    pub effect_layer: i32,
    /// 全体(シーン)基準のレンダリングの起点フレーム(0からの整数)
    pub origin_frame: i32,
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
    /// 基準座標
    pub y: f32,
    /// 基準座標
    pub z: f32,
    /// 回転角度 (360.0で1回転)
    pub rx: f32,
    /// 回転角度 (360.0で1回転)
    pub ry: f32,
    /// 回転角度 (360.0で1回転)
    pub rz: f32,
    /// 拡大率 (1.0=等倍)
    pub sx: f32,
    /// 拡大率 (1.0=等倍)
    pub sy: f32,
    /// 拡大率 (1.0=等倍)
    pub sz: f32,
    /// 中心座標 (基準座標からの相対)
    pub cx: f32,
    /// 中心座標 (基準座標からの相対)
    pub cy: f32,
    /// 中心座標 (基準座標からの相対)
    pub cz: f32,
    /// 不透明度 (0.0〜1.0/0.0=透明/1.0=不透明)
    pub alpha: f32,
}

/// オブジェクトの音声パラメータ構造体
#[repr(C)]
pub struct OBJECT_AUDIO_PARAM {
    /// 音量倍率 (1.0=等倍)
    pub vol_l: f32,
    /// 音量倍率 (1.0=等倍)
    pub vol_r: f32,
}

/// エフェクト実行の設定パラメータ構造体
#[repr(C)]
pub struct EFFECT_ITEM_PARAM {
    /// 設定名 ※エイリアスファイルの設定のキーの名称
    pub name: LPCWSTR,
    /// 設定値(UTF8) ※エイリアスファイルの設定値と同じフォーマット
    pub value: LPCSTR,
}

/// 画像フィルタ処理用構造体
#[repr(C)]
pub struct FILTER_PROC_VIDEO {
    /// シーン情報
    pub scene: *const SCENE_INFO,

    /// オブジェクト情報
    pub object: *const OBJECT_INFO,

    /// 現在のオブジェクトの画像データをPIXEL_RGBA形式で取得する (VRAMからデータを取得します)
    /// buffer : 画像データの格納先へのポインタ
    pub get_image_data: unsafe extern "C" fn(buffer: *mut PIXEL_RGBA),

    /// 現在のオブジェクトの画像データをPIXEL_RGBA形式で設定する (VRAMへデータを書き込みます)
    /// buffer : 画像データへのポインタ (nullptrの場合は初期データ無しで画像サイズを変更します)
    /// width,height : 画像サイズ
    pub set_image_data: unsafe extern "C" fn(buffer: *const PIXEL_RGBA, width: i32, height: i32),

    // 現在のオブジェクトの画像データのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    // 戻り値		: オブジェクトの画像データへのポインタ
    //				  ※現在の画像が変更(set_image_data)されるかフィルタ処理の終了まで有効
    /// 現在のオブジェクトのD3D画像リソースのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    /// 戻り値 : オブジェクト画像のID3D11Texture2Dのポインタ
    /// ※現在の画像が変更(set_image_data)されるかフィルタ処理の終了まで有効
    pub get_image_texture2d: unsafe extern "C" fn() -> *mut c_void,

    // 現在のフレームバッファの画像データのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    // 戻り値		: フレームバッファの画像データへのポインタ
    //				  ※フィルタ処理の終了まで有効
    /// 現在のフレームバッファのD3D画像リソースのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    /// 戻り値 : フレームバッファのID3D11Texture2Dのポインタ
    /// ※フィルタ処理の終了まで有効
    pub get_framebuffer_texture2d: unsafe extern "C" fn() -> *mut c_void,

    /// 編集セクション関数
    /// フィルタ処理中は参照系の関数が利用出来ます
    pub edit: *mut EDIT_SECTION,

    /// 現在のオブジェクトの画像パラメータ情報
    /// パラメータを直接変更することが出来ます
    /// ※このパラメータは画像出力項目のパラメータからの相対設定になります (スクリプトのobj.ox等と同じ)
    pub param: *mut OBJECT_IMAGE_PARAM,

    /// 指定オブジェクトの画像出力項目のパラメータを取得する
    /// object : 対象のオブジェクトのハンドル (nullptrを指定すると現在のオブジェクトが対象)
    /// ※フィルタ処理対象のシーンにあるオブジェクトのみ取得出来ます
    /// offset : 取得時間のオフセット(秒) (0なら現時間)
    /// output : パラメータの格納先へのポインタ
    /// output_size : パラメータの格納先のサイズ ※サイズ分のみ取得されます
    /// 戻り値 : 取得出来ない場合はfalse (画像オブジェクト以外が指定された場合)
    pub get_output_image_param: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        offset: f64,
        param: *mut OBJECT_IMAGE_PARAM,
        param_size: i32,
    ) -> bool,

    /// 指定のレイヤーにある画像オブジェクトを取得します
    /// layer : 対象のレイヤー番号
    /// offset : 取得時間のオフセット(秒) (0なら現時間)
    /// 戻り値 : 取得したオブジェクトのハンドル (存在しない場合はnullptrを返却)
    pub get_image_object: unsafe extern "C" fn(layer: i32, offset: f64) -> OBJECT_HANDLE,

    /// 指定の画像リソースをフレームバッファに描画します
    /// resource : 画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// x,y,z : 基準座標
    /// rx,ry,rz : 回転角度 (360.0で1回転)
    /// sx,sy,sz : 拡大率 (1.0=等倍)
    /// alpha : 不透明度 (0.0〜1.0/0.0=透明/1.0=不透明)
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
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
    /// vertex_type : 頂点リストの種別
    /// vertex_list : 頂点データリストへのポインタ (指定した種別の頂点データバッファへのポインタ)
    /// vertex_num : 頂点リストの頂点数 (頂点データの数)
    /// resource : テクスチャ画像リソース名 ※テクスチャ付きの場合のみ利用
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// 戻り値 : 失敗した場合はfalse (頂点数が不正な場合等)
    pub draw_poly: unsafe extern "C" fn(
        vertex_type: VERTEX_TYPE,
        vertex_list: *const c_void,
        vertex_num: i32,
        image: LPCWSTR,
    ) -> bool,

    /// 標準のアンカー枠を設定します ※func_proc_video()でtrueを返却した場合は自動で設定されます
    /// draw_image()などを利用してオブジェクトの描画を全て自身で処理する場合に利用します
    /// width,height : オブジェクトのサイズ ※0を指定すると固定サイズのアンカー枠になります
    pub set_default_anchor: unsafe extern "C" fn(width: i32, height: i32),

    /// 描画時の合成モードを設定します
    /// フレームバッファへの描画の合成モードは元々の合成モードが通常の場合のみ反映されます
    /// 合成モードを利用すると描画処理が重くなります
    /// blend : 合成モード
    pub set_blend_mode: unsafe extern "C" fn(blend: BLEND_MODE),

    /// 描画時の光沢度を設定します
    /// カメラ制御の光源設定が有効の時に利用されます
    /// shine : 光沢度(0.0〜1.0)
    pub set_material_shine: unsafe extern "C" fn(shine: f32),

    /// 描画時のサンプラーを設定します
    /// sampler : サンプラー種別
    pub set_sampler_mode: unsafe extern "C" fn(sampler: SAMPLER_MODE),

    /// 描画時に裏面を非表示にするかを設定します
    /// culling : 裏面を非表示にするか？
    pub set_culling_state: unsafe extern "C" fn(culling: bool),

    /// 描画時にオブジェクトをカメラの方向に向けるかを設定します
    /// billboard : ビルボード種別
    pub set_billboard_mode: unsafe extern "C" fn(billboard: BILLBOARD_MODE),

    /// 画像リソースを作成する (VRAMへデータを書き込みます)
    /// 画像リソースはdraw_image()などの描画に利用出来ます
    /// ※既に存在する画像リソース名を指定した場合はリソースを更新します
    /// resource : 作成する画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前) ※フィルタ処理後に破棄されます
    /// "tempbuffer" = 仮想バッファ ※内部実装はキャッシュバッファと同じ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前) ※レンダリング処理共用のキャッシュバッファ
    /// buffer : 画像データへのポインタ ※PIXEL_RGBA形式 (nullptrの場合は初期データ無しで作成します)
    /// width,height : 画像サイズ
    pub create_image_resource:
        unsafe extern "C" fn(image: LPCWSTR, buffer: *const PIXEL_RGBA, width: i32, height: i32),

    /// 指定の画像リソースのD3D画像リソースのポインタを取得する (ID3D11Texture2Dのポインタを取得します)
    /// resource : 画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// "random" = 乱数バッファ(0.0〜1.0の乱数値の256x256の領域) ※DXGI_FORMAT_R32_FLOAT(r値のみ)になります
    /// 戻り値 : 画像リソースのID3D11Texture2Dのポインタ (指定リソースが無い場合はnullptrを返却)
    /// ※画像リソースが変更されるかフィルタ処理の終了まで有効
    pub get_image_resource_texture2d: unsafe extern "C" fn(resource: LPCWSTR) -> *mut c_void,

    /// 画像リソースをコピーする
    /// dst_resource : コピー先の画像リソース名
    /// ※既に存在する画像リソース名を指定した場合はリソースを更新します
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前) ※フィルタ処理後に破棄されます
    /// "tempbuffer" = 仮想バッファ ※内部実装はキャッシュバッファと同じ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前) ※レンダリング処理共用のキャッシュバッファ
    /// src_resource : コピー元の画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "framebuffer" = フレームバッファ
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// "random" = 乱数バッファ(0.0〜1.0の乱数値の256x256の領域) ※DXGI_FORMAT_R32_FLOAT(r値のみ)になります
    /// "layer:xxxx[+]" = レイヤー上のオブジェクト(xxxxはレイヤー番号、末尾に"+"指定で追加フィルタを実行)
    /// "before" = 直前オブジェクト
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
    pub copy_image_resource:
        unsafe extern "C" fn(dst_resource: LPCWSTR, src_resource: LPCWSTR) -> bool,

    /// 画像リソースをクリアする
    /// resource : クリアする画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// color : クリアする色
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
    pub clear_image_resource: unsafe extern "C" fn(resource: LPCWSTR, color: PIXEL_RGBA) -> bool,

    /// 指定の画像リソースを描画先の画像リソースに描画します
    /// dst_resource : 描画先の画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// src_resource : 画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// x,y,z : 基準座標
    /// rx,ry,rz : 回転角度 (360.0で1回転)
    /// sx,sy,sz : 拡大率 (1.0=等倍)
    /// alpha : 不透明度 (0.0〜1.0/0.0=透明/1.0=不透明)
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
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
    /// dst_resource : 描画先の画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// vertex_type : 頂点リストの種別
    /// vertex_list : 頂点データリストへのポインタ (指定した種別の頂点データバッファへのポインタ)
    /// vertex_num : 頂点リストの頂点数 (頂点データの数)
    /// src_resource : テクスチャ画像リソース名 ※テクスチャ付きの場合のみ利用
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// 戻り値 : 失敗した場合はfalse (頂点数が不正な場合等)
    pub draw_poly_to_resource: unsafe extern "C" fn(
        dst_resource: LPCWSTR,
        vertex_type: VERTEX_TYPE,
        vertex_list: *const c_void,
        vertex_num: i32,
        src_resource: LPCWSTR,
    ) -> bool,

    /// ピクセルシェーダーを実行します
    /// cso_file : コンパイル済みピクセルシェーダーのバイナリファイル名 ※ファイル名部分のみ
    /// プラグインと同じフォルダのファイルから読み込んでキャッシュします
    /// ※ピクセルシェーダー5.0でコンパイルしたものが利用出来ます
    /// ピクセルシェーダーの入力は下記が利用できます
    /// float4 psmain(float4 pos : SV_Position) : SV_Target
    /// float4 psmain(float4 pos : SV_Position, float2 uv : TEXCOORD) : SV_Target
    /// ※シェーダーリフレクションを利用してシグネチャから判別しています
    /// target : 出力先の画像リソース名
    /// Direct3Dのレンダーターゲットに設定されます
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "framebuffer" = フレームバッファ
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// resource_list : 参照する画像リソース名のリストへのポインタ ※nullptrの場合は設定無し
    /// Direct3Dのシェーダーリソース(t0〜)に設定されます ※レンダーターゲットと同じリソースは利用出来ません
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// "random" = 乱数バッファ(0.0〜1.0の乱数値の256x256の領域) ※DXGI_FORMAT_R32_FLOAT(r値のみ)になります
    /// resource_num : 参照する画像リソースの数
    /// constant : 定数バッファへのポインタ ※nullptrの場合は定数バッファの設定無し
    /// Direct3Dの定数バッファ(b0)に設定します
    /// constant_size : 定数バッファのサイズ
    /// blend_state : Direct3DのBlendStateを設定します ※nullptrの場合は出力をそのままコピー
    /// sampler_state : Direct3DのSamplerState(s0)を設定します ※nullptrの場合は設定無し
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
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
    /// cso_file : コンパイル済みコンピュートシェーダーのバイナリファイル名 ※ファイル名部分のみ
    /// プラグインと同じフォルダのファイルから読み込んでキャッシュします
    /// ※コンピュートシェーダー5.0でコンパイルしたものが利用出来ます
    /// target_list : 読み書き先の画像リソース名のリストへのポインタ
    /// Direct3DのUnorderedAccessリソース(u0〜)に設定されます
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "framebuffer" = フレームバッファ
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// target_num : 読み書き先の画像リソースの数
    /// resource_list : 参照する画像リソース名のリストへのポインタ ※nullptrの場合は設定無し
    /// Direct3Dのシェーダーリソース(t0〜)に設定されます ※UnorderedAccessリソースと同じリソースは利用出来ません
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// "random" = 乱数バッファ(0.0〜1.0の乱数値の256x256の領域) ※DXGI_FORMAT_R32_FLOAT(r値のみ)になります
    /// resource_num : 参照する画像リソースの数
    /// constant : 定数バッファへのポインタ ※nullptrの場合は定数バッファの設定無し
    /// Direct3Dの定数バッファ(b0)に設定します
    /// constant_size : 定数バッファのサイズ
    /// count_x : X軸スレッドグループ数
    /// count_y : Y軸スレッドグループ数
    /// count_z : Z軸スレッドグループ数
    /// sampler_state : Direct3DのSamplerState(s0)を設定します ※nullptrの場合は設定無し
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
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

    /// 定義済みのD3Dの出力ブレンドのリソースのポインタを取得する (ID3D11BlendStateのポインタを取得します)
    /// blend : 出力ブレンド種別
    /// 戻り値 : ID3D11BlendStateのポインタ (指定種別が無い場合はnullptrを返却)
    pub get_blend_state: unsafe extern "C" fn(blend: BLEND_STATE_MODE) -> *mut c_void,

    /// 定義済みのD3Dのサンプラーのリソースのポインタを取得する (ID3D11SamplerStateのポインタを取得します)
    /// sampler : サンプラー種別
    /// 戻り値 : ID3D11SamplerStateのポインタ (指定種別が無い場合はnullptrを返却)
    pub get_sampler_state: unsafe extern "C" fn(sampler: SAMPLER_MODE) -> *mut c_void,

    /// ピクセルシェーダーを実行します
    /// data : コンパイル済みピクセルシェーダーのデータへのポインタ(ヘッダーファイルとして出力したデータを利用する)
    /// data_size : コンパイル済みピクセルシェーダーのサイズ
    /// ※ピクセルシェーダー5.0でコンパイルしたものが利用出来ます
    /// ピクセルシェーダーの入力は下記が利用できます
    /// float4 psmain(float4 pos : SV_Position) : SV_Target
    /// float4 psmain(float4 pos : SV_Position, float2 uv : TEXCOORD) : SV_Target
    /// ※シェーダーリフレクションを利用してシグネチャから判別しています
    /// target : 出力先の画像リソース名
    /// Direct3Dのレンダーターゲットに設定されます
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "framebuffer" = フレームバッファ
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// resource_list : 参照する画像リソース名のリストへのポインタ ※nullptrの場合は設定無し
    /// Direct3Dのシェーダーリソース(t0〜)に設定されます ※レンダーターゲットと同じリソースは利用出来ません
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// "random" = 乱数バッファ(0.0〜1.0の乱数値の256x256の領域) ※DXGI_FORMAT_R32_FLOAT(r値のみ)になります
    /// resource_num : 参照する画像リソースの数
    /// constant : 定数バッファへのポインタ ※nullptrの場合は定数バッファの設定無し
    /// Direct3Dの定数バッファ(b0)に設定します
    /// constant_size : 定数バッファのサイズ
    /// blend_state : Direct3DのBlendStateを設定します ※nullptrの場合は出力をそのままコピー
    /// sampler_state : Direct3DのSamplerState(s0)を設定します ※nullptrの場合は設定無し
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
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
    /// data : コンパイル済みコンピュートシェーダーのデータへのポインタ(ヘッダーファイルとして出力したデータを利用する)
    /// data_size : コンパイル済みコンピュートシェーダーのサイズ
    /// ※コンピュートシェーダー5.0でコンパイルしたものが利用出来ます
    /// target_list : 読み書き先の画像リソース名のリストへのポインタ
    /// Direct3DのUnorderedAccessリソース(u0〜)に設定されます
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "framebuffer" = フレームバッファ
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// target_num : 読み書き先の画像リソースの数
    /// resource_list : 参照する画像リソース名のリストへのポインタ ※nullptrの場合は設定無し
    /// Direct3Dのシェーダーリソース(t0〜)に設定されます ※UnorderedAccessリソースと同じリソースは利用出来ません
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// "random" = 乱数バッファ(0.0〜1.0の乱数値の256x256の領域) ※DXGI_FORMAT_R32_FLOAT(r値のみ)になります
    /// resource_num : 参照する画像リソースの数
    /// constant : 定数バッファへのポインタ ※nullptrの場合は定数バッファの設定無し
    /// Direct3Dの定数バッファ(b0)に設定します
    /// constant_size : 定数バッファのサイズ
    /// count_x : X軸スレッドグループ数
    /// count_y : Y軸スレッドグループ数
    /// count_z : Z軸スレッドグループ数
    /// sampler_state : Direct3DのSamplerState(s0)を設定します ※nullptrの場合は設定無し
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
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
    /// resource : 画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// "tempbuffer" = 仮想バッファ
    /// "cache:xxxx" = キャッシュバッファ(xxxxは任意の名前)
    /// "image:xxxx" = 画像ファイル(xxxxは画像ファイルパス) ※画像はVRAMにキャッシュされます
    /// width,height : 画像サイズの格納先へのポインタ
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
    pub get_image_resource_size:
        unsafe extern "C" fn(resource: LPCWSTR, width: *mut i32, height: *mut i32) -> bool,

    /// 画像リソースから指定フォーマットの画像データを取得する (VRAMからデータを取得します)
    /// resource : 画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// buffer : 画像データの格納先へのポインタ
    /// width,height : 画像データの格納先のサイズ ※画像リソースとサイズが一致する場合のみ取得出来ます
    /// pitch : 画像データの格納先の横1ラインのバイト数
    /// format : 取得する画像データのピクセルフォーマット
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
    pub get_image_resource_data: unsafe extern "C" fn(
        resource: LPCWSTR,
        buffer: *mut c_void,
        width: i32,
        height: i32,
        pitch: i32,
        format: OUTPUT_PIXEL_FORMAT,
    ) -> bool,

    /// 画像リソースに指定フォーマットの画像データを設定する (VRAMへデータを書き込みます)
    /// ※存在しない画像リソース名を指定した場合は新規作成します
    /// resource : 作成する画像リソース名
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前) ※フィルタ処理後に破棄されます
    /// buffer : 画像データへのポインタ
    /// width,height : 画像サイズ
    /// pitch : 画像データの横1ラインのバイト数
    /// format : 画像データのピクセルフォーマット
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
    pub set_image_resource_data: unsafe extern "C" fn(
        resource: LPCWSTR,
        buffer: *const c_void,
        width: i32,
        height: i32,
        pitch: i32,
        format: INPUT_PIXEL_FORMAT,
    ) -> bool,

    /// 冗長なので廃止します ※EDIT_SECTIONに移動しました
    #[deprecated = "冗長なので廃止します ※EDIT_SECTIONに移動しました"]
    pub deprecated_get_font: unsafe extern "C" fn(font: LPCWSTR) -> *mut c_void,

    /// 指定の汎用データ項目のデータサイズを変更します
    /// サイズを変更すると汎用データのポインタは更新されます
    /// filter_item_data : 対象のFILTER_ITEM_DATAへのポインタ
    /// size : 汎用データのサイズ
    pub set_filter_item_data_size: unsafe extern "C" fn(filter_item_data: *mut c_void, size: i32),

    /// ユーザーデータのポインタ (FLAG_USERDATAが有効の時に設定されます)
    /// func_create()で返却したユーザーデータのポインタ
    pub userdata: *mut c_void,

    /// 画像リソースを解放します
    /// resource : 解放する画像リソース名
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// 戻り値 : 失敗した場合はfalse (画像リソース名が不正な場合等)
    pub release_image_resource: unsafe extern "C" fn(resource: LPCWSTR) -> bool,

    /// 指定のエフェクトを実行します
    /// フィルタ効果、入力項目(図形等)のエフェクトが実行出来ます
    /// name : 実行するエフェクト名 (エイリアスファイルのeffect.nameの値)
    /// param_list : 設定パラメータのリストへのポインタ ※nullptrの場合は設定無し
    /// param_num : 設定パラメータの数
    /// resource : エフェクトの処理対象の画像リソース名 ※エフェクトが入力項目の場合は新規作成します
    /// "object" = 現在のオブジェクト ※nullptrの指定でも現在のオブジェクトになります
    /// "resource:xxxx" = 標準リソース(xxxxは任意の名前)
    /// 戻り値 : 失敗した場合はfalse (エフェクト名や画像リソース名が不正な場合等)
    pub exec_effect: unsafe extern "C" fn(
        name: LPCWSTR,
        param_list: *mut EFFECT_ITEM_PARAM,
        param_num: i32,
        resource: LPCWSTR,
    ) -> bool,

    /// 描画時の合成モードを設定します
    /// set_blend_mode()と異なりフレームバッファへの描画の合成モードも常に反映されます
    /// 合成モードを利用すると描画処理が重くなります
    /// blend : 合成モード
    pub set_blend_mode_force: unsafe extern "C" fn(blend: BLEND_MODE),

    /// 現在のオブジェクトに影響しているグループ制御オブジェクトを取得します
    /// layer : 上位の影響しているグループ制御のインデックス(0は直前のグループ制御)
    /// 戻り値 : 取得したグループ制御オブジェクトのハンドル (グループ制御対象外の場合はnullptrを返却)
    pub get_group_control_object: unsafe extern "C" fn(index: i32) -> OBJECT_HANDLE,

    /// 現在のオブジェクトに適用されるグループ制御の座標変換行列を取得します (グループ制御対象を結合した行列)
    /// 戻り値 : グループ制御対象外の場合はfalse
    pub get_group_matrix: unsafe extern "C" fn(matrix: *mut DxmMatrix) -> bool,
}

/// DirectXMathのXMMATRIX構造体の互換構造体
#[repr(C, align(16))]
pub struct DxmMatrix {
    pub m: [[f32; 4]; 4],
}

/// 音声フィルタ処理用構造体
#[repr(C)]
pub struct FILTER_PROC_AUDIO {
    /// シーン情報
    pub scene: *const SCENE_INFO,

    /// オブジェクト情報
    pub object: *const OBJECT_INFO,

    /// 現在のオブジェクトの音声データを取得する
    /// buffer : 音声データの格納先へのポインタ ※音声データはPCM(float)32bit
    /// channel : 音声データのチャンネル ( 0 = 左チャンネル / 1 = 右チャンネル )
    pub get_sample_data: unsafe extern "C" fn(buffer: *mut f32, channel: i32),

    /// 現在のオブジェクトの音声データを設定する
    /// buffer : 音声データへのポインタ ※音声データはPCM(float)32bit
    /// channel : 音声データのチャンネル ( 0 = 左チャンネル / 1 = 右チャンネル )
    pub set_sample_data: unsafe extern "C" fn(buffer: *const f32, channel: i32),

    /// 編集セクション関数
    /// フィルタ処理中は参照系の関数が利用出来ます
    pub edit: *mut EDIT_SECTION,

    /// 現在のオブジェクトの音声パラメータ情報
    /// パラメータを直接変更することが出来ます
    /// ※このパラメータは音声出力項目のパラメータからの相対設定になります
    pub param: *mut OBJECT_AUDIO_PARAM,

    /// 指定オブジェクトの音声出力項目のパラメータを取得する
    /// object : 対象のオブジェクトのハンドル (nullptrを指定すると現在のオブジェクトが対象)
    /// ※フィルタ処理対象のシーンにあるオブジェクトのみ取得出来ます
    /// offset : 取得時間のオフセット(秒) (0なら現時間)
    /// param : パラメータの格納先へのポインタ
    /// param_size : パラメータの格納先のサイズ ※サイズ分のみ取得されます
    /// 戻り値 : 取得出来ない場合はfalse (音声オブジェクト以外が指定された場合)
    pub get_output_audio_param: unsafe extern "C" fn(
        object: OBJECT_HANDLE,
        offset: f64,
        param: *mut OBJECT_AUDIO_PARAM,
        param_size: i32,
    ) -> bool,

    /// 指定のレイヤー位置にある音声オブジェクトを取得します
    /// layer : 対象のレイヤー番号
    /// offset : 取得時間のオフセット(秒) (0なら現時間)
    /// 戻り値 : 取得したオブジェクトのハンドル (存在しない場合はnullptrを返却)
    pub get_audio_object: unsafe extern "C" fn(layer: i32, offset: f64) -> OBJECT_HANDLE,

    /// 指定の汎用データ項目のデータサイズを変更します
    /// サイズを変更すると汎用データのポインタは更新されます
    /// filter_item_data : 対象のFILTER_ITEM_DATAへのポインタ
    /// size : 汎用データのサイズ (16KB以下)
    pub set_filter_item_data_size: unsafe extern "C" fn(filter_item_data: *mut c_void, size: i32),

    /// ユーザーデータのポインタ (FLAG_USERDATAが有効の時に設定されます)
    /// func_create()で返却したユーザーデータのポインタ
    pub userdata: *mut c_void,
}

impl FILTER_PLUGIN_TABLE {
    /// 画像フィルタをサポートする
    pub const FLAG_VIDEO: i32 = 1;
    /// 音声フィルタをサポートする
    /// 画像と音声のフィルタ処理は別々のスレッドで処理されます
    pub const FLAG_AUDIO: i32 = 2;
    /// メディアオブジェクトの初期入力をする (メディアオブジェクトにする場合)
    pub const FLAG_INPUT: i32 = 4;
    /// フィルタオブジェクトをサポートする (フィルタオブジェクトに対応する場合)
    /// フィルタオブジェクトの場合は画像サイズの変更が出来ません
    pub const FLAG_FILTER: i32 = 8;
    /// ユーザーデータをサポートする ※func_create(),func_destroy()が呼ばれるようになります
    pub const FLAG_USERDATA: i32 = 16;
    /// オブジェクト・フィルタ効果の追加メニューリストに表示しない
    pub const FLAG_HIDEMENU: i32 = 32;
}

/// フィルタプラグイン構造体
#[repr(C)]
pub struct FILTER_PLUGIN_TABLE {
    /// フラグ
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
    /// 戻り値 : falseを返却すると以降のフィルタや出力処理が中断されます
    pub func_proc_video: Option<extern "C" fn(video: *mut FILTER_PROC_VIDEO) -> bool>,

    /// 音声フィルタ処理関数へのポインタ (FLAG_AUDIOが有効の時のみ呼ばれます)
    /// 戻り値 : falseを返却すると以降のフィルタや出力処理が中断されます
    pub func_proc_audio: Option<extern "C" fn(audio: *mut FILTER_PROC_AUDIO) -> bool>,

    /// エフェクトのインスタンスが生成される時に呼ばれる関数へのポインタ (FLAG_USERDATAが有効の時のみ呼ばれます)
    /// ※オブジェクトと関連しない状態でも呼ばれます
    /// effect_id : エフェクトのID (アプリ起動毎の固有ID)
    /// 戻り値 : 任意のユーザーデータのポインタ
    pub func_create: Option<extern "C" fn(effect_id: i64) -> *mut c_void>,

    /// エフェクトのインスタンスが破棄される時に呼ばれる関数へのポインタ (FLAG_USERDATAが有効の時のみ呼ばれます)
    /// 編集データやUndoバッファ等の全てのインスタンスが破棄されると呼ばれます
    /// ※オブジェクトと関連しない状態でも呼ばれます
    /// effect_id : エフェクトのID (アプリ起動毎の固有ID)
    /// userdata : func_create()で返却したユーザーデータのポインタ
    pub func_destroy: Option<extern "C" fn(effect_id: i64, userdata: *mut c_void)>,
}
