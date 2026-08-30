use crate::{
    common::{AnyResult, LeakManager},
    filter::{
        AudioObjectInfo, FilterConfigItem, FilterPlugin, FilterPluginTable, FilterProcAudio,
        FilterProcVideo, FilterUserdata, FilterUserdataHandle, ObjectInfo, SceneInfo,
        VideoObjectInfo,
    },
    utils::catch_unwind_with_panic_info,
};

type UserdataContainer<T> = std::sync::Arc<parking_lot::RwLock<T>>;

impl<T: FilterUserdata> FilterProcAudio<T> {
    unsafe fn from_raw(
        raw_ptr: *const aviutl2_sys::filter2::FILTER_PROC_AUDIO,
    ) -> FilterProcAudio<T> {
        let raw = unsafe { &*raw_ptr };
        FilterProcAudio {
            scene: unsafe { SceneInfo::from_raw(raw.scene) },
            object: unsafe { ObjectInfo::from_raw(raw.object) },
            audio_object: unsafe { AudioObjectInfo::from_raw(raw.object) },
            read_section: unsafe { crate::generic::ReadSection::from_raw(raw.edit) },
            param: unsafe { (&*raw.param).into() },
            userdata: unsafe { userdata_handle_from_raw::<T>(raw.userdata) },
            inner: raw_ptr,
        }
    }
}
impl<T: FilterUserdata> FilterProcVideo<T> {
    unsafe fn from_raw(
        raw_ptr: *const aviutl2_sys::filter2::FILTER_PROC_VIDEO,
    ) -> FilterProcVideo<T> {
        let raw = unsafe { &*raw_ptr };
        FilterProcVideo {
            scene: unsafe { SceneInfo::from_raw(raw.scene) },
            object: unsafe { ObjectInfo::from_raw(raw.object) },
            video_object: unsafe { VideoObjectInfo::from_raw(raw.object) },
            param: unsafe { (&*raw.param).into() },
            read_section: unsafe { crate::generic::ReadSection::from_raw(raw.edit) },
            userdata: unsafe { userdata_handle_from_raw::<T>(raw.userdata) },
            prevent_post_effect: false,
            inner: raw_ptr,
        }
    }
}

