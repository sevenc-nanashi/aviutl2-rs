//! スクリプトモジュール ヘッダーファイル for AviUtl ExEdit2
//! By ＫＥＮくん

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_double, c_int};

/// スクリプトモジュール引数構造体
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SCRIPT_MODULE_PARAM {
    /// 引数の数を取得する
    ///
    /// # Returns
    ///
    /// 引数の数
    pub get_param_num: unsafe extern "C" fn() -> c_int,

    /// 引数を整数で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合は0)
    pub get_param_int: unsafe extern "C" fn(index: c_int) -> c_int,

    /// 引数を浮動小数点で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合は0)
    pub get_param_double: unsafe extern "C" fn(index: c_int) -> c_double,

    /// 引数を文字列(UTF-8)で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合はnullptr)
    pub get_param_string: unsafe extern "C" fn(index: c_int) -> *const c_char,

    /// 引数をデータのポインタで取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合はnullptr)
    pub get_param_data: unsafe extern "C" fn(index: c_int) -> *mut c_void,

    /// 引数の連想配列要素を整数で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    /// * `key` - キー名(UTF-8)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合は0)
    pub get_param_table_int: unsafe extern "C" fn(index: c_int, key: *const c_char) -> c_int,

    /// 引数の連想配列要素を浮動小数点で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    /// * `key` - キー名(UTF-8)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合は0)
    pub get_param_table_double:
        unsafe extern "C" fn(index: c_int, key: *const c_char) -> c_double,

    /// 引数の連想配列要素を文字列(UTF-8)で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    /// * `key` - キー名(UTF-8)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合はnullptr)
    pub get_param_table_string:
        unsafe extern "C" fn(index: c_int, key: *const c_char) -> *const c_char,

    /// 引数の配列要素の数を取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    ///
    /// # Returns
    ///
    /// 配列要素の数
    pub get_param_array_num: unsafe extern "C" fn(index: c_int) -> c_int,

    /// 引数の配列要素を整数で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    /// * `key` - 配列のインデックス(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合は0)
    pub get_param_array_int: unsafe extern "C" fn(index: c_int, key: c_int) -> c_int,

    /// 引数の配列要素を浮動小数点で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    /// * `key` - 配列のインデックス(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合は0)
    pub get_param_array_double: unsafe extern "C" fn(index: c_int, key: c_int) -> c_double,

    /// 引数の配列要素を文字列(UTF-8)で取得する
    ///
    /// # Arguments
    ///
    /// * `index` - 引数の位置(0〜)
    /// * `key` - 配列のインデックス(0〜)
    ///
    /// # Returns
    ///
    /// 引数の値 (取得出来ない場合はnullptr)
    pub get_param_array_string: unsafe extern "C" fn(index: c_int, key: c_int) -> *const c_char,

    /// 整数の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値
    pub push_result_int: unsafe extern "C" fn(value: c_int),

    /// 浮動小数点の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値
    pub push_result_double: unsafe extern "C" fn(value: c_double),

    /// 文字列(UTF-8)の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値
    pub push_result_string: unsafe extern "C" fn(value: *const c_char),

    /// データのポインタの戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値
    pub push_result_data: unsafe extern "C" fn(value: *mut c_void),

    /// 整数連想配列の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `key` - キー名(UTF-8)の配列
    /// * `value` - 戻り値の配列
    /// * `num` - 配列の要素数
    pub push_result_table_int:
        unsafe extern "C" fn(key: *mut *const c_char, value: *mut c_int, num: c_int),

    /// 浮動小数点連想配列の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `key` - キー名(UTF-8)の配列
    /// * `value` - 戻り値の配列
    /// * `num` - 配列の要素数
    pub push_result_table_double:
        unsafe extern "C" fn(key: *mut *const c_char, value: *mut c_double, num: c_int),

    /// 文字列(UTF-8)連想配列の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `key` - キー名(UTF-8)の配列
    /// * `value` - 戻り値の配列
    /// * `num` - 配列の要素数
    pub push_result_table_string:
        unsafe extern "C" fn(key: *mut *const c_char, value: *mut *const c_char, num: c_int),

    /// 整数配列の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値の配列
    /// * `num` - 配列の要素数
    pub push_result_array_int: unsafe extern "C" fn(value: *mut c_int, num: c_int),

    /// 浮動小数点配列の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値の配列
    /// * `num` - 配列の要素数
    pub push_result_array_double: unsafe extern "C" fn(value: *mut c_double, num: c_int),

    /// 文字列(UTF-8)配列の戻り値を追加する
    ///
    /// # Arguments
    ///
    /// * `value` - 戻り値の配列
    /// * `num` - 配列の要素数
    pub push_result_array_string: unsafe extern "C" fn(value: *mut *const c_char, num: c_int),

    /// エラーメッセージを設定する
    /// 呼び出された関数をエラー終了する場合に設定します
    ///
    /// # Arguments
    ///
    /// * `message` - エラーメッセージ(UTF-8)
    pub set_error: unsafe extern "C" fn(message: *const c_char),
}

/// スクリプトモジュール関数定義構造体
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SCRIPT_MODULE_FUNCTION {
    /// 関数名
    pub name: *const u16,
    /// 関数へのポインタ
    pub func: unsafe extern "C" fn(smp: *mut SCRIPT_MODULE_PARAM),
}

/// スクリプトモジュール構造体
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SCRIPT_MODULE_TABLE {
    /// スクリプトモジュールの情報
    pub information: *const u16,
    /// 登録する関数の一覧 (SCRIPT_MODULE_FUNCTIONを列挙して関数名がnullの要素で終端したリストへのポインタ)
    pub functions: *mut SCRIPT_MODULE_FUNCTION,
}
