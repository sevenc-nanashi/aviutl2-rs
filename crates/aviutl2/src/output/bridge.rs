use std::num::NonZeroIsize;

use crate::{
    common::{AnyResult, LeakManager, alert_error, format_file_filters},
    output::{FromRawAudioSamples, OutputInfo, OutputPlugin},
};

use aviutl2_sys::common::{WAVE_FORMAT_IEEE_FLOAT, WAVE_FORMAT_PCM};

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

pub unsafe fn initialize_plugin_c<T: OutputSingleton>(version: u32) -> bool {
    match initialize_plugin::<T>(version) {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub unsafe fn initialize_plugin_c_unwind<T: OutputSingleton>(version: u32) -> bool {
    match crate::utils::catch_unwind_with_panic_info(|| unsafe {
        initialize_plugin_c::<T>(version)
    }) {
        Ok(result) => result,
        Err(panic_info) => {
            log::error!(
                "Panic occurred during plugin initialization: {}",
                panic_info
            );
            alert_error(&panic_info);
            false
        }
    }
}

pub(crate) fn initialize_plugin<T: OutputSingleton>(version: u32) -> AnyResult<()> {
    crate::common::ensure_minimum_aviutl2_version(version.into())?;
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = T::new(info)?;
    let plugin = InternalOutputPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    Ok(())
}

pub unsafe fn uninitialize_plugin<T: OutputSingleton>() {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    *plugin_state = None;
}

pub unsafe fn uninitialize_plugin_c_unwind<T: OutputSingleton>() {
    match crate::utils::catch_unwind_with_panic_info(|| unsafe { uninitialize_plugin::<T>() }) {
        Ok(()) => {}
        Err(panic_info) => {
            log::error!(
                "Panic occurred during plugin uninitialization: {}",
                panic_info
            );
            alert_error(&panic_info);
        }
    }
}

fn create_table_impl<T: OutputSingleton>(
    unwind: bool,
) -> *mut aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    let plugin_state = plugin_state.as_mut().expect("Plugin not initialized");
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

    let func_output = if unwind {
        func_output_unwind::<T>
    } else {
        func_output::<T>
    };
    let func_config = if unwind {
        func_config_unwind::<T>
    } else {
        func_config::<T>
    };
    let func_get_config_text = if unwind {
        func_get_config_text_unwind::<T>
    } else {
        func_get_config_text::<T>
    };

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    let table = aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
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
    };
    let table = Box::new(table);
    Box::leak(table)
}

pub unsafe fn create_table<T: OutputSingleton>() -> *mut aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    create_table_impl::<T>(false)
}

pub unsafe fn create_table_unwind<T: OutputSingleton>()
-> *mut aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    match crate::utils::catch_unwind_with_panic_info(|| create_table_impl::<T>(true)) {
        Ok(table) => table,
        Err(panic_info) => {
            log::error!("Panic occurred during create_table: {}", panic_info);
            alert_error(&panic_info);
            std::ptr::null_mut()
        }
    }
}

extern "C" fn func_output<T: OutputSingleton>(oip: *mut aviutl2_sys::output2::OUTPUT_INFO) -> bool {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
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
extern "C" fn func_output_unwind<T: OutputSingleton>(
    oip: *mut aviutl2_sys::output2::OUTPUT_INFO,
) -> bool {
    match crate::utils::catch_unwind_with_panic_info(|| func_output::<T>(oip)) {
        Ok(result) => result,
        Err(panic_info) => {
            log::error!("Panic occurred during func_output: {}", panic_info);
            alert_error(&panic_info);
            false
        }
    }
}

extern "C" fn func_config<T: OutputSingleton>(
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
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
extern "C" fn func_config_unwind<T: OutputSingleton>(
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    match crate::utils::catch_unwind_with_panic_info(|| func_config::<T>(hwnd, dll_hinst)) {
        Ok(result) => result,
        Err(panic_info) => {
            log::error!("Panic occurred during func_config: {}", panic_info);
            alert_error(&panic_info);
            false
        }
    }
}

extern "C" fn func_get_config_text<T: OutputSingleton>() -> *const u16 {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
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
extern "C" fn func_get_config_text_unwind<T: OutputSingleton>() -> *const u16 {
    match crate::utils::catch_unwind_with_panic_info(|| func_get_config_text::<T>()) {
        Ok(text) => text,
        Err(panic_info) => {
            log::error!("Panic occurred during func_get_config_text: {}", panic_info);
            alert_error(&panic_info);
            std::ptr::null()
        }
    }
}

pub trait OutputSingleton
where
    Self: 'static + Send + Sync + OutputPlugin,
{
    fn __get_singleton_state() -> &'static std::sync::RwLock<Option<InternalOutputPluginState<Self>>>;
    fn with_instance<R>(f: impl FnOnce(&Self) -> R) -> R {
        let lock = Self::__get_singleton_state();
        let guard = lock.read().unwrap();
        let state = guard.as_ref().expect("Plugin not initialized");
        f(&state.instance)
    }
    fn with_instance_mut<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        let lock = Self::__get_singleton_state();
        let mut guard = lock.write().unwrap();
        let state = guard.as_mut().expect("Plugin not initialized");
        f(&mut state.instance)
    }
}

/// 出力プラグインを登録するマクロ。
///
/// # Arguments
///
/// - `unwind`: panic時にunwindするかどうか。デフォルトは`true`。
#[macro_export]
macro_rules! register_output_plugin {
    ($struct:ident, $($key:ident = $value:expr),* $(,)?) => {
        ::aviutl2::__internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializeLogger(logger: *mut $crate::sys::logger2::LOG_HANDLE) {
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
                        $crate::logger::__initialize_logger_unwind(logger)
                    } else {
                        $crate::logger::__initialize_logger(logger)
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe {
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
                            $crate::output::__bridge::initialize_plugin_c_unwind::<$struct>(version)
                        } else {
                            $crate::output::__bridge::initialize_plugin_c::<$struct>(version)
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe {
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
                            $crate::output::__bridge::uninitialize_plugin_c_unwind::<$struct>()
                        } else {
                            $crate::output::__bridge::uninitialize_plugin::<$struct>()
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetOutputPluginTable()
            -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE {
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
                        unsafe { $crate::output::__bridge::create_table_unwind::<$struct>() }
                    } else {
                        unsafe { $crate::output::__bridge::create_table::<$struct>() }
                    }
                }
            }
        }
    };
    ($struct:ident, $($key:ident),* $(,)?) => {
        $crate::register_output_plugin!($struct, $( $key = true ),* );
    };
    ($struct:ident) => {
        $crate::register_output_plugin!($struct, );
    };
}
