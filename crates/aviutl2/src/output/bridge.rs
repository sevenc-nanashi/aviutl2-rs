use std::num::NonZeroIsize;

use crate::{
    common::{
        alert_error, format_file_filters, free_leaked_memory, leak_large_string,
        result_to_bool_with_dialog,
    },
    output::{FromRawAudioSamples, OutputInfo, OutputPlugin},
};

use aviutl2_sys::{
    input2::WAVE_FORMAT_PCM,
    output2::{LPCWSTR, WAVE_FORMAT_IEEE_FLOAT},
};

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

/// 出力プラグインを登録するマクロ。
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
