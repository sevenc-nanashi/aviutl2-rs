#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code
)]

use std::ffi::c_void;

pub use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    Graphics::Gdi::{BI_RGB, BITMAPINFOHEADER},
    Media::{Audio::WAVE_FORMAT_PCM, Multimedia::WAVE_FORMAT_IEEE_FLOAT},
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
pub struct OUTPUT_INFO {
    pub flag: i32,
    pub w: i32,
    pub h: i32,
    pub rate: i32,
    pub scale: i32,
    pub n: i32,
    pub audio_rate: i32,
    pub audio_ch: i32,
    pub audio_n: i32,
    pub savefile: LPCWSTR,
    pub func_get_video: Option<extern "C" fn(frame: i32, format: u32) -> *mut c_void>,
    pub func_get_audio: Option<
        extern "C" fn(start: i32, length: i32, readed: *mut i32, format: u32) -> *mut c_void,
    >,
    pub func_is_abort: Option<extern "C" fn() -> bool>,
    pub func_rest_time_disp: Option<extern "C" fn(now: i32, total: i32)>,
    pub func_set_buffer_size: Option<extern "C" fn(video_size: i32, audio_size: i32)>,
}

impl OUTPUT_INFO {
    pub const FLAG_VIDEO: i32 = 1;
    pub const FLAG_AUDIO: i32 = 2;
}

#[repr(C)]
pub struct OUTPUT_PLUGIN_TABLE {
    pub flag: i32,
    pub name: LPCWSTR,
    pub filefilter: LPCWSTR,
    pub information: LPCWSTR,
    pub func_output: Option<extern "C" fn(oip: *mut OUTPUT_INFO) -> bool>,
    pub func_config: Option<extern "C" fn(hwnd: HWND, dll_hinst: HINSTANCE) -> bool>,
    pub func_get_config_text: Option<extern "C" fn() -> LPCWSTR>,
}

impl OUTPUT_PLUGIN_TABLE {
    pub const FLAG_VIDEO: i32 = 1;
    pub const FLAG_AUDIO: i32 = 2;
}
