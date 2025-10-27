use crate::{
    common::{LeakManager, alert_error},
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
}

pub unsafe fn initialize_plugin<T: ScriptModuleSingleton>(version: u32) -> bool {
    let plugin_state = T::__get_singleton_state();
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
    let plugin = InternalScriptModuleState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    true
}
pub unsafe fn uninitialize_plugin<T: ScriptModuleSingleton>() {
    let plugin_state = T::__get_singleton_state();
    *plugin_state.write().unwrap() = None;
}

pub unsafe fn create_table<T: ScriptModuleSingleton>() -> *mut aviutl2_sys::module2::SCRIPT_MODULE_TABLE
{
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

extern "C" fn unreachable_function(_: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM) {
    unreachable!("This function should never be called");
}

/// スクリプトモジュールを登録するマクロ。
#[macro_export]
macro_rules! register_script_module {
    ($struct:ident) => {
        ::aviutl2::internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe { $crate::module::__bridge::initialize_plugin::<$struct>(version) }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe { $crate::module::__bridge::uninitialize_plugin::<$struct>() }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetScriptModuleTable()
            -> *mut aviutl2::sys::module2::SCRIPT_MODULE_TABLE {
                unsafe { $crate::module::__bridge::create_table::<$struct>() }
            }
        }
    };
}
