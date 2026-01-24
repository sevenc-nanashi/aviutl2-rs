use crate::{
    common::{AnyResult, LeakManager, alert_error},
    filter::{
        AudioObjectInfo, FilterConfigItem, FilterPlugin, FilterPluginTable, FilterProcAudio,
        FilterProcVideo, ObjectInfo, SceneInfo, VideoObjectInfo,
    },
    utils::catch_unwind_with_panic_info,
};

impl FilterProcAudio {
    unsafe fn from_raw(raw_ptr: *const aviutl2_sys::filter2::FILTER_PROC_AUDIO) -> FilterProcAudio {
        let raw = unsafe { &*raw_ptr };
        FilterProcAudio {
            scene: unsafe { SceneInfo::from_raw(raw.scene) },
            object: unsafe { ObjectInfo::from_raw(raw.object) },
            audio_object: unsafe { AudioObjectInfo::from_raw(raw.object) },
            inner: raw_ptr,
        }
    }
}
impl FilterProcVideo {
    unsafe fn from_raw(raw_ptr: *const aviutl2_sys::filter2::FILTER_PROC_VIDEO) -> FilterProcVideo {
        let raw = unsafe { &*raw_ptr };
        FilterProcVideo {
            scene: unsafe { SceneInfo::from_raw(raw.scene) },
            object: unsafe { ObjectInfo::from_raw(raw.object) },
            video_object: unsafe { VideoObjectInfo::from_raw(raw.object) },
            inner: raw_ptr,
        }
    }
}

impl SceneInfo {
    unsafe fn from_raw(raw: *const aviutl2_sys::filter2::SCENE_INFO) -> SceneInfo {
        let raw = unsafe { &*raw };
        SceneInfo {
            width: raw.width as u32,
            height: raw.height as u32,
            frame_rate: num_rational::Rational32::new(raw.rate, raw.scale),
            sample_rate: raw.sample_rate as u32,
        }
    }
}
impl ObjectInfo {
    unsafe fn from_raw(raw: *const aviutl2_sys::filter2::OBJECT_INFO) -> ObjectInfo {
        let raw = unsafe { &*raw };
        ObjectInfo {
            id: raw.id,
            effect_id: raw.effect_id,
            frame: raw.frame as u32,
            frame_total: raw.frame_total as u32,
            time: raw.time,
            time_total: raw.time_total,
        }
    }
}
impl VideoObjectInfo {
    unsafe fn from_raw(raw: *const aviutl2_sys::filter2::OBJECT_INFO) -> VideoObjectInfo {
        let raw = unsafe { &*raw };
        VideoObjectInfo {
            width: raw.width as u32,
            height: raw.height as u32,
        }
    }
}
impl AudioObjectInfo {
    unsafe fn from_raw(raw: *const aviutl2_sys::filter2::OBJECT_INFO) -> AudioObjectInfo {
        let raw = unsafe { &*raw };
        AudioObjectInfo {
            sample_index: raw.sample_index as u64,
            sample_total: raw.sample_total as u64,
            sample_num: raw.sample_num as u32,
            channel_num: raw.channel_num as u32,
        }
    }
}

pub struct InternalFilterPluginState<T: Send + Sync + FilterPlugin> {
    plugin_info: FilterPluginTable,
    global_leak_manager: LeakManager,
    leak_manager: LeakManager,
    config_pointers: Vec<*const aviutl2_sys::filter2::FILTER_ITEM>,
    config_items: Vec<FilterConfigItem>,

    instance: T,
}
unsafe impl<T: Send + Sync + FilterPlugin> Send for InternalFilterPluginState<T> {}
unsafe impl<T: Send + Sync + FilterPlugin> Sync for InternalFilterPluginState<T> {}

impl<T: Send + Sync + FilterPlugin> InternalFilterPluginState<T> {
    pub fn new(instance: T) -> Self {
        let plugin_info = instance.plugin_info();
        let config_items = plugin_info.config_items.clone();
        Self {
            plugin_info,
            global_leak_manager: LeakManager::new(),
            leak_manager: LeakManager::new(),
            config_pointers: Vec::new(),
            config_items,

            instance,
        }
    }

    pub fn should_apply_configs(&self) -> bool {
        for (item, raw) in self.config_items.iter().zip(self.config_pointers.iter()) {
            if unsafe { item.should_apply_from_raw(*raw) } {
                return true;
            }
        }
        false
    }

