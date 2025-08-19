use std::num::NonZeroIsize;

use crate::{
    common::{
        format_file_filters, leak_large_string, load_large_string, result_to_bool_with_dialog,
    },
    input::{
        AudioFormat, AudioInputInfo, ImageFormat, InputPlugin, IntoAudio, IntoImage, VideoInputInfo,
    },
};

pub use raw_window_handle::RawWindowHandle;

impl ImageFormat {
    fn bytes_count(&self) -> usize {
        match self {
            ImageFormat::Rgb => 3,  // RGB format
            ImageFormat::Rgba => 4, // RGBA format
            ImageFormat::Yuy2 => 4, // YUY2 format (packed YUV 4:2:2)
            ImageFormat::Pa64 => 8, // DXGI_FORMAT_R16G16B16A16_UNORM (packed 16-bit per channel)
            ImageFormat::Hf64 => 8, // DXGI_FORMAT_R16G16B16A16_FLOAT (half-float)
            ImageFormat::Yc48 => 6, // YC48 (AviUtl1)
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
            ImageFormat::Rgb | ImageFormat::Rgba => aviutl2_sys::input2::BI_RGB,
            ImageFormat::Yuy2 => aviutl2_sys::input2::BI_YUY2,
            ImageFormat::Pa64 => aviutl2_sys::input2::BI_PA64,
            ImageFormat::Hf64 => aviutl2_sys::input2::BI_HF64,
            ImageFormat::Yc48 => aviutl2_sys::input2::BI_YC48, // Custom format for AviUtl1
        };
        aviutl2_sys::input2::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<aviutl2_sys::input2::BITMAPINFOHEADER>() as u32,
            biWidth: self.width as i32,
            biHeight: self.height as i32,
            biPlanes: 1,
            biBitCount: (self.format.bytes_count() * 8) as u16, // Bits per pixel
            biCompression: bi_compression,
            biSizeImage: (self.width * self.height * self.format.bytes_count() as u32),
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
            AudioFormat::IeeeFloat32 => aviutl2_sys::input2::WAVE_FORMAT_IEEE_FLOAT,
            AudioFormat::Pcm16 => aviutl2_sys::input2::WAVE_FORMAT_PCM,
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

static WILL_FREE_ON_NEXT_CALL: std::sync::LazyLock<std::sync::Mutex<Vec<usize>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

fn free_leaked_memory() {
    let mut will_free = WILL_FREE_ON_NEXT_CALL.lock().unwrap();
    for ptr in will_free.drain(..) {
        unsafe {
            let _ = Box::from_raw(ptr as *mut u8);
        }
    }
}

struct InternalInputHandle<T: Send + Sync> {
    video_format: Option<VideoInputInfo>,
    audio_format: Option<AudioInputInfo>,

    handle: T,
}

pub unsafe fn create_table<T: InputPlugin>(
    plugin: &T,
    func_open: extern "C" fn(aviutl2_sys::input2::LPCWSTR) -> aviutl2_sys::input2::INPUT_HANDLE,
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
) -> aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
    let plugin_info = plugin.plugin_info();
    let file_filter = format_file_filters(&plugin_info.file_filters);

    let name = if cfg!(debug_assertions) {
        format!("{} (Debug)", plugin_info.name)
    } else {
        plugin_info.name.clone()
    };
    let information = if cfg!(debug_assertions) {
        format!("{} (Debug Build)", plugin_info.information)
    } else {
        plugin_info.information.clone()
    };

    let mut flag = plugin_info.input_type.to_bits();
    if plugin_info.concurrent {
        flag |= aviutl2_sys::input2::INPUT_PLUGIN_TABLE::FLAG_CONCURRENT;
    }
    flag |= aviutl2_sys::input2::INPUT_PLUGIN_TABLE::FLAG_MULTI_TRACK;

    aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
        flag,
        name: leak_large_string(&name),
        filefilter: leak_large_string(&file_filter),
        information: leak_large_string(&information),
        func_open: Some(func_open),
        func_close: Some(func_close),
        func_info_get: Some(func_info_get),
        func_read_video: Some(func_read_video),
        func_read_audio: Some(func_read_audio),
        func_config: plugin_info.can_config.then_some(func_config),
        func_set_track: None,     // TODO
        func_time_to_frame: None, // TODO
    }
}
pub unsafe fn func_open<T: InputPlugin>(
    plugin: &T,
    file: aviutl2_sys::input2::LPCWSTR,
) -> aviutl2_sys::input2::INPUT_HANDLE {
    free_leaked_memory();
    let path = load_large_string(file);
    match plugin.open(std::path::PathBuf::from(path)) {
        Ok(handle) => {
            let boxed_handle: Box<InternalInputHandle<T::InputHandle>> =
                Box::new(InternalInputHandle {
                    video_format: None,
                    audio_format: None,
                    handle,
                });
            Box::into_raw(boxed_handle) as aviutl2_sys::input2::INPUT_HANDLE
        }
        Err(_) => std::ptr::null_mut(),
    }
}
pub unsafe fn func_close<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
) -> bool {
    free_leaked_memory();
    let handle = *unsafe { Box::from_raw(ih as *mut InternalInputHandle<T::InputHandle>) };
    (T::close(plugin, handle.handle)).is_ok()
}
pub unsafe fn func_info_get<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    iip: *mut aviutl2_sys::input2::INPUT_INFO,
) -> bool {
    free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };

    match T::get_input_info(plugin, &handle.handle) {
        Ok(info) => {
            if let Some(video_info) = info.video {
                handle.video_format = Some(video_info.clone());
                let fps = video_info.fps;
                let num_frames = video_info.num_frames;
                let width = video_info.width;
                let height = video_info.height;
                let image_format = video_info.into_raw();
                let image_format = Box::new(image_format);
                let image_format_ptr = Box::into_raw(image_format);
                WILL_FREE_ON_NEXT_CALL
                    .lock()
                    .unwrap()
                    .push(image_format_ptr as usize);
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_VIDEO;
                    (*iip).rate = *fps.numer();
                    (*iip).scale = *fps.denom();
                    (*iip).n = num_frames as _;
                    (*iip).format = image_format_ptr;
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
                let audio_format = Box::new(audio_format);
                let audio_format_ptr = Box::into_raw(audio_format);
                WILL_FREE_ON_NEXT_CALL
                    .lock()
                    .unwrap()
                    .push(audio_format_ptr as usize);
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_AUDIO;
                    (*iip).audio_n = num_samples as _;
                    (*iip).audio_format = audio_format_ptr;
                    (*iip).audio_format_size = audio_format_size;
                }
            }

            true
        }
        Err(_) => false,
    }
}
pub unsafe fn func_read_video<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    frame: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    free_leaked_memory();
    let handle = unsafe { &*(ih as *mut InternalInputHandle<T::InputHandle>) };
    match T::read_video(plugin, &handle.handle, frame) {
        Ok(image_buffer) => {
            let image_data = image_buffer.into_image().0;
            if !image_data.is_empty() {
                #[cfg(debug_assertions)]
                {
                    let video_format = handle
                        .video_format
                        .as_ref()
                        .expect("Unreachable: Video format not set");
                    assert_eq!(
                        image_data.len(),
                        ((video_format.width * video_format.height) as usize
                            * video_format.format.bytes_count()),
                        "Image data size does not match expected size"
                    );
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        image_data.as_ptr(),
                        buf as *mut u8,
                        image_data.len(),
                    );
                }
            }
            image_data.len() as i32
        }
        Err(_) => 0,
    }
}

