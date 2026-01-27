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

pub unsafe fn initialize_plugin_c_unwind<T: GenericSingleton>(version: u32) -> bool {
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

pub(crate) fn initialize_plugin<T: GenericSingleton>(version: u32) -> AnyResult<()> {
    crate::common::ensure_minimum_aviutl2_version(version.into())?;
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = T::new(info)?;
    let plugin = InternalGenericPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    Ok(())
}
fn register_plugin_impl<T: GenericSingleton>(
    host: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
    unwind: bool,
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
    if unwind {
        handle.register_project_load_handler(on_project_load_unwind::<T>);
        handle.register_project_save_handler(on_project_save_unwind::<T>);
        handle.register_clear_cache_handler(on_clear_cache_unwind::<T>);
        handle.register_change_scene_handler(on_change_scene_unwind::<T>);
    } else {
        handle.register_project_load_handler(on_project_load::<T>);
        handle.register_project_save_handler(on_project_save::<T>);
        handle.register_clear_cache_handler(on_clear_cache::<T>);
        handle.register_change_scene_handler(on_change_scene::<T>);
    }
    T::register(&mut plugin_state.instance, &mut handle);

    fn on_project_load_impl<T: GenericSingleton>(project: *mut aviutl2_sys::plugin2::PROJECT_FILE) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let mut project = ProjectFile::from_raw(project);
            instance.on_project_load(&mut project);
        });
    }
    extern "C" fn on_project_load<T: GenericSingleton>(
        project: *mut aviutl2_sys::plugin2::PROJECT_FILE,
    ) {
        on_project_load_impl::<T>(project);
    }
    extern "C" fn on_project_load_unwind<T: GenericSingleton>(
        project: *mut aviutl2_sys::plugin2::PROJECT_FILE,
    ) {
        if let Err(panic_info) =
            crate::utils::catch_unwind_with_panic_info(|| on_project_load_impl::<T>(project))
        {
            log::error!("Panic occurred during on_project_load: {}", panic_info);
            alert_error(&panic_info);
        }
    }

    fn on_project_save_impl<T: GenericSingleton>(project: *mut aviutl2_sys::plugin2::PROJECT_FILE) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let mut project = ProjectFile::from_raw(project);
            instance.on_project_save(&mut project);
        });
    }
    extern "C" fn on_project_save<T: GenericSingleton>(
        project: *mut aviutl2_sys::plugin2::PROJECT_FILE,
    ) {
        on_project_save_impl::<T>(project);
    }
    extern "C" fn on_project_save_unwind<T: GenericSingleton>(
        project: *mut aviutl2_sys::plugin2::PROJECT_FILE,
    ) {
        if let Err(panic_info) =
            crate::utils::catch_unwind_with_panic_info(|| on_project_save_impl::<T>(project))
        {
            log::error!("Panic occurred during on_project_save: {}", panic_info);
            alert_error(&panic_info);
        }
    }

    fn on_clear_cache_impl<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let edit_section = crate::generic::EditSection::from_raw(edit_section);
            instance.on_clear_cache(&edit_section);
        });
    }
    extern "C" fn on_clear_cache<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        on_clear_cache_impl::<T>(edit_section);
    }
    extern "C" fn on_clear_cache_unwind<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        if let Err(panic_info) =
            crate::utils::catch_unwind_with_panic_info(|| on_clear_cache_impl::<T>(edit_section))
        {
            log::error!("Panic occurred during on_clear_cache: {}", panic_info);
            alert_error(&panic_info);
        }
    }

    fn on_change_scene_impl<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        <T as GenericSingleton>::with_instance_mut(|instance| unsafe {
            let edit_section = crate::generic::EditSection::from_raw(edit_section);
            instance.on_change_scene(&edit_section);
        });
    }
    extern "C" fn on_change_scene<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        on_change_scene_impl::<T>(edit_section);
    }
    extern "C" fn on_change_scene_unwind<T: GenericSingleton>(
        edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    ) {
        if let Err(panic_info) =
            crate::utils::catch_unwind_with_panic_info(|| on_change_scene_impl::<T>(edit_section))
        {
            log::error!("Panic occurred during on_change_scene: {}", panic_info);
            alert_error(&panic_info);
        }
    }
}

pub unsafe fn register_plugin<T: GenericSingleton>(
    host: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
) {
    register_plugin_impl::<T>(host, false);
}

pub unsafe fn register_plugin_unwind<T: GenericSingleton>(
    host: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
) {
    if let Err(panic_info) =
        crate::utils::catch_unwind_with_panic_info(|| register_plugin_impl::<T>(host, true))
    {
        log::error!("Panic occurred during register_plugin: {}", panic_info);
        alert_error(&panic_info);
    }
}
pub unsafe fn uninitialize_plugin<T: GenericSingleton>() {
    let plugin_state = T::__get_singleton_state();
    *plugin_state.write().unwrap() = None;
}

pub unsafe fn uninitialize_plugin_c_unwind<T: GenericSingleton>() {
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

/// 汎用プラグインを登録するマクロ。
///
/// # Arguments
///
/// - `unwind`: panic時にunwindするかどうか。デフォルトは`true`。
#[macro_export]
macro_rules! register_generic_plugin {
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
            unsafe extern "C" fn InitializeConfig(
                config: *mut $crate::sys::config2::CONFIG_HANDLE
            ) {
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
                        $crate::config::__initialize_config_handle_unwind(config)
                    } else {
                        $crate::config::__initialize_config_handle(config)
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe {
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
                            $crate::generic::__bridge::initialize_plugin_c_unwind::<$struct>(version)
                        } else {
                            $crate::generic::__bridge::initialize_plugin_c::<$struct>(version)
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe {
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
                            $crate::generic::__bridge::uninitialize_plugin_c_unwind::<$struct>()
                        } else {
                            $crate::generic::__bridge::uninitialize_plugin::<$struct>()
                        }
                    }
                };
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn RegisterPlugin(host: *mut aviutl2::sys::plugin2::HOST_APP_TABLE) {
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
                        unsafe { $crate::generic::__bridge::register_plugin_unwind::<$struct>(host) };
                    } else {
                        unsafe { $crate::generic::__bridge::register_plugin::<$struct>(host) };
                    }
                }
            }
        }
    };
    ($struct:ident, $($key:ident),* $(,)?) => {
        $crate::register_generic_plugin!($struct, $( $key = true ),* );
    };
    ($struct:ident) => {
        $crate::register_generic_plugin!($struct, );
    };
}
