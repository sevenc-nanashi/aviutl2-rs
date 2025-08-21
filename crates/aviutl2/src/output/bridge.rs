use std::num::NonZeroIsize;

use crate::{
    common::{
        alert_error, format_file_filters, free_leaked_memory, leak_large_string,
        result_to_bool_with_dialog,
    },
    output::{
        FromRawAudioSamples, FromRawVideoFrame, Hf64VideoFrame, OutputInfo, OutputPlugin,
        Pa64VideoFrame, RawBgrVideoFrame, RawHf64VideoFrame, RawPa64VideoFrame, RawYc48VideoFrame,
        RawYuy2VideoFrame, RgbVideoFrame, VideoOutputInfo, Yc48VideoFrame, Yuy2VideoFrame,
    },
};

use aviutl2_sys::{
    input2::WAVE_FORMAT_PCM,
    output2::{BI_RGB, LPCWSTR, WAVE_FORMAT_IEEE_FLOAT},
};
use half::f16;

use super::Yc48;

impl FromRawVideoFrame for RgbVideoFrame {
    const FORMAT: u32 = BI_RGB;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 3 bytes (BGR)
                let pixel_r = unsafe { *frame_data_ptr.add(i * 3 + 2) };
                let pixel_g = unsafe { *frame_data_ptr.add(i * 3 + 1) };
                let pixel_b = unsafe { *frame_data_ptr.add(i * 3) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((pixel_r, pixel_g, pixel_b));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for Yuy2VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_YUY2;

    fn check(video: &VideoOutputInfo) -> Result<(), String> {
        if video.width % 2 != 0 || video.height % 2 != 0 {
            return Err("YUY2 format requires even width and height".to_string());
        }
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height / 2) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        for y in 0..video.height as usize {
            for x in 0..(video.width / 2) as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 4 bytes (YUY2)
                let d_y1 = unsafe { *frame_data_ptr.add(i * 4) };
                let d_u = unsafe { *frame_data_ptr.add(i * 4 + 1) };
                let d_y2 = unsafe { *frame_data_ptr.add(i * 4 + 2) };
                let d_v = unsafe { *frame_data_ptr.add(i * 4 + 3) };

                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((d_y1, d_u, d_y2, d_v));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height * 2) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for RawBgrVideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_RGB;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let frame_buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 3) as usize)
                .to_owned()
        };

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for RawYuy2VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_YUY2;

    fn check(video: &VideoOutputInfo) -> Result<(), String> {
        if video.width % 2 != 0 || video.height % 2 != 0 {
            return Err("YUY2 format requires even width and height".to_string());
        }
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let frame_buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 2) as usize)
                .to_owned()
        };

        Self { data: frame_buffer }
    }
}

impl FromRawVideoFrame for Hf64VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_HF64;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        let frame_data_ptr = frame_data_ptr as *const u16;
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 8 bytes (RGBA)
                let pixel_r = unsafe { *frame_data_ptr.add(i * 4) };
                let pixel_g = unsafe { *frame_data_ptr.add(i * 4 + 1) };
                let pixel_b = unsafe { *frame_data_ptr.add(i * 4 + 2) };
                let pixel_a = unsafe { *frame_data_ptr.add(i * 4 + 3) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((
                        f16::from_bits(pixel_r),
                        f16::from_bits(pixel_g),
                        f16::from_bits(pixel_b),
                        f16::from_bits(pixel_a),
                    ));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for Yc48VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_YC48;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        let frame_data_ptr = frame_data_ptr as *const i16;
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 6 bytes (YCbCr)
                let pixel_y = unsafe { *frame_data_ptr.add(i * 3) };
                let pixel_cr = unsafe { *frame_data_ptr.add(i * 3 + 1) };
                let pixel_cb = unsafe { *frame_data_ptr.add(i * 3 + 2) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write(Yc48 {
                        y: pixel_y,
                        cr: pixel_cr,
                        cb: pixel_cb,
                    });
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for Pa64VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_PA64;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        let frame_data_ptr = frame_data_ptr as *const u16;
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 8 bytes (RGBA)
                let pixel_r = unsafe { *frame_data_ptr.add(i * 4) };
                let pixel_g = unsafe { *frame_data_ptr.add(i * 4 + 1) };
                let pixel_b = unsafe { *frame_data_ptr.add(i * 4 + 2) };
                let pixel_a = unsafe { *frame_data_ptr.add(i * 4 + 3) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((pixel_r, pixel_g, pixel_b, pixel_a));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}

#[duplicate::duplicate_item(
    Name                Type  elms FMT;
    [RawHf64VideoFrame] [u16] [4]  [aviutl2_sys::output2::BI_HF64];
    [RawYc48VideoFrame] [i16] [3]  [aviutl2_sys::output2::BI_YC48];
    [RawPa64VideoFrame] [u16] [4]  [aviutl2_sys::output2::BI_PA64];
)]
impl FromRawVideoFrame for Name {
    const FORMAT: u32 = FMT;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let frame_buffer = unsafe {
            std::slice::from_raw_parts(
                frame_data_ptr as *const Type,
                (video.width * video.height * elms) as usize,
            )
            .to_owned()
        };

