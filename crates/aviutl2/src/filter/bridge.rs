use std::num::NonZeroIsize;

use crate::{
    common::{
        AnyResult, LeakManager, alert_error, format_file_filters, leak_and_forget_as_wide_string,
        load_wide_string,
    },
    filter::{FilterPlugin, FilterPluginTable},
};

#[doc(hidden)]
pub struct InternalFilterPluginState<T: Send + Sync + FilterPlugin> {
    plugin_info: FilterPluginTable,
    leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + FilterPlugin> InternalFilterPluginState<T> {
    pub fn new(instance: T) -> Self {
        let plugin_info = instance.plugin_info();
        Self {
            plugin_info,
            leak_manager: LeakManager::new(),
            instance,
        }
    }
}

pub unsafe fn create_table<T: FilterPlugin>(
    plugin_state: &InternalFilterPluginState<T>,
    func_proc_video: extern "C" fn(video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO) -> bool,
    func_proc_audio: extern "C" fn(audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO) -> bool,
) -> aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
    let plugin_info = &plugin_state.plugin_info;

    let name = if cfg!(debug_assertions) {
        format!("{} (Debug)", plugin_info.name)
    } else {
        plugin_info.name.clone()
    };
    let information = if cfg!(debug_assertions) {
        format!("(Debug Build) {}", plugin_info.information)
    } else {
        plugin_info.information.clone()
    };

    let flag = plugin_info.input_type.to_bits()
        | (if plugin_info.wants_initial_input {
            aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_INPUT
        } else {
            0
        });

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
        flag,
        name: leak_and_forget_as_wide_string(&name),
        information: leak_and_forget_as_wide_string(&information),
        label: plugin_info
            .label
            .as_ref()
            .map_or(std::ptr::null(), |s| leak_and_forget_as_wide_string(s)),
        items: std::ptr::null(),
        func_proc_video: (((flag & aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_VIDEO) != 0)
            .then_some(func_proc_video)),
        func_proc_audio: ((flag & aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_AUDIO != 0)
            .then_some(func_proc_audio)),
    }
}
pub unsafe fn func_proc_video<T: FilterPlugin>(
    plugin_state: &InternalFilterPluginState<T>,
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    // TODO
    false
}
pub unsafe fn func_proc_audio<T: FilterPlugin>(
    plugin_state: &InternalFilterPluginState<T>,
    audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    // TODO
    false
}

/// フィルタプラグインを登録するマクロ。
#[macro_export]
macro_rules! register_filter_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_input_plugin {
            use super::$struct;
            use $crate::input::FilterPlugin as _;

            static PLUGIN: std::sync::LazyLock<
                aviutl2::input::__bridge::InternalFilterPluginState<$struct>,
            > = std::sync::LazyLock::new(|| {
                aviutl2::input::__bridge::InternalFilterPluginState::new($struct::new())
            });

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetFilterPluginTable()
            -> *mut aviutl2::sys::input2::FILTER_PLUGIN_TABLE {
                let table = unsafe {
                    $crate::filter::__bridge::create_table::<$struct>(
                        &PLUGIN,
                        func_open,
                        func_close,
                        func_info_get,
                        func_read_video,
                        func_read_audio,
                        func_config,
                        func_set_track,
                        func_time_to_frame,
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_open(
                file: aviutl2::sys::common::LPCWSTR,
            ) -> aviutl2::sys::input2::INPUT_HANDLE {
                unsafe { $crate::input::__bridge::func_open(&*PLUGIN, file) }
            }

            extern "C" fn func_close(ih: aviutl2::sys::input2::INPUT_HANDLE) -> bool {
                unsafe { $crate::input::__bridge::func_close(&*PLUGIN, ih) }
            }

            extern "C" fn func_info_get(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                iip: *mut aviutl2::sys::input2::INPUT_INFO,
            ) -> bool {
                unsafe { $crate::input::__bridge::func_info_get(&*PLUGIN, ih, iip) }
            }

            extern "C" fn func_read_video(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                frame: i32,
                buf: *mut std::ffi::c_void,
            ) -> i32 {
                unsafe { $crate::input::__bridge::func_read_video(&*PLUGIN, ih, frame, buf) }
            }

            extern "C" fn func_read_audio(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                start: i32,
                length: i32,
                buf: *mut std::ffi::c_void,
            ) -> i32 {
                unsafe {
                    $crate::input::__bridge::func_read_audio(&*PLUGIN, ih, start, length, buf)
                }
            }

            extern "C" fn func_config(
                hwnd: aviutl2::sys::input2::HWND,
                dll_hinst: aviutl2::sys::input2::HINSTANCE,
            ) -> bool {
                unsafe { $crate::input::__bridge::func_config(&*PLUGIN, hwnd, dll_hinst) }
            }

            extern "C" fn func_set_track(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                track_type: i32,
                track: i32,
            ) -> i32 {
                unsafe { $crate::input::__bridge::func_set_track(&*PLUGIN, ih, track_type, track) }
            }

            extern "C" fn func_time_to_frame(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                time: f64,
            ) -> i32 {
                unsafe { $crate::input::__bridge::func_time_to_frame(&*PLUGIN, ih, time) }
            }
        }
    };
}
