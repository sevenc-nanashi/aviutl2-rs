use std::num::NonZeroIsize;

use crate::{
    common::{
        AnyResult, LeakManager, alert_error, format_file_filters, leak_and_forget_as_wide_string,
        load_wide_string,
    },
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

pub struct InternalInputPluginState<T: Send + Sync + InputPlugin> {
    plugin_info: InputPluginTable,
    leak_manager: LeakManager,

    instance: T,
}

impl<T: Send + Sync + InputPlugin> InternalInputPluginState<T> {
    pub fn new(instance: T) -> Self {
        let plugin_info = instance.plugin_info();
        Self {
            plugin_info,
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

#[allow(clippy::too_many_arguments)]
pub unsafe fn create_table<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    func_open: extern "C" fn(aviutl2_sys::common::LPCWSTR) -> aviutl2_sys::input2::INPUT_HANDLE,
    func_close: extern "C" fn(aviutl2_sys::input2::INPUT_HANDLE) -> bool,
    func_info_get: extern "C" fn(
        aviutl2_sys::input2::INPUT_HANDLE,
        *mut aviutl2_sys::input2::INPUT_INFO,
    ) -> bool,
    func_read_video: extern "C" fn(
        aviutl2_sys::input2::INPUT_HANDLE,
        i32,
        *mut std::ffi::c_void,
    ) -> i32,
    func_read_audio: extern "C" fn(
        aviutl2_sys::input2::INPUT_HANDLE,
        i32,
        i32,
        *mut std::ffi::c_void,
    ) -> i32,
    func_config: extern "C" fn(aviutl2_sys::input2::HWND, aviutl2_sys::input2::HINSTANCE) -> bool,
    func_set_track: extern "C" fn(aviutl2_sys::input2::INPUT_HANDLE, i32, i32) -> i32,
    func_time_to_frame: extern "C" fn(aviutl2_sys::input2::INPUT_HANDLE, f64) -> i32,
) -> aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
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
    aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
        flag,
        name: leak_and_forget_as_wide_string(&name),
        filefilter: leak_and_forget_as_wide_string(&file_filter),
        information: leak_and_forget_as_wide_string(&information),
        func_open: Some(func_open),
        func_close: Some(func_close),
        func_info_get: Some(func_info_get),
        func_read_video: Some(func_read_video),
        func_read_audio: Some(func_read_audio),
        func_config: plugin_info.can_config.then_some(func_config),
        func_set_track: Some(func_set_track),
        func_time_to_frame: Some(func_time_to_frame),
    }
}
pub unsafe fn func_open<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    file: aviutl2_sys::common::LPCWSTR,
) -> aviutl2_sys::input2::INPUT_HANDLE {
    plugin_state.leak_manager.free_leaked_memory();
    let path = load_wide_string(file);
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
pub unsafe fn func_close<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
) -> bool {
    plugin_state.leak_manager.free_leaked_memory();
    let handle = *unsafe { Box::from_raw(ih as *mut InternalInputHandle<T::InputHandle>) };
    let plugin = &plugin_state.instance;
    match plugin.close(handle.handle) {
        Ok(()) => true,
        Err(e) => {
            log::error!("Error during func_close: {}", e);
            false
        }
    }
}
pub unsafe fn func_info_get<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    iip: *mut aviutl2_sys::input2::INPUT_INFO,
) -> bool {
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
pub unsafe fn func_read_video<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    frame: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
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

pub unsafe fn func_read_audio<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    start: i32,
    length: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
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

pub unsafe fn func_config<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    hwnd: aviutl2_sys::input2::HWND,
    dll_hinst: aviutl2_sys::input2::HINSTANCE,
) -> bool {
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
pub unsafe fn func_set_track<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    track_type: i32,
    track: i32,
) -> i32 {
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
pub unsafe fn func_time_to_frame<T: InputPlugin>(
    plugin_state: &InternalInputPluginState<T>,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    time: f64,
) -> i32 {
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

/// 入力プラグインを登録するマクロ。
#[macro_export]
macro_rules! register_input_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_input_plugin {
            use super::$struct;
            use $crate::input::InputPlugin as _;

            static PLUGIN: std::sync::LazyLock<
                aviutl2::input::__bridge::InternalInputPluginState<$struct>,
            > = std::sync::LazyLock::new(|| {
                aviutl2::input::__bridge::InternalInputPluginState::new($struct::new())
            });

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetInputPluginTable()
            -> *mut aviutl2::sys::input2::INPUT_PLUGIN_TABLE {
                let table = unsafe {
                    $crate::input::__bridge::create_table::<$struct>(
                        &PLUGIN,
                        func_open,
                        func_close,
                        func_info_get,
                        func_read_video,
                        func_read_audio,
                        func_config,
                        func_set_track,
                        func_time_to_frame,
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_open(
                file: aviutl2::sys::common::LPCWSTR,
            ) -> aviutl2::sys::input2::INPUT_HANDLE {
                unsafe { $crate::input::__bridge::func_open(&*PLUGIN, file) }
            }

            extern "C" fn func_close(ih: aviutl2::sys::input2::INPUT_HANDLE) -> bool {
                unsafe { $crate::input::__bridge::func_close(&*PLUGIN, ih) }
            }

            extern "C" fn func_info_get(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                iip: *mut aviutl2::sys::input2::INPUT_INFO,
            ) -> bool {
                unsafe { $crate::input::__bridge::func_info_get(&*PLUGIN, ih, iip) }
            }

            extern "C" fn func_read_video(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                frame: i32,
                buf: *mut std::ffi::c_void,
            ) -> i32 {
                unsafe { $crate::input::__bridge::func_read_video(&*PLUGIN, ih, frame, buf) }
            }

            extern "C" fn func_read_audio(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                start: i32,
                length: i32,
                buf: *mut std::ffi::c_void,
            ) -> i32 {
                unsafe {
                    $crate::input::__bridge::func_read_audio(&*PLUGIN, ih, start, length, buf)
                }
            }

            extern "C" fn func_config(
                hwnd: aviutl2::sys::input2::HWND,
                dll_hinst: aviutl2::sys::input2::HINSTANCE,
            ) -> bool {
                unsafe { $crate::input::__bridge::func_config(&*PLUGIN, hwnd, dll_hinst) }
            }

            extern "C" fn func_set_track(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                track_type: i32,
                track: i32,
            ) -> i32 {
                unsafe { $crate::input::__bridge::func_set_track(&*PLUGIN, ih, track_type, track) }
            }

            extern "C" fn func_time_to_frame(
                ih: aviutl2::sys::input2::INPUT_HANDLE,
                time: f64,
            ) -> i32 {
                unsafe { $crate::input::__bridge::func_time_to_frame(&*PLUGIN, ih, time) }
            }
        }
    };
}
