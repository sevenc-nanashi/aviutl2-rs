use crate::{
    common::{format_file_filters, leak_large_string, load_large_string},
    input::{AudioFormat, ImageFormat, InputPlugin, IntoAudio, IntoImage},
};

use super::{alert_error, result_to_bool_with_dialog};

impl ImageFormat {
    fn into_raw(&self) -> aviutl2_sys::input2::BITMAPINFOHEADER {
        aviutl2_sys::input2::BITMAPINFOHEADER {
            biSize: std::mem::size_of::<aviutl2_sys::input2::BITMAPINFOHEADER>() as u32,
            biWidth: self.width as i32,
            biHeight: self.height as i32,
            biPlanes: 1,
            biBitCount: 32, // Assuming RGBA format
            biCompression: aviutl2_sys::input2::BI_RGB,
            biSizeImage: (self.width * self.height * 4) as u32, // 4 bytes per pixel for RGBA
            biXPelsPerMeter: 0,                                 // Not used
            biYPelsPerMeter: 0,                                 // Not used
            biClrUsed: 0,                                       // Not used
            biClrImportant: 0,                                  // Not used
        }
    }
}
impl AudioFormat {
    fn into_raw(&self) -> aviutl2_sys::input2::WAVEFORMATEX {
        aviutl2_sys::input2::WAVEFORMATEX {
            wFormatTag: aviutl2_sys::input2::WAVE_FORMAT_PCM as u16,
            nChannels: self.channels as u16,
            nSamplesPerSec: self.sample_rate as u32,
            nAvgBytesPerSec: (self.sample_rate * (self.channels as u32) * 4),
            nBlockAlign: (self.channels * 4) as u16, // 4 bytes per sample for float
            wBitsPerSample: 32,                      // Assuming float samples
            cbSize: 0,                               // No extra data
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
    width: u32,
    height: u32,
    handle: T,
}

pub fn create_table<T: InputPlugin>(
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
    let table = plugin.plugin_info();
    let file_filter = format_file_filters(&table.file_filters);

    return aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
        flag: table.input_type.to_bits(),
        name: leak_large_string(&table.name),
        filefilter: leak_large_string(&file_filter),
        information: leak_large_string(&table.information),
        func_open: Some(func_open),
        func_close: Some(func_close),
        func_info_get: Some(func_info_get),
        func_read_video: Some(func_read_video),
        func_read_audio: Some(func_read_audio),
        func_config: table.can_config.then_some(func_config),
    };
}
pub fn func_open<T: InputPlugin>(
    plugin: &T,
    file: aviutl2_sys::input2::LPCWSTR,
) -> aviutl2_sys::input2::INPUT_HANDLE {
    free_leaked_memory();
    let path = load_large_string(file);
    match plugin.open(std::path::PathBuf::from(path)) {
        Ok(handle) => {
            let boxed_handle: Box<InternalInputHandle<T::InputHandle>> =
                Box::new(InternalInputHandle {
                    width: 0,
                    height: 0,
                    handle,
                });
            Box::into_raw(boxed_handle) as aviutl2_sys::input2::INPUT_HANDLE
        }
        Err(e) => {
            alert_error(&e);
            std::ptr::null_mut()
        }
    }
}
pub fn func_close<T: InputPlugin>(plugin: &T, ih: aviutl2_sys::input2::INPUT_HANDLE) -> bool {
    free_leaked_memory();
    let handle = *unsafe { Box::from_raw(ih as *mut InternalInputHandle<T::InputHandle>) };
    result_to_bool_with_dialog(T::close(plugin, handle.handle))
}
pub fn func_info_get<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    iip: *mut aviutl2_sys::input2::INPUT_INFO,
) -> bool {
    free_leaked_memory();
    let handle = unsafe { &mut *(ih as *mut InternalInputHandle<T::InputHandle>) };

    match T::get_input_info(plugin, &handle.handle) {
        Ok(info) => {
            if let Some(video_info) = info.video {
                handle.width = video_info.image_format.width;
                handle.height = video_info.image_format.height;
                let image_format = video_info.image_format.into_raw();
                let image_format = Box::new(image_format);
                let image_format_ptr = Box::into_raw(image_format);
                WILL_FREE_ON_NEXT_CALL
                    .lock()
                    .unwrap()
                    .push(image_format_ptr as usize);
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_VIDEO;
                    (*iip).rate = video_info.fps;
                    (*iip).scale = video_info.scale;
                    (*iip).n = video_info.num_frames;
                    (*iip).format = image_format_ptr;
                    (*iip).format_size =
                        (4 * video_info.image_format.width * video_info.image_format.height) as i32;
                    (*iip).audio_n = 0;
                    (*iip).audio_format = std::ptr::null_mut();
                    (*iip).audio_format_size = 0;
                }
            }

            if let Some(audio_info) = info.audio {
                let audio_format = audio_info.audio_format.into_raw();
                let audio_format = Box::new(audio_format);
                let audio_format_ptr = Box::into_raw(audio_format);
                WILL_FREE_ON_NEXT_CALL
                    .lock()
                    .unwrap()
                    .push(audio_format_ptr as usize);
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_AUDIO;
                    (*iip).audio_n = audio_info.num_samples;
                    (*iip).audio_format = audio_format_ptr;
                    (*iip).audio_format_size =
                        std::mem::size_of_val(&audio_info.audio_format) as i32;
                }
            }

            if info.concurrent {
                unsafe {
                    (*iip).flag |= aviutl2_sys::input2::INPUT_INFO::FLAG_CONCURRENT;
                }
            }

            true
        }
        Err(e) => {
            alert_error(&e);
            false
        }
    }
}
pub fn func_read_video<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    frame: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    free_leaked_memory();
    let handle = unsafe { &*(ih as *mut InternalInputHandle<T::InputHandle>) };
    match T::read_video(plugin, &handle.handle, frame) {
        Ok(image_buffer) => {
            let width = handle.width;
            let height = handle.height;
            let image_data = image_buffer.into_image(width, height).0;
            if !image_data.is_empty() {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        image_data.as_ptr() as *const u8,
                        buf as *mut u8,
                        image_data.len(),
                    );
                }
            }
            image_data.len() as i32
        }
        Err(e) => {
            alert_error(&e);
            0
        }
    }
}

pub fn func_read_audio<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    start: i32,
    length: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    free_leaked_memory();
    let handle = unsafe { &*(ih as *mut T::InputHandle) };
    match T::read_audio(plugin, handle, start, length) {
        Ok(audio_buffer) => {
            let audio_data = audio_buffer.into_audio().0;
            let len = audio_data.len();
            if len > 0 {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        audio_data.as_ptr() as *const f32,
                        buf as *mut f32,
                        len,
                    );
                }
            }
            len as i32
        }
        Err(e) => {
            alert_error(&e);
            0
        }
    }
}

pub fn func_config<T: InputPlugin>(
    plugin: &T,
    hwnd: aviutl2_sys::input2::HWND,
    dll_hinst: aviutl2_sys::input2::HINSTANCE,
) -> bool {
    free_leaked_memory();
    result_to_bool_with_dialog(plugin.config(hwnd, dll_hinst))
}

#[macro_export]
macro_rules! register_input_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_input_plugin {
            use super::*;

            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

            #[unsafe(no_mangle)]
            extern "C" fn GetInputPluginTable() -> *mut aviutl2::sys::input2::INPUT_PLUGIN_TABLE {
                let table = $crate::input::__bridge::create_table::<$struct>(
                    &PLUGIN,
                    func_open,
                    func_close,
                    func_info_get,
                    func_read_video,
                    func_read_audio,
                    func_config,
                );
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