        Self { data: frame_buffer }
    }
}

impl FromRawAudioSamples for f32 {
    const FORMAT: u32 = WAVE_FORMAT_IEEE_FLOAT;

    unsafe fn from_raw(length: i32, num_channels: u32, frame_data_ptr: *const u8) -> Vec<Self> {
        let frame_data_slice = unsafe {
            std::slice::from_raw_parts(
                frame_data_ptr as *const f32,
                length as usize * num_channels as usize,
            )
        };
        frame_data_slice.to_vec()
    }
}
impl FromRawAudioSamples for i16 {
    const FORMAT: u32 = WAVE_FORMAT_PCM;

    unsafe fn from_raw(length: i32, num_channels: u32, frame_data_ptr: *const u8) -> Vec<Self> {
        let frame_data_slice = unsafe {
            std::slice::from_raw_parts(
                frame_data_ptr as *const i16,
                length as usize * num_channels as usize,
            )
        };
        frame_data_slice.to_vec()
    }
}

pub unsafe fn create_table<T: OutputPlugin>(
    plugin: &T,
    func_output: extern "C" fn(*mut aviutl2_sys::output2::OUTPUT_INFO) -> bool,
    func_config: extern "C" fn(aviutl2_sys::output2::HWND, aviutl2_sys::output2::HINSTANCE) -> bool,
    func_get_config_text: extern "C" fn() -> LPCWSTR,
) -> aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    free_leaked_memory();
    let plugin_info = plugin.plugin_info();
    let filefilter = format_file_filters(&plugin_info.file_filters);
    let filefilter = leak_large_string(&filefilter);

    let name = if cfg!(debug_assertions) {
        format!("{} (Debug)", plugin_info.name)
    } else {
        plugin_info.name
    };
    let information = if cfg!(debug_assertions) {
        format!("{} (Debug Build)", plugin_info.information)
    } else {
        plugin_info.information
    };

    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
        flag: plugin_info.output_type.to_bits(),
        name: leak_large_string(&name),
        filefilter,
        information: leak_large_string(&information),
        func_output: Some(func_output),
        func_config: plugin_info.can_config.then_some(func_config),
        func_get_config_text: Some(func_get_config_text),
    }
}

pub unsafe fn func_output<T: OutputPlugin>(
    plugin: &T,
    oip: *mut aviutl2_sys::output2::OUTPUT_INFO,
) -> bool {
    let oip = unsafe { &mut *oip };
    let output_info = OutputInfo::from_raw(oip);
    result_to_bool_with_dialog(plugin.output(output_info))
}

pub unsafe fn func_config<T: OutputPlugin>(
    plugin: &T,
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    let mut handle =
        raw_window_handle::Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
    handle.hinstance = Some(NonZeroIsize::new(dll_hinst as isize).unwrap());
    result_to_bool_with_dialog(plugin.config(handle))
}

pub unsafe fn func_get_config_text<T: OutputPlugin>(plugin: &T) -> *mut u16 {
    let text = plugin.config_text();
    match text {
        Ok(text) => leak_large_string(&text),
        Err(e) => {
            alert_error(&e);
            leak_large_string("Error")
        }
    }
}

#[macro_export]
macro_rules! register_output_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_output_plugin {
            use super::*;

            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetOutputPluginTable()
            -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE {
                let table = unsafe {
                    $crate::output::__bridge::create_table::<$struct>(
                        &*PLUGIN,
                        func_output,
                        func_config,
                        func_get_config_text,
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_output(oip: *mut aviutl2::sys::output2::OUTPUT_INFO) -> bool {
                unsafe { $crate::output::__bridge::func_output(&*PLUGIN, oip) }
            }

            extern "C" fn func_config(
                hwnd: aviutl2::sys::output2::HWND,
                dll_hinst: aviutl2::sys::output2::HINSTANCE,
            ) -> bool {
                unsafe { $crate::output::__bridge::func_config(&*PLUGIN, hwnd, dll_hinst) }
            }

            extern "C" fn func_get_config_text() -> *const u16 {
                unsafe { $crate::output::__bridge::func_get_config_text(&*PLUGIN) }
            }
        }
    };
}
