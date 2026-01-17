use crate::{
    common::{AnyResult, LeakManager, alert_error},
    generic::{
        GenericPlugin, ProjectFile,
        binding::{HostAppHandle, PluginRegistry},
    },
};

#[doc(hidden)]
pub struct InternalGenericPluginState<T: Send + Sync + GenericPlugin> {
    plugin_registry: PluginRegistry,

    kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    global_leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + GenericPlugin> InternalGenericPluginState<T> {
    pub fn new(instance: T) -> Self {
        Self {
            plugin_registry: PluginRegistry::new(),
            kill_switch: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            global_leak_manager: LeakManager::new(),
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

pub unsafe fn initialize_plugin_c<T: GenericSingleton>(version: u32) -> bool {
    match initialize_plugin::<T>(version) {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub(crate) fn initialize_plugin<T: GenericSingleton>(version: u32) -> AnyResult<()> {
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = T::new(info)?;
    let plugin = InternalGenericPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    Ok(())
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
            host,
            &mut plugin_state.global_leak_manager,
            kill_switch,
            &mut plugin_state.plugin_registry,
        )
    };
    handle.register_project_load_handler(on_project_load::<T>);
    handle.register_project_save_handler(on_project_save::<T>);
    handle.register_clear_cache_handler(on_clear_cache::<T>);
    handle.register_change_scene_handler(on_change_scene::<T>);
    T::register(&mut plugin_state.instance, &mut handle);

    extern "C" fn on_project_load<T: GenericSingleton>(
        project: *mut aviutl2_sys::plugin2::PROJECT_FILE,
    ) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let mut project = ProjectFile::from_raw(project);
            instance.on_project_load(&mut project);
        });
    }

    extern "C" fn on_project_save<T: GenericSingleton>(
        project: *mut aviutl2_sys::plugin2::PROJECT_FILE,
    ) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let mut project = ProjectFile::from_raw(project);
            instance.on_project_save(&mut project);
        });
    }

    extern "C" fn on_clear_cache<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let edit_section = crate::generic::EditSection::from_raw(edit_section);
            instance.on_clear_cache(&edit_section);
        });
    }

    extern "C" fn on_change_scene<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let edit_section = crate::generic::EditSection::from_raw(edit_section);
            instance.on_change_scene(&edit_section);
        });
    }
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
            unsafe extern "C" fn InitializeLogger(logger: *mut $crate::sys::logger2::LOG_HANDLE) {
                $crate::logger::__initialize_logger(logger)
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe { $crate::generic::__bridge::initialize_plugin_c::<$struct>(version) }
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
