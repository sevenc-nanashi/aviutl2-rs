use std::num::NonZeroIsize;

use crate::{
    common::{AnyResult, LeakManager, alert_error, format_file_filters, load_wide_string},
    input::{
        AudioFormat, AudioInputInfo, AudioReturner, ImageReturner, InputInfo, InputPixelFormat,
        InputPlugin, InputPluginTable, VideoInputInfo,
    },
};

impl InputPixelFormat {
    fn bytes_count_per_pixel(&self) -> usize {
        match self {
            InputPixelFormat::Bgr => 3,  // RGB format
            InputPixelFormat::Bgra => 4, // RGBA format
            InputPixelFormat::Yuy2 => 2, // YUY2 format (packed YUV 4:2:2, 4 bytes per 2 pixels)
            InputPixelFormat::Pa64 => 8, // DXGI_FORMAT_R16G16B16A16_UNORM (packed 16-bit per channel)
            InputPixelFormat::Hf64 => 8, // DXGI_FORMAT_R16G16B16A16_FLOAT (half-float)
            InputPixelFormat::Yc48 => 6, // YC48 (AviUtl1)
        }
    }
}

impl AudioFormat {
    fn bytes_per_sample(&self) -> usize {
        match self {
            AudioFormat::IeeeFloat32 => 4, // 32-bit float
            AudioFormat::Pcm16 => 2,       // 16-bit PCM
        }
    }
}

impl VideoInputInfo {
    fn into_raw(self) -> aviutl2_sys::input2::BITMAPINFOHEADER {
        let bi_compression = match self.format {
            InputPixelFormat::Bgr | InputPixelFormat::Bgra => aviutl2_sys::common::BI_RGB,
            InputPixelFormat::Yuy2 => aviutl2_sys::common::BI_YUY2,
            InputPixelFormat::Pa64 => aviutl2_sys::common::BI_PA64,
            InputPixelFormat::Hf64 => aviutl2_sys::common::BI_HF64,
            InputPixelFormat::Yc48 => aviutl2_sys::common::BI_YC48,
        };

        // NOTE:
        // biHeightをマイナスにしてBI_RGBでも上からにするやつは使えない（AviUtが落ちる）
        aviutl2_sys::input2::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<aviutl2_sys::input2::BITMAPINFOHEADER>() as u32,
            biWidth: self.width as i32,
            biHeight: self.height as i32,
            biPlanes: 1,
            biBitCount: (self.format.bytes_count_per_pixel() * 8) as u16, // Bits per pixel
            biCompression: bi_compression,
            biSizeImage: (self.width * self.height * self.format.bytes_count_per_pixel() as u32),
            biXPelsPerMeter: 0, // Not used
            biYPelsPerMeter: 0, // Not used
            biClrUsed: 0,       // Not used
            biClrImportant: 0,  // Not used
        }
    }
}

impl AudioInputInfo {
    fn into_raw(self) -> aviutl2_sys::input2::WAVEFORMATEX {
        let format = match self.format {
            AudioFormat::IeeeFloat32 => aviutl2_sys::common::WAVE_FORMAT_IEEE_FLOAT,
            AudioFormat::Pcm16 => aviutl2_sys::common::WAVE_FORMAT_PCM,
        };
        let bytes_per_sample = self.format.bytes_per_sample();
        aviutl2_sys::input2::WAVEFORMATEX {
            wFormatTag: format as u16,
            nChannels: self.channels,
            nSamplesPerSec: self.sample_rate,
            nAvgBytesPerSec: (self.sample_rate
                * (self.channels as u32)
                * (bytes_per_sample as u32)),
            nBlockAlign: (self.channels * bytes_per_sample as u16),
            wBitsPerSample: u16::try_from(bytes_per_sample * 8usize)
                .expect("Invalid bits per sample"),
            cbSize: 0, // No extra data
        }
    }
}

#[doc(hidden)]
pub struct InternalInputPluginState<T: Send + Sync + InputPlugin> {
    plugin_info: InputPluginTable,
    global_leak_manager: LeakManager,
    leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + InputPlugin> InternalInputPluginState<T> {
    pub fn new(instance: T) -> Self {
        let plugin_info = instance.plugin_info();
        Self {
            plugin_info,
            global_leak_manager: LeakManager::new(),
            leak_manager: LeakManager::new(),
            instance,
        }
    }
}

struct InternalInputHandle<T: Send + Sync> {
    input_info: Option<InputInfo>,
    num_tracks: std::sync::Mutex<Option<AnyResult<(u32, u32)>>>,
    current_video_track: std::sync::OnceLock<u32>,
    current_audio_track: std::sync::OnceLock<u32>,