pub unsafe fn func_read_audio<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    start: i32,
    length: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    free_leaked_memory();
    let handle = unsafe { &*(ih as *mut InternalInputHandle<T::InputHandle>) };
    match T::read_audio(plugin, &handle.handle, start, length) {
        Ok(audio_buffer) => {
            let audio_data = audio_buffer.into_audio().0;
            let len = audio_data.len();
            if len > 0 {
                #[cfg(debug_assertions)]
                {
                    let audio_format = handle
                        .audio_format
                        .as_ref()
                        .expect("Unreachable: Audio format not set");
                    assert_eq!(
                        len,
                        (audio_format.channels as usize
                            * length as usize
                            * audio_format.format.bytes_per_sample()),
                        "Audio data size does not match expected size"
                    );
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(audio_data.as_ptr(), buf as *mut u8, len);
                }
            }
            len as i32
        }
        Err(_) => 0,
    }
}

pub unsafe fn func_config<T: InputPlugin>(
    plugin: &T,
    hwnd: aviutl2_sys::input2::HWND,
    dll_hinst: aviutl2_sys::input2::HINSTANCE,
) -> bool {
    free_leaked_memory();
    let mut handle =
        raw_window_handle::Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
    handle.hinstance = Some(NonZeroIsize::new(dll_hinst as isize).unwrap());
    result_to_bool_with_dialog(plugin.config(handle))
}

#[macro_export]
macro_rules! register_input_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_input_plugin {
            use super::*;

            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

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
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_open(
                file: aviutl2::sys::input2::LPCWSTR,
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
        }
    };
}
