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

pub unsafe fn initialize_plugin<T: ScriptModule>(
    plugin_state: &std::sync::Arc<std::sync::RwLock<Option<InternalScriptModuleState<T>>>>,
    version: u32,
) -> bool {
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
    T::__internal_setup_plugin_handle(std::sync::Arc::clone(plugin_state));
    let plugin = InternalScriptModuleState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    true
}

pub unsafe fn create_table<T: ScriptModule>(
    plugin_state: &InternalScriptModuleState<T>,
) -> aviutl2_sys::module2::SCRIPT_MODULE_TABLE {
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
    aviutl2_sys::module2::SCRIPT_MODULE_TABLE {
        information: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&information),
        functions: functions_ptr,
    }
}

extern "C" fn unreachable_function(_: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM) {
    unreachable!("This function should never be called");
}

/// スクリプトモジュールを登録するマクロ。
#[macro_export]
macro_rules! register_script_module {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_script_module {
            use super::$struct;
            use $crate::module::ScriptModule as _;

            static PLUGIN: ::std::sync::LazyLock<
                ::std::sync::Arc<
                    ::std::sync::RwLock<
                        Option<$crate::module::__bridge::InternalScriptModuleState<$struct>>,
                    >,
                >,
            > = ::std::sync::LazyLock::new(|| {
                ::std::sync::Arc::new(::std::sync::RwLock::new(None))
            });

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe { $crate::module::__bridge::initialize_plugin(&PLUGIN, version) }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                *PLUGIN.write().unwrap() = None;
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetScriptModuleTable()
            -> *mut aviutl2::sys::module2::SCRIPT_MODULE_TABLE {
                let table = unsafe {
                    $crate::module::__bridge::create_table::<$struct>(
                        &PLUGIN
                            .read()
                            .unwrap()
                            .as_ref()
                            .expect("Plugin not initialized"),
                    )
                };
                Box::into_raw(Box::new(table))
            }
        }
    };
}