    handle: T,
}

pub unsafe fn initialize_plugin_c<T: InputSingleton>(version: u32) -> bool {
    match initialize_plugin::<T>(version) {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to initialize plugin: {}", e);
            alert_error(&e);
            false
        }
    }
}

pub(crate) fn initialize_plugin<T: InputSingleton>(version: u32) -> AnyResult<()> {
    let plugin_state = T::__get_singleton_state();
    let info = crate::common::AviUtl2Info {
        version: version.into(),
    };
    let internal = T::new(info)?;
    let plugin = InternalInputPluginState::new(internal);
    *plugin_state.write().unwrap() = Some(plugin);

    Ok(())
}

pub unsafe fn uninitialize_plugin<T: InputSingleton>() {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    *plugin_state = None;
}

pub unsafe fn create_table<T: InputSingleton>() -> *mut aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
    let plugin_state = T::__get_singleton_state();
    let mut plugin_state = plugin_state.write().unwrap();
    let plugin_state = plugin_state.as_mut().expect("Plugin not initialized");
    let plugin_info = &plugin_state.plugin_info;
    let file_filter = format_file_filters(&plugin_info.file_filters);

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

    let mut flag = plugin_info.input_type.to_bits();
    if plugin_info.concurrent {
        flag |= aviutl2_sys::input2::INPUT_PLUGIN_TABLE::FLAG_CONCURRENT;
    }
    flag |= aviutl2_sys::input2::INPUT_PLUGIN_TABLE::FLAG_MULTI_TRACK;

    // NOTE: プラグイン名などの文字列はAviUtlが終了するまで解放しない
    let table = aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
        flag,
        name: plugin_state.global_leak_manager.leak_as_wide_string(&name),
        filefilter: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&file_filter),
        information: plugin_state
            .global_leak_manager
            .leak_as_wide_string(&information),
        func_open: Some(func_open::<T>),
        func_close: Some(func_close::<T>),
        func_info_get: Some(func_info_get::<T>),
        func_read_video: Some(func_read_video::<T>),
        func_read_audio: Some(func_read_audio::<T>),
        func_config: plugin_info.can_config.then_some(func_config::<T>),
        func_set_track: Some(func_set_track::<T>),
        func_time_to_frame: Some(func_time_to_frame::<T>),
    };
    let table = Box::new(table);
    Box::leak(table)
}

