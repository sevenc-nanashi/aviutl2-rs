use std::num::NonZeroIsize;

use crate::{
    common::{LeakManager, alert_error, format_file_filters},
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

pub fn initialize_plugin<T: OutputSingleton>(version: u32) -> bool {
    let plugin_state = T::get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = match T::new(info) {
        Ok(plugin) => plugin,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            return false;
        }
    };
    let plugin = InternalOutputPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    true
}

pub fn uninitialize_plugin<T: OutputSingleton>() {
    let plugin_state = T::get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    *plugin_state = None;
}

pub fn create_table<T: OutputSingleton>() -> *mut aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    let plugin_state = T::get_singleton_state();
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
        func_output: Some(func_output::<T>),
        func_config: plugin_info.can_config.then_some(func_config::<T>),
        func_get_config_text: Some(func_get_config_text::<T>),
    };
    let table = Box::new(table);
    Box::leak(table)
}

extern "C" fn func_output<T: OutputSingleton>(oip: *mut aviutl2_sys::output2::OUTPUT_INFO) -> bool {
    let plugin_state = T::get_singleton_state();
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

extern "C" fn func_config<T: OutputSingleton>(
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    let plugin_state = T::get_singleton_state();
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

extern "C" fn func_get_config_text<T: OutputSingleton>() -> *const u16 {
    let plugin_state = T::get_singleton_state();
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

pub trait OutputSingleton
where
    Self: 'static + Send + Sync + OutputPlugin,
{
    fn get_singleton_state() -> &'static std::sync::RwLock<Option<InternalOutputPluginState<Self>>>;
}

/// 出力プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_output_plugin {
    ($struct:ident) => {
        ::aviutl2::internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                $crate::output::__bridge::initialize_plugin::<$struct>(version)
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                $crate::output::__bridge::uninitialize_plugin::<$struct>()
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetOutputPluginTable()
            -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE {
                $crate::output::__bridge::create_table::<$struct>()
            }
        }
    };
}
