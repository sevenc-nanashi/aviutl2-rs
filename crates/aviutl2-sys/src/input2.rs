#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code
)]

use std::ffi::c_void;

pub use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::{BI_BITFIELDS, BI_RGB, BITMAPINFOHEADER},
    Media::{
        Audio::{WAVE_FORMAT_PCM, WAVEFORMATEX},
        Multimedia::WAVE_FORMAT_IEEE_FLOAT,
    },
};

macro_rules! fourcc {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24))
    };
}

pub const BI_YUY2: u32 = fourcc!('Y', 'U', 'Y', '2');
pub const BI_PA64: u32 = fourcc!('P', 'A', '6', '4');
pub const BI_YC48: u32 = fourcc!('Y', 'C', '4', '8');
pub const BI_HF64: u32 = fourcc!('H', 'F', '6', '4');

pub type LPCWSTR = *const u16;

#[repr(C)]
pub struct INPUT_INFO {
    pub flag: i32,
    pub rate: i32,
    pub scale: i32,
    pub n: i32,
    pub format: *mut BITMAPINFOHEADER,
    pub format_size: i32,
    pub audio_n: i32,
    pub audio_format: *mut WAVEFORMATEX,
    pub audio_format_size: i32,
}

impl INPUT_INFO {
    pub const FLAG_VIDEO: i32 = 1;
    pub const FLAG_AUDIO: i32 = 2;
    pub const FLAG_CONCURRENT: i32 = 16;
}

pub type INPUT_HANDLE = *mut c_void;

#[repr(C)]
pub struct INPUT_PLUGIN_TABLE {
    pub flag: i32,
    pub name: LPCWSTR,
    pub filefilter: LPCWSTR,
    pub information: LPCWSTR,
    pub func_open: Option<extern "C" fn(file: LPCWSTR) -> INPUT_HANDLE>,
    pub func_close: Option<extern "C" fn(ih: INPUT_HANDLE) -> bool>,
    pub func_info_get: Option<extern "C" fn(ih: INPUT_HANDLE, iip: *mut INPUT_INFO) -> bool>,
    pub func_read_video:
        Option<extern "C" fn(ih: INPUT_HANDLE, frame: i32, buf: *mut c_void) -> i32>,
    pub func_read_audio:
        Option<extern "C" fn(ih: INPUT_HANDLE, start: i32, length: i32, buf: *mut c_void) -> i32>,
    pub func_config: Option<extern "C" fn(hwnd: HWND, dll_hinst: HINSTANCE) -> bool>,
}

impl INPUT_PLUGIN_TABLE {
    pub const FLAG_VIDEO: i32 = 1;
    pub const FLAG_AUDIO: i32 = 2;
}
