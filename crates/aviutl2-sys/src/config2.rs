#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::{common::LPCWSTR, plugin2::LPCSTR};

/// フォント情報構造体
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FONT_INFO {
    /// フォント名
    pub name: LPCWSTR,
    /// フォントサイズ
    pub size: f32,
}

/// 設定ハンドル
#[repr(C)]
pub struct CONFIG_HANDLE {
    /// アプリケーションデータフォルダのパス
    pub app_data_path: LPCWSTR,

    /// 現在の言語設定で定義されているテキストを取得します
    /// 参照する言語設定のセクションはInitializeConfig()を定義したプラグインのファイル名になります
    /// text : 元のテキスト(.aul2ファイルのキー名)
    /// 戻り値 : 定義されているテキストへのポインタ (未定義の場合は引数のテキストのポインタが返却されます)
    ///         ※言語設定が更新されるまで有効
    pub translate: unsafe extern "C" fn(handle: *mut CONFIG_HANDLE, text: LPCWSTR) -> LPCWSTR,

    /// 現在の言語設定で定義されているテキストを取得します ※任意のセクションから取得出来ます
    /// section : 言語設定のセクション(.aul2ファイルのセクション名)
    /// text : 元のテキスト(.aul2ファイルのキー名)
    /// 戻り値 : 定義されているテキストへのポインタ (未定義の場合は引数のテキストのポインタが返却されます)
    ///         ※言語設定が更新されるまで有効
    pub get_language_text: unsafe extern "C" fn(
        handle: *mut CONFIG_HANDLE,
        section: LPCWSTR,
        text: LPCWSTR,
    ) -> LPCWSTR,

    /// 設定ファイルで定義されているフォント情報を取得します
    /// key : 設定ファイル(style.conf)の[Font]のキー名
    /// 戻り値 : フォント情報構造体へのポインタ (取得出来ない場合はデフォルトのフォントが返却されます)
    ///         ※次にこの関数を呼び出すまで有効
    pub get_font_info:
        unsafe extern "C" fn(handle: *mut CONFIG_HANDLE, key: LPCSTR) -> *mut FONT_INFO,

    /// 設定ファイルで定義されている色コードを取得します
    /// key : 設定ファイル(style.conf)の[Color]のキー名
    /// 戻り値 : 定義されている色コードの値 (取得出来ない場合は0が返却されます)
    pub get_color_code: unsafe extern "C" fn(handle: *mut CONFIG_HANDLE, key: LPCSTR) -> i32,

    /// 設定ファイルで定義されているレイアウトサイズを取得します
    /// key : 設定ファイル(style.conf)の[Layout]のキー名
    /// 戻り値 : 定義されているサイズ (取得出来ない場合は0が返却されます)
    pub get_layout_size: unsafe extern "C" fn(handle: *mut CONFIG_HANDLE, key: LPCSTR) -> i32,
}
