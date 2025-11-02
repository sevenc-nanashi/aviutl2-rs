use crate::{
    common::LeakManager,
    generic::{GenericPlugin, HostAppTable},
};
use std::num::NonZeroIsize;

mod to_plugin_table {
    pub trait ToPluginTable<T> {
        fn initialize_plugin(version: u32) -> bool;
        fn to_plugin_table(&self) -> *mut T;
        fn uninitialize_plugin();
    }
}
use to_plugin_table::ToPluginTable;
impl<T: crate::input::InputPlugin + crate::input::__bridge::InputSingleton>
    ToPluginTable<aviutl2_sys::input2::INPUT_PLUGIN_TABLE> for T
{
    fn initialize_plugin(version: u32) -> bool {
        crate::input::__bridge::initialize_plugin::<T>(version)
    }
    fn to_plugin_table(&self) -> *mut aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
        crate::input::__bridge::create_table::<T>()
    }
    fn uninitialize_plugin() {
        crate::input::__bridge::uninitialize_plugin::<T>()
    }
}

#[doc(hidden)]
pub struct InternalGenericPluginState<T: Send + Sync + GenericPlugin> {
    version: u32,
    host: Option<*mut aviutl2_sys::plugin2::HOST_APP_TABLE>,

    global_leak_manager: LeakManager,
    leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + GenericPlugin> InternalGenericPluginState<T> {
    pub fn new(instance: T, version: u32) -> Self {
        Self {
            version,
            host: None,
            input_plugins: Vec::new(),
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
}

pub fn initialize_plugin<T: GenericSingleton>(version: u32) -> bool {
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
pub fn register_plugin<T: GenericSingleton>(
    host: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
) -> std::result::Result<(), ()> {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    let plugin_state = plugin_state.as_mut().ok_or(())?;
    plugin_state.host = Some(host);

    let handle = HostAppTable { internal: host };
    T::register(&plugin_state.instance, handle).map_err(|e| {
        log::error!("Failed to register plugin: {}", e);
        crate::common::alert_error(&e);
    })?;

    Ok(())
}
pub fn uninitialize_plugin<T: GenericSingleton>() {
    let plugin_state = T::__get_singleton_state();
    if let Some(state) = plugin_state.write().unwrap().as_mut() {
        state.teardown_input_plugins();
    }
    *plugin_state.write().unwrap() = None;
}

/// 汎用プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_host_app_plugin {
    ($struct:ident) => {
        ::aviutl2::__internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                $crate::input::__bridge::initialize_plugin::<$struct>(version)
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                $crate::input::__bridge::uninitialize_plugin::<$struct>()
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn RegisterPlugin(host: *mut aviutl2::sys::generic::HOST_APP_TABLE) -> void {
                $crate::generic::__bridge::register_plugin::<$struct>(host)
            }
        }
    };
}
