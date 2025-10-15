use crate::{
    common::{LeakManager, alert_error},
    filter::{
        AudioObjectInfo, FilterConfigItem, FilterPlugin, FilterPluginTable, FilterProcAudio,
        FilterProcVideo, ObjectInfo, SceneInfo, VideoObjectInfo,
    },
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
pub unsafe fn initialize_plugin<T: FilterPlugin>(
    plugin_state: &std::sync::RwLock<Option<InternalFilterPluginState<T>>>,
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
    let plugin = InternalFilterPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    true
}

pub unsafe fn create_table<T: FilterPlugin>(
    plugin_state: &mut InternalFilterPluginState<T>,
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
        | (if plugin_info.as_object {
            aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_INPUT
        } else {
            0
        });

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
        items: config_items as _,
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
    let mut video = unsafe { FilterProcVideo::from_raw(video) };
    if let Err(e) = plugin.proc_video(&plugin_state.config_items, &mut video) {
        log::error!("Error in proc_video: {}", e);
        return false;
    }
    true
}
pub unsafe fn func_proc_audio<T: FilterPlugin>(
    plugin_state: &InternalFilterPluginState<T>,
    audio: *mut aviutl2_sys::filter2::FILTER_PROC_AUDIO,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let plugin = &plugin_state.instance;
    let mut audio = unsafe { FilterProcAudio::from_raw(audio) };
    if let Err(e) = plugin.proc_audio(&plugin_state.config_items, &mut audio) {
        log::error!("Error in proc_audio: {}", e);
        return false;
    }
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
                        &mut PLUGIN
                            .write()
                            .unwrap()
                            .as_mut()
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
                    {
                        // AviUtl2 -> aviutl2-rsの設定の反映は2回行っても特に問題ないはずなので、
                        // read()ロックをアップグレードしてロックが途切れないようにするといった
                        // 高等テクニックは使わない。
                        let plugin_lock = PLUGIN.read().unwrap();
                        let plugin = plugin_lock.as_ref().expect("Plugin not initialized");
                        if plugin.should_apply_configs() {
                            drop(plugin_lock);
                            PLUGIN.write().unwrap().as_mut().unwrap().apply_configs();
                        }
                    }
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
                    {
                        // AviUtl2 -> aviutl2-rsの設定の反映は2回行っても特に問題ないはずなので、
                        // read()ロックをアップグレードしてロックが途切れないようにするといった
                        // 高等テクニックは使わない。
                        let plugin_lock = PLUGIN.read().unwrap();
                        let plugin = plugin_lock.as_ref().expect("Plugin not initialized");
                        if plugin.should_apply_configs() {
                            drop(plugin_lock);
                            PLUGIN.write().unwrap().as_mut().unwrap().apply_configs();
                        }
                    }
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
