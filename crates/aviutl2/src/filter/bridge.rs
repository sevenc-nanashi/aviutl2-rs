use std::num::NonZeroIsize;

use crate::{
    common::{AnyResult, LeakManager, alert_error, format_file_filters, load_wide_string},
    filter::{FilterPlugin, FilterPluginTable},
};

#[doc(hidden)]
pub struct InternalFilterPluginState<T: Send + Sync + FilterPlugin> {
    plugin_info: FilterPluginTable,
    global_leak_manager: LeakManager,
    leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + FilterPlugin> InternalFilterPluginState<T> {
    pub fn new(instance: T) -> Self {
        let plugin_info = instance.plugin_info();
        Self {
            plugin_info,
            global_leak_manager: LeakManager::new(),
            leak_manager: LeakManager::new(),
            instance,
        }
    }
}

pub unsafe fn initialize_plugin<T: FilterPlugin>(
    plugin_state: &std::sync::RwLock<Option<InternalFilterPluginState<T>>>,
    version: u32,
) -> bool {
    let info = crate::common::AviUtl2Info { version };
    let internal = match T::new(info) {
        Ok(plugin) => plugin,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            return false;
        }
    };
    let plugin = InternalFilterPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    true
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

    let flag = plugin_info.filter_type.to_bits()
        | (if plugin_info.wants_initial_input {
            aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_INPUT
        } else {
            0
        });

    crate::odbg!(&flag);

    let items = plugin_info
        .config_items
        .iter()
        .map(|item| {
            plugin_state
                .global_leak_manager
                .leak(item.to_raw(&plugin_state.global_leak_manager))
        })
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();
    let items = plugin_state
        .global_leak_manager
        .leak_value_vec(items.iter().map(|&p| p as usize).collect());

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
        flag,
        name: plugin_state.global_leak_manager.leak_as_wide_string(&name),
        information: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&information),
        label: plugin_info.label.as_ref().map_or(std::ptr::null(), |s| {
            plugin_state.global_leak_manager.leak_as_wide_string(s)
        }),
        items: items as _,
        func_proc_video: Some(func_proc_video),
        func_proc_audio: Some(func_proc_audio),
    }
}
pub unsafe fn func_proc_video<T: FilterPlugin>(
    plugin_state: &InternalFilterPluginState<T>,
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    // TODO
    true
}
pub unsafe fn func_proc_audio<T: FilterPlugin>(
    plugin_state: &InternalFilterPluginState<T>,
    audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    // TODO
    true
}

/// フィルタプラグインを登録するマクロ。
#[macro_export]
macro_rules! register_filter_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_filter_plugin {
            use super::$struct;
            use $crate::filter::FilterPlugin;

            static PLUGIN: std::sync::LazyLock<
                std::sync::RwLock<
                    Option<$crate::filter::__bridge::InternalFilterPluginState<$struct>>,
                >,
            > = std::sync::LazyLock::new(|| std::sync::RwLock::new(None));

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe { $crate::filter::__bridge::initialize_plugin(&PLUGIN, version) }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                *PLUGIN.write().unwrap() = None;
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetFilterPluginTable()
            -> *mut aviutl2::sys::filter2::FILTER_PLUGIN_TABLE {
                let table = unsafe {
                    $crate::filter::__bridge::create_table::<$struct>(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                        func_proc_video,
                        func_proc_audio,
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_proc_audio(
                video: *mut aviutl2::sys::filter2::FILTER_PROC_AUDIO,
            ) -> bool {
                unsafe {
                    $crate::filter::__bridge::func_proc_audio(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                        video,
                    )
                }
            }

            extern "C" fn func_proc_video(
                video: *mut aviutl2::sys::filter2::FILTER_PROC_VIDEO,
            ) -> bool {
                unsafe {
                    $crate::filter::__bridge::func_proc_video(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                        video,
                    )
                }
            }
        }
    };
}
