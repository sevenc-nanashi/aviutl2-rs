#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_double, c_int};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SCRIPT_MODULE_PARAM {
    pub get_param_num: unsafe extern "C" fn() -> c_int,
    pub get_param_int: unsafe extern "C" fn(index: c_int) -> c_int,
    pub get_param_double: unsafe extern "C" fn(index: c_int) -> c_double,
    pub get_param_string: unsafe extern "C" fn(index: c_int) -> *const c_char,
    pub get_param_data: unsafe extern "C" fn(index: c_int) -> *mut c_void,
    pub get_param_table_int: unsafe extern "C" fn(index: c_int, key: *const c_char) -> c_int,
    pub get_param_table_double:
        unsafe extern "C" fn(index: c_int, key: *const c_char) -> c_double,
    pub get_param_table_string:
        unsafe extern "C" fn(index: c_int, key: *const c_char) -> *const c_char,
    pub get_param_array_num: unsafe extern "C" fn(index: c_int) -> c_int,
    pub get_param_array_int: unsafe extern "C" fn(index: c_int, key: c_int) -> c_int,
    pub get_param_array_double: unsafe extern "C" fn(index: c_int, key: c_int) -> c_double,
    pub get_param_array_string: unsafe extern "C" fn(index: c_int, key: c_int) -> *const c_char,
    pub push_result_int: unsafe extern "C" fn(value: c_int),
    pub push_result_double: unsafe extern "C" fn(value: c_double),
    pub push_result_string: unsafe extern "C" fn(value: *const c_char),
    pub push_result_data: unsafe extern "C" fn(value: *mut c_void),
    pub push_result_table_int:
        unsafe extern "C" fn(key: *mut *const c_char, value: *mut c_int, num: c_int),
    pub push_result_table_double:
        unsafe extern "C" fn(key: *mut *const c_char, value: *mut c_double, num: c_int),
    pub push_result_table_string:
        unsafe extern "C" fn(key: *mut *const c_char, value: *mut *const c_char, num: c_int),
    pub push_result_array_int: unsafe extern "C" fn(value: *mut c_int, num: c_int),
    pub push_result_array_double: unsafe extern "C" fn(value: *mut c_double, num: c_int),
    pub push_result_array_string: unsafe extern "C" fn(value: *mut *const c_char, num: c_int),
    pub set_error: unsafe extern "C" fn(message: *const c_char),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SCRIPT_MODULE_FUNCTION {
    pub name: *const u16,
    pub func: unsafe extern "C" fn(smp: *mut SCRIPT_MODULE_PARAM),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SCRIPT_MODULE_TABLE {
    pub information: *const u16,
    pub functions: *mut SCRIPT_MODULE_FUNCTION,
}
