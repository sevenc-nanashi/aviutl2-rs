#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::common::LPCWSTR;

/// ログ出力ハンドル
/// ログ出力は1024文字で制限されます
#[repr(C)]
pub struct LOG_HANDLE {
    /// プラグイン用のログを出力します
    /// handle : ログ出力ハンドル
    /// message : ログメッセージ
    pub log: unsafe extern "C" fn(handle: *mut LOG_HANDLE, message: LPCWSTR),

    /// infoレベルのログを出力します
    /// handle : ログ出力ハンドル
    /// message : ログメッセージ
    pub info: unsafe extern "C" fn(handle: *mut LOG_HANDLE, message: LPCWSTR),

    /// warnレベルのログを出力します
    /// handle : ログ出力ハンドル
    /// message : ログメッセージ
    pub warn: unsafe extern "C" fn(handle: *mut LOG_HANDLE, message: LPCWSTR),

    /// errorレベルのログを出力します
    /// handle : ログ出力ハンドル
    /// message : ログメッセージ
    pub error: unsafe extern "C" fn(handle: *mut LOG_HANDLE, message: LPCWSTR),

    /// verboseレベルのログを出力します
    /// handle : ログ出力ハンドル
    /// message : ログメッセージ
    pub verbose: unsafe extern "C" fn(handle: *mut LOG_HANDLE, message: LPCWSTR),
}