    pub fn apply_configs(&mut self) {
        for (item, raw) in self
            .config_items
            .iter_mut()
            .zip(self.config_pointers.iter())
        {
            unsafe { item.apply_from_raw(*raw) };
        }
    }
}

fn update_configs<T: Send + Sync + FilterPlugin>(
    plugin_state: &std::sync::RwLock<Option<InternalFilterPluginState<T>>>,
) {
    // AviUtl2 -> aviutl2-rsの設定の反映は2回行っても特に問題ないはずなので、
    // read()ロックをアップグレードしてロックが途切れないようにするといった
    // 高等テクニックは使わない。
    let plugin_lock = plugin_state.read().unwrap();
    let plugin = plugin_lock.as_ref().expect("Plugin not initialized");
    if plugin.should_apply_configs() {
        drop(plugin_lock);
        plugin_state
            .write()
            .unwrap()
            .as_mut()
            .unwrap()
            .apply_configs();
    }
}

pub trait FilterSingleton
where
    Self: 'static + Send + Sync + FilterPlugin,
{
    fn __get_singleton_state() -> &'static std::sync::RwLock<Option<InternalFilterPluginState<Self>>>;
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

pub unsafe fn initialize_plugin_c<T: FilterSingleton>(version: u32) -> bool {
    match initialize_plugin::<T>(version) {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub unsafe fn initialize_plugin_c_unwind<T: FilterSingleton>(version: u32) -> bool {
    match catch_unwind_with_panic_info(|| unsafe { initialize_plugin_c::<T>(version) }) {
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

pub(crate) fn initialize_plugin<T: FilterSingleton>(version: u32) -> AnyResult<()> {
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = T::new(info)?;
    let plugin = InternalFilterPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    Ok(())
}
pub unsafe fn uninitialize_plugin<T: FilterSingleton>() {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    *plugin_state = None;
}

pub unsafe fn uninitialize_plugin_c_unwind<T: FilterSingleton>() {
    match crate::utils::catch_unwind_with_panic_info(|| unsafe {
        uninitialize_plugin::<T>()
    }) {
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
fn create_table_impl<T: FilterSingleton>(
    unwind: bool,
) -> *mut aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    let plugin_state = plugin_state.as_mut().expect("Plugin not initialized");
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

    let config_items = plugin_info
        .config_items
        .iter()
        .map(|item| {
            plugin_state
                .global_leak_manager
                .leak(item.to_raw(&plugin_state.global_leak_manager))
        })
        .collect::<Vec<_>>();
    plugin_state.config_pointers = config_items.to_vec();
    // null終端
    plugin_state
        .config_pointers
        .push(std::ptr::null::<aviutl2_sys::filter2::FILTER_ITEM>());
    let config_items = plugin_state.global_leak_manager.leak_value_vec(
        plugin_state
            .config_pointers
            .iter()
            .map(|p| *p as usize)
            .collect(),
    );

    let func_proc_video = if unwind {
        func_proc_video_unwind::<T>
    } else {
        func_proc_video::<T>
    };
    let func_proc_audio = if unwind {
        func_proc_audio_unwind::<T>
    } else {
        func_proc_audio::<T>
    };

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    let table = aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
        flag: plugin_info.flags.to_bits(),
        name: plugin_state.global_leak_manager.leak_as_wide_string(&name),
        information: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&information),
        label: plugin_info.label.as_ref().map_or(std::ptr::null(), |s| {
            plugin_state.global_leak_manager.leak_as_wide_string(s)
        }),
        items: config_items as _,
        func_proc_video: Some(func_proc_video),
        func_proc_audio: Some(func_proc_audio),
    };
    let table = Box::new(table);
    Box::leak(table)
}

pub unsafe fn create_table<T: FilterSingleton>() -> *mut aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
    create_table_impl::<T>(false)
}

pub unsafe fn create_table_unwind<T: FilterSingleton>()
-> *mut aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
    match crate::utils::catch_unwind_with_panic_info(|| create_table_impl::<T>(true)) {
        Ok(table) => table,
        Err(panic_info) => {
            log::error!("Panic occurred during create_table: {}", panic_info);
            alert_error(&panic_info);
            std::ptr::null_mut()
        }
    }
}

fn proc_video_impl<T: FilterSingleton>(
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> AnyResult<()> {
    let plugin_lock = T::__get_singleton_state();
    anyhow::ensure!(!plugin_lock.is_poisoned(), "Plugin state lock is poisoned");
    update_configs::<T>(plugin_lock);
    let plugin_state = plugin_lock.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");

    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let mut video = unsafe { FilterProcVideo::from_raw(video) };
    plugin.proc_video(&plugin_state.config_items, &mut video)
}

fn proc_audio_impl<T: FilterSingleton>(
    audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO,
) -> AnyResult<()> {
    let plugin_lock = T::__get_singleton_state();
    update_configs::<T>(plugin_lock);
    let plugin_state = plugin_lock.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let mut audio = unsafe { FilterProcAudio::from_raw(audio) };
    plugin.proc_audio(&plugin_state.config_items, &mut audio)
}

extern "C" fn func_proc_video<T: FilterSingleton>(
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> bool {
    match proc_video_impl::<T>(video) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error in proc_video: {}", e);
            false
        }
    }
}
extern "C" fn func_proc_video_unwind<T: FilterSingleton>(
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> bool {
    match catch_unwind_with_panic_info(|| proc_video_impl::<T>(video)) {
        Ok(Ok(())) => true,
        Ok(Err(e)) => {
            log::error!("Error in proc_video: {}", e);
            false
        }
        Err(e) => {
            log::error!("Panic in proc_video: {}", e);
            false
        }
    }
}
extern "C" fn func_proc_audio<T: FilterSingleton>(
    audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO,
) -> bool {
    match proc_audio_impl::<T>(audio) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error in proc_audio: {}", e);
            false
        }
    }
}
extern "C" fn func_proc_audio_unwind<T: FilterSingleton>(
    audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO,
) -> bool {
    match catch_unwind_with_panic_info(|| proc_audio_impl::<T>(audio)) {
        Ok(Ok(())) => true,
        Ok(Err(e)) => {
            log::error!("Error in proc_audio: {}", e);
            false
        }
        Err(e) => {
            log::error!("Panic in proc_audio: {}", e);
            false
        }
    }
}

/// フィルタプラグインを登録するマクロ。
///
/// # Arguments
///
/// - `unwind`: panic時にunwindするかどうか。デフォルトは`true`。
#[macro_export]
macro_rules! register_filter_plugin {
    ($struct:ident, $($key:ident = $value:expr),* $(,)?) => {
        ::aviutl2::__internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializeLogger(logger: *mut $crate::sys::logger2::LOG_HANDLE) {
                $crate::__macro_if! {
                    if unwind in (unwind = true, $( $key = $value ),* ) {
                        if let Err(panic_info) = $crate::__catch_unwind_with_panic_info(|| {
                            $crate::logger::__initialize_logger(logger)
                        }) {
                            $crate::log::error!(
                                "Panic occurred during InitializeLogger: {}",
                                panic_info
                            );
                            $crate::__alert_error(&panic_info);
                        }
                    } else {
                        $crate::logger::__initialize_logger(logger)
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe {
                    $crate::__macro_if! {
                        if unwind in (unwind = true, $( $key = $value ),* ) {
                            $crate::filter::__bridge::initialize_plugin_c_unwind::<$struct>(version)
                        } else {
                            $crate::filter::__bridge::initialize_plugin_c::<$struct>(version)
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe {
                    $crate::__macro_if! {
                        if unwind in (unwind = true, $( $key = $value ),* ) {
                            $crate::filter::__bridge::uninitialize_plugin_c_unwind::<$struct>()
                        } else {
                            $crate::filter::__bridge::uninitialize_plugin::<$struct>()
                        }
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetFilterPluginTable()
            -> *mut aviutl2::sys::filter2::FILTER_PLUGIN_TABLE {
                $crate::__macro_if! {
                    if unwind in (unwind = true, $( $key = $value ),* ) {
                        unsafe { $crate::filter::__bridge::create_table_unwind::<$struct>() }
                    } else {
                        unsafe { $crate::filter::__bridge::create_table::<$struct>() }
                    }
                }
            }
        }
    };
    ($struct:ident, $($key:ident),* $(,)?) => {
        $crate::register_filter_plugin!($struct, $( $key = true ),* );
    };
    ($struct:ident) => {
        $crate::register_filter_plugin!($struct, );
    };
}
