use crate::{
    common::LeakManager,
    generic::{EditSection, GenericPlugin, HostAppHandle, HostAppTable, PluginRegistry},
};
use std::num::NonZeroIsize;

#[doc(hidden)]
pub struct InternalGenericPluginState<T: Send + Sync + GenericPlugin> {
    version: u32,

    plugin_registry: PluginRegistry,

    kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    global_leak_manager: LeakManager,
    leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + GenericPlugin> InternalGenericPluginState<T> {
    pub fn new(instance: T, version: u32) -> Self {
        Self {
            version,
            plugin_registry: PluginRegistry::new(),
            kill_switch: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            global_leak_manager: LeakManager::new(),
            leak_manager: LeakManager::new(),
            instance,
        }
    }
}

pub trait GenericSingleton
where
    Self: 'static + Send + Sync + GenericPlugin,
{
    fn __get_singleton_state()
    -> &'static std::sync::RwLock<Option<InternalGenericPluginState<Self>>>;
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

pub unsafe fn initialize_plugin<T: GenericSingleton>(version: u32) -> bool {
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = match T::new(info) {
        Ok(plugin) => plugin,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            crate::common::alert_error(&e);
            return false;
        }
    };
    let plugin = InternalGenericPluginState::new(internal, version);
    *plugin_state.write().unwrap() = Some(plugin);

    true
}
pub unsafe fn register_plugin<T: GenericSingleton>(
    host: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
) {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    let plugin_state = plugin_state.as_mut().expect("Plugin not initialized");

    let kill_switch = plugin_state.kill_switch.clone();
    let mut handle = unsafe {
        HostAppHandle::new(
            plugin_state.version,
            host,
            &mut plugin_state.global_leak_manager,
            kill_switch,
            &mut plugin_state.plugin_registry,
        )
    };
    T::register(&plugin_state.instance, &mut handle);
}
pub unsafe fn uninitialize_plugin<T: GenericSingleton>() {
    let plugin_state = T::__get_singleton_state();
    *plugin_state.write().unwrap() = None;
}

/// 汎用プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_generic_plugin {
    ($struct:ident) => {
        ::aviutl2::__internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe { $crate::generic::__bridge::initialize_plugin::<$struct>(version) }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe { $crate::generic::__bridge::uninitialize_plugin::<$struct>() };
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn RegisterPlugin(host: *mut aviutl2::sys::plugin2::HOST_APP_TABLE) {
                unsafe { $crate::generic::__bridge::register_plugin::<$struct>(host) };
            }
        }
    };
}