unsafe fn userdata_handle_from_raw<T: FilterUserdata>(
    userdata: *mut std::ffi::c_void,
) -> FilterUserdataHandle<T> {
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>() {
        assert!(
            userdata.is_null(),
            "unit filter userdata pointer must be null"
        );
        let unit = ();
        let value = unsafe { std::ptr::read((&raw const unit).cast::<T>()) };
        return FilterUserdataHandle::new(std::sync::Arc::new(parking_lot::RwLock::new(value)));
    }
    assert!(
        !userdata.is_null(),
        "filter userdata pointer must not be null"
    );
    let userdata = unsafe { &*userdata.cast::<UserdataContainer<T>>() };
    FilterUserdataHandle::new(std::sync::Arc::clone(userdata))
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
            layer: raw.layer as u32,
            frame: raw.frame as u32,
            frame_total: raw.frame_total as u32,
            time: raw.time,
            time_total: raw.time_total,
            is_filter_object: (raw.flag & aviutl2_sys::filter2::OBJECT_INFO::FLAG_FILTER_OBJECT)
                != 0,
            frame_s: raw.frame_s as u32,
            frame_e: raw.frame_e as u32,
            effect_layer: raw.effect_layer as u32,
            origin_frame: raw.origin_frame as u32,
        }
    }
}
impl VideoObjectInfo {
    unsafe fn from_raw(raw: *const aviutl2_sys::filter2::OBJECT_INFO) -> VideoObjectInfo {
        let raw = unsafe { &*raw };
        VideoObjectInfo {
            width: raw.width as u32,
            height: raw.height as u32,
            index: raw.index as u32,
            num: if raw.num == 0 {
                None
            } else {
                Some(raw.num as u32)
            },
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
    Self: 'static + Send + Sync + crate::filter::FilterPlugin,
{
    fn __get_singleton_state()
    -> &'static std::sync::RwLock<Option<crate::filter::__bridge::InternalFilterPluginState<Self>>>;
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
            tracing::error!("Failed to initialize plugin: {}", e);
            let _ = crate::logger::write_error_log(&format!("{e}"));
            false
        }
    }
}

pub unsafe fn initialize_plugin_c_unwind<T: FilterSingleton>(version: u32) -> bool {
    match catch_unwind_with_panic_info(|| unsafe { initialize_plugin_c::<T>(version) }) {
        Ok(result) => result,
        Err(panic_info) => {
            tracing::error!(
                "Panic occurred during plugin initialization: {}",
                panic_info
            );
            let _ = crate::logger::write_error_log(&panic_info);
            false
        }
    }
}

pub(crate) fn initialize_plugin<T: FilterSingleton>(version: u32) -> AnyResult<()> {
    crate::common::ensure_minimum_aviutl2_version(version.into())?;
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
    match crate::utils::catch_unwind_with_panic_info(|| unsafe { uninitialize_plugin::<T>() }) {
        Ok(()) => {}
        Err(panic_info) => {
            tracing::error!(
                "Panic occurred during plugin uninitialization: {}",
                panic_info
            );
            let _ = crate::logger::write_error_log(&panic_info);
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

    let name = plugin_info.name.clone();
    let information = plugin_info.information.clone();

    // 2.1.7時点ではHideRuleを末尾に置かないとAviUtl2がバグる
    let first_hide_rule_index = plugin_info
        .config_items
        .iter()
        .position(|item| matches!(item, crate::filter::FilterConfigItem::HideRule(_)));
    if let Some(index) = first_hide_rule_index {
        assert!(
            plugin_info.config_items[index..]
                .iter()
                .all(|item| matches!(item, crate::filter::FilterConfigItem::HideRule(_))),
            "HideRule must be at the end of config_items (AviUtl2 2.1.7 bug)"
        );
    }

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
    let uses_userdata = std::any::TypeId::of::<T::Userdata>() != std::any::TypeId::of::<()>();
    let (func_create, func_destroy) = if uses_userdata {
        if unwind {
            (
                Some(func_create_unwind::<T> as extern "C" fn(i64) -> *mut std::ffi::c_void),
                Some(func_destroy_unwind::<T> as extern "C" fn(i64, *mut std::ffi::c_void)),
            )
        } else {
            (
                Some(func_create::<T> as extern "C" fn(i64) -> *mut std::ffi::c_void),
                Some(func_destroy::<T> as extern "C" fn(i64, *mut std::ffi::c_void)),
            )
        }
    } else {
        (None, None)
    };
    let flag = if uses_userdata {
        plugin_info.flags.to_bits() | aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_USERDATA
    } else {
        plugin_info.flags.to_bits()
    };

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    let table = aviutl2_sys::filter2::FILTER_PLUGIN_TABLE {
        flag,
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
        func_create,
        func_destroy,
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
            tracing::error!("Panic occurred during create_table: {}", panic_info);
            let _ = crate::logger::write_error_log(&panic_info);
            std::ptr::null_mut()
        }
    }
}

fn proc_video_impl<T: FilterSingleton>(
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> AnyResult<bool> {
    let plugin_lock = T::__get_singleton_state();
    anyhow::ensure!(!plugin_lock.is_poisoned(), "Plugin state lock is poisoned");
    update_configs::<T>(plugin_lock);
    let plugin_state = plugin_lock.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");

    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let mut video = unsafe { FilterProcVideo::<T::Userdata>::from_raw(video) };
    plugin.proc_video(&plugin_state.config_items, &mut video)?;
    video.apply_param();
    Ok(video.prevent_post_effect)
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
    let mut audio = unsafe { FilterProcAudio::<T::Userdata>::from_raw(audio) };
    plugin.proc_audio(&plugin_state.config_items, &mut audio)?;
    audio.apply_param();
    Ok(())
}

fn create_userdata_impl<T: FilterSingleton>(effect_id: i64) -> *mut std::ffi::c_void {
    let userdata = std::sync::Arc::new(parking_lot::RwLock::new(T::Userdata::new(effect_id)));
    Box::into_raw(Box::new(userdata)).cast()
}

fn destroy_userdata_impl<T: FilterSingleton>(userdata: *mut std::ffi::c_void) {
    assert!(
        !userdata.is_null(),
        "filter userdata pointer must not be null"
    );
    unsafe {
        drop(Box::from_raw(
            userdata.cast::<UserdataContainer<T::Userdata>>(),
        ));
    }
}

extern "C" fn func_create<T: FilterSingleton>(effect_id: i64) -> *mut std::ffi::c_void {
    create_userdata_impl::<T>(effect_id)
}

extern "C" fn func_create_unwind<T: FilterSingleton>(effect_id: i64) -> *mut std::ffi::c_void {
    match catch_unwind_with_panic_info(|| create_userdata_impl::<T>(effect_id)) {
        Ok(userdata) => userdata,
        Err(panic_info) => {
            tracing::error!("Panic in filter userdata creation: {}", panic_info);
            let _ = crate::logger::write_error_log(&panic_info);
            std::ptr::null_mut()
        }
    }
}

extern "C" fn func_destroy<T: FilterSingleton>(_effect_id: i64, userdata: *mut std::ffi::c_void) {
    destroy_userdata_impl::<T>(userdata);
}

extern "C" fn func_destroy_unwind<T: FilterSingleton>(
    _effect_id: i64,
    userdata: *mut std::ffi::c_void,
) {
    match catch_unwind_with_panic_info(|| destroy_userdata_impl::<T>(userdata)) {
        Ok(()) => {}
        Err(panic_info) => {
            tracing::error!("Panic in filter userdata destruction: {}", panic_info);
            let _ = crate::logger::write_error_log(&panic_info);
        }
    }
}

extern "C" fn func_proc_video<T: FilterSingleton>(
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> bool {
    match proc_video_impl::<T>(video) {
        Ok(prevent_post_effect) => !prevent_post_effect,
        Err(e) => {
            tracing::error!("Error in proc_video: {}", e);
            false
        }
    }
}
extern "C" fn func_proc_video_unwind<T: FilterSingleton>(
    video: *mut aviutl2_sys::filter2::FILTER_PROC_VIDEO,
) -> bool {
    match catch_unwind_with_panic_info(|| proc_video_impl::<T>(video)) {
        Ok(Ok(prevent_post_effect)) => !prevent_post_effect,
        Ok(Err(e)) => {
            tracing::error!("Error in proc_video: {}", e);
            false
        }
        Err(e) => {
            tracing::error!("Panic in proc_video: {}", e);
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
            tracing::error!("Error in proc_audio: {}", e);
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
            tracing::error!("Error in proc_audio: {}", e);
            false
        }
        Err(e) => {
            tracing::error!("Panic in proc_audio: {}", e);
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
        $crate::__internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn RequiredVersion() -> u32 {
                $crate::MINIMUM_AVIUTL2_VERSION.into()
            }

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
            unsafe extern "C" fn InitializeCache(
                cache: *mut $crate::sys::cache2::CACHE_HANDLE
            ) {
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
                        $crate::cache::__initialize_cache_unwind(cache)
                    } else {
                        $crate::cache::__initialize_cache(cache)
                    }
                }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe {
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
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
                    $crate::comptime_if::comptime_if! {
                        if unwind where (unwind = true, $( $key = $value ),* ) {
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
                $crate::comptime_if::comptime_if! {
                    if unwind where (unwind = true, $( $key = $value ),* ) {
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

#[cfg(test)]
mod tests {
    use super::*;

    static LAST_EFFECT_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);
    static DROP_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    struct TestUserdata {
        value: i32,
    }

    impl Drop for TestUserdata {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl FilterUserdata for TestUserdata {
        fn new(effect_id: i64) -> Self {
            LAST_EFFECT_ID.store(effect_id, std::sync::atomic::Ordering::SeqCst);
            Self { value: 42 }
        }
    }

    struct TestPlugin;

    impl FilterPlugin for TestPlugin {
        type Userdata = TestUserdata;

        fn new(_info: crate::common::AviUtl2Info) -> crate::common::AnyResult<Self> {
            Ok(Self)
        }

        fn plugin_info(&self) -> FilterPluginTable {
            FilterPluginTable {
                name: String::new(),
                label: None,
                information: String::new(),
                flags: Default::default(),
                config_items: Vec::new(),
            }
        }
    }

    impl FilterSingleton for TestPlugin {
        fn __get_singleton_state()
        -> &'static std::sync::RwLock<Option<InternalFilterPluginState<Self>>> {
            static STATE: std::sync::RwLock<Option<InternalFilterPluginState<TestPlugin>>> =
                std::sync::RwLock::new(None);
            &STATE
        }
    }

    struct UnitPlugin;

    impl FilterPlugin for UnitPlugin {
        type Userdata = ();

        fn new(_info: crate::common::AviUtl2Info) -> crate::common::AnyResult<Self> {
            Ok(Self)
        }

        fn plugin_info(&self) -> FilterPluginTable {
            FilterPluginTable {
                name: String::new(),
                label: None,
                information: String::new(),
                flags: Default::default(),
                config_items: Vec::new(),
            }
        }
    }

    impl FilterSingleton for UnitPlugin {
        fn __get_singleton_state()
        -> &'static std::sync::RwLock<Option<InternalFilterPluginState<Self>>> {
            static STATE: std::sync::RwLock<Option<InternalFilterPluginState<UnitPlugin>>> =
                std::sync::RwLock::new(None);
            &STATE
        }
    }

    #[test]
    fn userdata_lifecycle_keeps_value_alive_while_handle_exists() {
        LAST_EFFECT_ID.store(-1, std::sync::atomic::Ordering::SeqCst);
        DROP_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);

        let raw = create_userdata_impl::<TestPlugin>(123);
        let handle = unsafe { userdata_handle_from_raw::<TestUserdata>(raw) };

        assert_eq!(
            LAST_EFFECT_ID.load(std::sync::atomic::Ordering::SeqCst),
            123
        );
        assert_eq!(handle.read().value, 42);

        drop(handle);
        destroy_userdata_impl::<TestPlugin>(raw);
        assert_eq!(DROP_COUNT.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn userdata_handle_locks_on_demand() {
        let handle = FilterUserdataHandle::new(std::sync::Arc::new(parking_lot::RwLock::new(
            TestUserdata { value: 1 },
        )));

        let read = handle.read();
        assert_eq!(read.value, 1);
        assert!(handle.try_read().is_some());
        assert!(handle.try_write().is_none());
        drop(read);

        {
            let mut write = handle.write();
            write.value = 2;
            assert!(handle.try_read().is_none());
            assert!(handle.try_write().is_none());
        }
        assert_eq!(handle.read().value, 2);
    }

    #[test]
    fn unit_userdata_uses_null_sdk_pointer() {
        let handle = unsafe { userdata_handle_from_raw::<()>(std::ptr::null_mut()) };

        assert_eq!(*handle.read(), ());
        assert!(handle.try_write().is_some());
    }

    #[test]
    fn plugin_table_enables_callbacks_only_for_non_unit_userdata() {
        *TestPlugin::__get_singleton_state().write().unwrap() =
            Some(InternalFilterPluginState::new(TestPlugin));
        let table = create_table_impl::<TestPlugin>(false);
        let table = unsafe { &*table };
        assert_ne!(
            table.flag & aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_USERDATA,
            0
        );
        assert!(table.func_create.is_some());
        assert!(table.func_destroy.is_some());
        *TestPlugin::__get_singleton_state().write().unwrap() = None;

        *UnitPlugin::__get_singleton_state().write().unwrap() =
            Some(InternalFilterPluginState::new(UnitPlugin));
        let table = create_table_impl::<UnitPlugin>(false);
        let table = unsafe { &*table };
        assert_eq!(
            table.flag & aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_USERDATA,
            0
        );
        assert!(table.func_create.is_none());
        assert!(table.func_destroy.is_none());
        *UnitPlugin::__get_singleton_state().write().unwrap() = None;
    }
}
