use crate::{
    common::{AnyResult, LeakManager, alert_error},
    module::{ScriptModule, ScriptModuleTable},
};

#[doc(hidden)]
pub struct InternalScriptModuleState<T: Send + Sync + ScriptModule> {
    plugin_info: ScriptModuleTable,
    global_leak_manager: LeakManager,

    pub instance: T,
}

impl<T: Send + Sync + ScriptModule> InternalScriptModuleState<T> {
    pub fn new(instance: T) -> Self {
        let plugin_info = instance.plugin_info();
        Self {
            plugin_info,
            global_leak_manager: LeakManager::new(),
            instance,
        }
    }
}

pub trait ScriptModuleSingleton
where
    Self: ScriptModule + Sized + Send + Sync + 'static,
{
    fn __get_singleton_state() -> &'static std::sync::RwLock<Option<InternalScriptModuleState<Self>>>;

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

pub unsafe fn initialize_plugin_c<T: ScriptModuleSingleton>(version: u32) -> bool {
    match initialize_plugin::<T>(version) {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub unsafe fn initialize_plugin_c_unwind<T: ScriptModuleSingleton>(version: u32) -> bool {
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

pub(crate) fn initialize_plugin<T: ScriptModuleSingleton>(version: u32) -> AnyResult<()> {
    crate::common::ensure_minimum_aviutl2_version(version.into())?;
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = T::new(info)?;
    let plugin = InternalScriptModuleState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    Ok(())
}
pub unsafe fn uninitialize_plugin<T: ScriptModuleSingleton>() {
    let plugin_state = T::__get_singleton_state();
    *plugin_state.write().unwrap() = None;
}

pub unsafe fn uninitialize_plugin_c_unwind<T: ScriptModuleSingleton>() {
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

pub unsafe fn create_table<T: ScriptModuleSingleton>()
-> *mut aviutl2_sys::module2::SCRIPT_MODULE_TABLE {
    let plugin_state_lock = T::__get_singleton_state();
    let plugin_state = plugin_state_lock.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    let plugin_info = &plugin_state.plugin_info;
    let information = if cfg!(debug_assertions) {
        format!("(Debug Build) {}", plugin_info.information)
    } else {
        plugin_info.information.clone()
    };

    let module_functions: Vec<aviutl2_sys::module2::SCRIPT_MODULE_FUNCTION> = plugin_info
        .functions
        .iter()
        .map(|f| aviutl2_sys::module2::SCRIPT_MODULE_FUNCTION {
            name: plugin_state
                .global_leak_manager
                .leak_as_wide_string(&f.name),
            func: f.func,
        })
        .chain(std::iter::once(
            aviutl2_sys::module2::SCRIPT_MODULE_FUNCTION {
                name: std::ptr::null(),
                func: unreachable_function,
            },
        ))
        .collect();
    let functions_ptr = plugin_state
        .global_leak_manager
        .leak_value_vec(module_functions);

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    let table = aviutl2_sys::module2::SCRIPT_MODULE_TABLE {
        information: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&information),
        functions: functions_ptr,
    };
    let table = Box::new(table);
    Box::leak(table)
}

pub unsafe fn create_table_unwind<T: ScriptModuleSingleton>()
-> *mut aviutl2_sys::module2::SCRIPT_MODULE_TABLE {
    match crate::utils::catch_unwind_with_panic_info(|| unsafe { create_table::<T>() }) {
        Ok(table) => table,
        Err(panic_info) => {
            log::error!("Panic occurred during create_table: {}", panic_info);
            alert_error(&panic_info);
            std::ptr::null_mut()
        }
    }
}

extern "C" fn unreachable_function(_: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM) {
    unreachable!("This function should never be called");
}

/// スクリプトモジュールを登録するマクロ。
///
/// # Arguments
///
/// - `unwind`: panic時にunwindするかどうか。デフォルトは`true`。
#[macro_export]
macro_rules! register_script_module {
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
                            $crate::module::__bridge::initialize_plugin_c_unwind::<$struct>(version)
                        } else {
                            $crate::module::__bridge::initialize_plugin_c::<$struct>(version)
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe {
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
                            $crate::module::__bridge::uninitialize_plugin_c_unwind::<$struct>()
                        } else {
                            $crate::module::__bridge::uninitialize_plugin::<$struct>()
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetScriptModuleTable()
            -> *mut aviutl2::sys::module2::SCRIPT_MODULE_TABLE {
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
                        unsafe { $crate::module::__bridge::create_table_unwind::<$struct>() }
                    } else {
                        unsafe { $crate::module::__bridge::create_table::<$struct>() }
                    }
                }
            }
        }
    };
    ($struct:ident, $($key:ident),* $(,)?) => {
        $crate::register_script_module!($struct, $( $key = true ),* );
    };
    ($struct:ident) => {
        $crate::register_script_module!($struct, );
    };
}
