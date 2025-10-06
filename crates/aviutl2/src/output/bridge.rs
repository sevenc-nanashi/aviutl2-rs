use std::num::NonZeroIsize;

use crate::{
    common::{LeakManager, alert_error, format_file_filters},
    output::{FromRawAudioSamples, OutputInfo, OutputPlugin},
};

use aviutl2_sys::common::{LPCWSTR, WAVE_FORMAT_IEEE_FLOAT, WAVE_FORMAT_PCM};

pub struct InternalOutputPluginState<T: Send + Sync + OutputPlugin> {
    leak_manager: LeakManager,
    global_leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + OutputPlugin> InternalOutputPluginState<T> {
    pub fn new(instance: T) -> Self {
        Self {
            leak_manager: LeakManager::new(),
            global_leak_manager: LeakManager::new(),
            instance,
        }
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
    plugin_state: &InternalOutputPluginState<T>,
    func_output: extern "C" fn(*mut aviutl2_sys::output2::OUTPUT_INFO) -> bool,
    func_config: extern "C" fn(aviutl2_sys::output2::HWND, aviutl2_sys::output2::HINSTANCE) -> bool,
    func_get_config_text: extern "C" fn() -> LPCWSTR,
) -> aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    log::info!("Creating OUTPUT_PLUGIN_TABLE");
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let plugin_info = plugin.plugin_info();
    let filefilter = format_file_filters(&plugin_info.file_filters);

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

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
        flag: plugin_info.output_type.to_bits(),
        name: plugin_state.global_leak_manager.leak_as_wide_string(&name),
        filefilter: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&filefilter),
        information: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&information),
        func_output: Some(func_output),
        func_config: plugin_info.can_config.then_some(func_config),
        func_get_config_text: Some(func_get_config_text),
    }
}

pub unsafe fn func_output<T: OutputPlugin>(
    plugin_state: &InternalOutputPluginState<T>,
    oip: *mut aviutl2_sys::output2::OUTPUT_INFO,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let oip = unsafe { &mut *oip };
    let output_info = OutputInfo::from_raw(oip);
    match plugin.output(output_info) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error during func_output: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub unsafe fn func_config<T: OutputPlugin>(
    plugin_state: &InternalOutputPluginState<T>,
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let mut handle =
        raw_window_handle::Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
    handle.hinstance = Some(NonZeroIsize::new(dll_hinst as isize).unwrap());
    match plugin.config(handle) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error during func_config: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub unsafe fn func_get_config_text<T: OutputPlugin>(
    plugin_state: &InternalOutputPluginState<T>,
) -> *const u16 {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let text = plugin.config_text();
    match text {
        Ok(text) => plugin_state.leak_manager.leak_as_wide_string(&text),
        Err(e) => {
            log::error!("Error during func_get_config_text: {}", e);
            plugin_state
                .leak_manager
                .leak_as_wide_string(format!("エラー：{}", e).as_str())
        }
    }
}

/// 出力プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_output_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_output_plugin {
            use super::$struct;
            use $crate::output::OutputPlugin as _;

            static PLUGIN: std::sync::LazyLock<
                std::sync::RwLock<
                    Option<$crate::output::__bridge::InternalOutputPluginState<$struct>>,
                >,
            > = std::sync::LazyLock::new(|| std::sync::RwLock::new(None));

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                let info = $crate::common::AviUtl2Info { version };
                let internal = match $struct::new(info) {
                    Ok(plugin) => plugin,
                    Err(e) => {
                        $crate::log::error!("Failed to initialize plugin: {}", e);
                        return false;
                    }
                };
                let plugin = $crate::output::__bridge::InternalOutputPluginState::new(internal);
                *PLUGIN.write().unwrap() = Some(plugin);

                true
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                *PLUGIN.write().unwrap() = None;
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetOutputPluginTable()
            -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE {
                let table = unsafe {
                    $crate::output::__bridge::create_table::<$struct>(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                        func_output,
                        func_config,
                        func_get_config_text,
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_output(oip: *mut aviutl2::sys::output2::OUTPUT_INFO) -> bool {
                unsafe {
                    $crate::output::__bridge::func_output(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                        oip,
                    )
                }
            }

            extern "C" fn func_config(
                hwnd: aviutl2::sys::output2::HWND,
                dll_hinst: aviutl2::sys::output2::HINSTANCE,
            ) -> bool {
                unsafe {
                    $crate::output::__bridge::func_config(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                        hwnd,
                        dll_hinst,
                    )
                }
            }

            extern "C" fn func_get_config_text() -> *const u16 {
                unsafe {
                    $crate::output::__bridge::func_get_config_text(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                    )
                }
            }
        }
    };
}