extern "C" fn func_open<T: InputSingleton>(
    file: aviutl2_sys::common::LPCWSTR,
) -> aviutl2_sys::input2::INPUT_HANDLE {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let path = unsafe { load_wide_string(file) };
    log::info!("func_open called with path: {}", path);
    let plugin = &plugin_state.instance;
    match plugin.open(std::path::PathBuf::from(path)) {
        Ok(handle) => {
            let boxed_handle: Box<InternalInputHandle<T::InputHandle>> =
                Box::new(InternalInputHandle {
                    input_info: None,
                    num_tracks: std::sync::Mutex::new(None),
                    current_video_track: std::sync::OnceLock::new(),
                    current_audio_track: std::sync::OnceLock::new(),
                    handle,
                });
            Box::into_raw(boxed_handle) as aviutl2_sys::input2::INPUT_HANDLE
        }
        Err(e) => {
            log::error!("Error during func_open: {}", e);
            std::ptr::null_mut()
        }
    }
}
extern "C" fn func_close<T: InputSingleton>(ih: aviutl2_sys::input2::INPUT_HANDLE) -> bool {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let handle = unsafe { Box::from_raw(ih as *mut InternalInputHandle<T::InputHandle>) };
    let plugin = &plugin_state.instance;
    match plugin.close(handle.handle) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error during func_close: {}", e);
            false
        }
    }
}
extern "C" fn func_info_get<T: InputSingleton>(
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    iip: *mut aviutl2_sys::input2::INPUT_INFO,
) -> bool {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };
    let video_track = {
        *handle
            .current_video_track
            .get()
            .expect("unreachable: func_set_track should have been called before func_info_get")
    };
    let audio_track = {
        *handle
            .current_audio_track
            .get()
            .expect("unreachable: func_set_track should have been called before func_info_get")
    };
    let plugin = &plugin_state.instance;

    match T::get_input_info(plugin, &mut handle.handle, video_track, audio_track) {
        Ok(info) => {
            handle.input_info = Some(info.clone());
            if let Some(video_info) = info.video {
                let fps = video_info.fps;
                let num_frames = video_info.num_frames;
                let manual_frame_index = video_info.manual_frame_index;
                let width = video_info.width;
                let height = video_info.height;
                let image_format = video_info.into_raw();
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_VIDEO;
                    if manual_frame_index {
                        (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_TIME_TO_FRAME;
                    }
                    (*iip).rate = *fps.numer();
                    (*iip).scale = *fps.denom();
                    (*iip).n = num_frames as _;
                    (*iip).format = plugin_state.leak_manager.leak(image_format);
                    (*iip).format_size = (4 * width * height) as i32; // 4 bytes per pixel for RGBA
                    (*iip).audio_n = 0;
                    (*iip).audio_format = std::ptr::null_mut();
                    (*iip).audio_format_size = 0;
                }
            }

            if let Some(audio_info) = info.audio {
                let num_samples = audio_info.num_samples;
                let audio_format = audio_info.into_raw();
                let audio_format_size = std::mem::size_of_val(&audio_format) as i32;
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_AUDIO;
                    (*iip).audio_n = num_samples as _;
                    (*iip).audio_format = plugin_state.leak_manager.leak(audio_format);
                    (*iip).audio_format_size = audio_format_size;
                }
            }

            true
        }
        Err(e) => {
            log::error!("Error during func_info_get: {}", e);
            false
        }
    }
}
extern "C" fn func_read_video<T: InputSingleton>(
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    frame: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };
    let plugin = &plugin_state.instance;
    let frame = frame as u32;
    let mut returner = unsafe { ImageReturner::new(buf as *mut u8) };
    let read_result = if plugin_state.plugin_info.concurrent {
        T::read_video(plugin, &handle.handle, frame, &mut returner)
    } else {
        T::read_video_mut(plugin, &mut handle.handle, frame, &mut returner)
    };
    match read_result {
        Ok(()) => {
            #[cfg(debug_assertions)]
            {
                let video_format = handle
                    .input_info
                    .as_ref()
                    .expect("Unreachable: Input info not set")
                    .video
                    .as_ref()
                    .expect("Unreachable: Video format not set");
                assert_eq!(
                    returner.written,
                    ((video_format.width * video_format.height) as usize
                        * video_format.format.bytes_count_per_pixel()),
                    "Image data size does not match expected size"
                );
            }
            returner.written as i32
        }
        Err(e) => {
            log::error!("Error during func_read_video: {}", e);
            0
        }
    }
}

extern "C" fn func_read_audio<T: InputSingleton>(
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    start: i32,
    length: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };
    let plugin = &plugin_state.instance;
    let mut returner = unsafe { AudioReturner::new(buf as *mut u8) };
    let read_result = if plugin_state.plugin_info.concurrent {
        T::read_audio(plugin, &handle.handle, start, length, &mut returner)
    } else {
        T::read_audio_mut(plugin, &mut handle.handle, start, length, &mut returner)
    };
    match read_result {
        Ok(()) => {
            #[cfg(debug_assertions)]
            {
                let audio_format = handle
                    .input_info
                    .as_ref()
                    .expect("Unreachable: Input info not set")
                    .audio
                    .as_ref()
                    .expect("Unreachable: Audio format not set");
                assert_eq!(
                    returner.written,
                    ((length as usize)
                        * (audio_format.channels as usize)
                        * audio_format.format.bytes_per_sample()),
                    "Audio data size does not match expected size"
                );
            }
            returner.written as i32
        }
        Err(e) => {
            log::error!("Error during func_read_audio: {}", e);
            0
        }
    }
}

extern "C" fn func_config<T: InputSingleton>(
    hwnd: aviutl2_sys::input2::HWND,
    dll_hinst: aviutl2_sys::input2::HINSTANCE,
) -> bool {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let mut handle =
        raw_window_handle::Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
    handle.hinstance = Some(NonZeroIsize::new(dll_hinst as isize).unwrap());
    let plugin = &plugin_state.instance;
    match plugin.config(handle) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error during func_config: {}", e);
            alert_error(&e);
            false
        }
    }
}
extern "C" fn func_set_track<T: InputSingleton>(
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    track_type: i32,
    track: i32,
) -> i32 {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };
    let plugin = &plugin_state.instance;
    if track == -1 {
        // track == -1：トラック数取得
        if handle.num_tracks.lock().unwrap().is_none() {
            let num_tracks = plugin.get_track_count(&mut handle.handle).map_err(|e| {
                log::error!("Failed to get track count: {}", e);
                e
            });

            if matches!(num_tracks, Ok((0, _))) {
                handle
                    .current_video_track
                    .set(0)
                    .expect("unreachable: func_set_track should only be called once per handle");
            }
            if matches!(num_tracks, Ok((_, 0))) {
                handle
                    .current_audio_track
                    .set(0)
                    .expect("unreachable: func_set_track should only be called once per handle");
            }
            *handle.num_tracks.lock().unwrap() = Some(num_tracks);
        }
        match &*handle.num_tracks.lock().unwrap() {
            Some(Ok((video_tracks, audio_tracks))) => {
                if track_type == aviutl2_sys::input2::INPUT_PLUGIN_TABLE::TRACK_TYPE_VIDEO {
                    *video_tracks as i32
                } else if track_type == aviutl2_sys::input2::INPUT_PLUGIN_TABLE::TRACK_TYPE_AUDIO {
                    *audio_tracks as i32
                } else {
                    log::error!("Invalid track type: {}", track_type);
                    -1 // Invalid track type
                }
            }
            Some(Err(e)) => {
                log::error!("Error occurred while getting track count: {}", e);
                -1 // Error occurred
            }
            None => {
                unreachable!("Track count should have been initialized before this point");
            }
        }
    } else {
        // track != -1：トラック設定
        match track_type {
            aviutl2_sys::input2::INPUT_PLUGIN_TABLE::TRACK_TYPE_VIDEO => {
                let new_track = plugin
                    .can_set_video_track(&mut handle.handle, track as u32)
                    .map_or_else(
                        |e| {
                            log::debug!("Failed to set video track: {}", e);
                            -1
                        },
                        |t| t as i32,
                    );
                handle
                    .current_video_track
                    .set(new_track as u32)
                    .expect("unreachable: func_set_track should only be called once per handle");
                new_track
            }
            aviutl2_sys::input2::INPUT_PLUGIN_TABLE::TRACK_TYPE_AUDIO => {
                let new_track = plugin
                    .can_set_audio_track(&mut handle.handle, track as u32)
                    .map_or_else(
                        |e| {
                            log::debug!("Failed to set audio track: {}", e);
                            -1
                        },
                        |t| t as i32,
                    );
                handle
                    .current_audio_track
                    .set(new_track as u32)
                    .expect("unreachable: func_set_track should only be called once per handle");
                new_track
            }
            _ => -1, // Invalid track type
        }
    }
}
extern "C" fn func_time_to_frame<T: InputSingleton>(
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    time: f64,
) -> i32 {
    let plugin_state = T::__get_singleton_state();
    let plugin_state = plugin_state.read().unwrap();
    let plugin_state = plugin_state.as_ref().expect("Plugin not initialized");
    plugin_state.leak_manager.free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };
    let video_track = {
        *handle
            .current_video_track
            .get()
            .expect("unreachable: func_set_track should have been called before func_time_to_frame")
    };
    let plugin = &plugin_state.instance;
    match T::time_to_frame(plugin, &mut handle.handle, video_track, time) {
        Ok(frame) => frame as i32,
        Err(e) => {
            log::error!("Error during func_time_to_frame: {}", e);
            0
        }
    }
}

pub trait InputSingleton
where
    Self: 'static + Send + Sync + InputPlugin,
{
    fn __get_singleton_state() -> &'static std::sync::RwLock<Option<InternalInputPluginState<Self>>>;
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

/// 入力プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_input_plugin {
    ($struct:ident) => {
        ::aviutl2::__internal_module! {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn InitializePlugin(version: u32) -> bool {
                unsafe { $crate::input::__bridge::initialize_plugin_c::<$struct>(version) }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn UninitializePlugin() {
                unsafe { $crate::input::__bridge::uninitialize_plugin::<$struct>() }
            }

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetInputPluginTable()
            -> *mut aviutl2::sys::input2::INPUT_PLUGIN_TABLE {
                unsafe { $crate::input::__bridge::create_table::<$struct>() }
            }
        }
    };
}
