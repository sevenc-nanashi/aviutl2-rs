use super::{ImageFormat, InputPlugin};

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
    func_config: Option<
        extern "C" fn(aviutl2_sys::input2::HWND, aviutl2_sys::input2::HINSTANCE) -> bool,
    >,
) -> aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
    let table = plugin.plugin_info();
    let mut file_filter = String::new();
    for filter in table.file_filters {
        if !file_filter.is_empty() {
            file_filter.push('\x00');
        }
        file_filter.push_str(&filter.name);
        file_filter.push('\x00');
        file_filter.push_str(
            &filter
                .extensions
                .iter()
                .map(|ext| format!("*.{}", ext))
                .collect::<Vec<_>>()
                .join(";"),
        );
        file_filter.push('\x00');
    }

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
        func_config,
    };
}
pub fn func_open<T: InputPlugin>(
    plugin: &T,
    file: aviutl2_sys::input2::LPCWSTR,
) -> aviutl2_sys::input2::INPUT_HANDLE {
    free_leaked_memory();
    let mut path_vec = vec![];
    let pointer = file as *const u16;
    for i in 0.. {
        let c = unsafe { *pointer.add(i) };
        if c == 0 {
            break;
        }
        path_vec.push(c);
    }
    let path = String::from_utf16_lossy(&path_vec);
    match plugin.open(std::path::PathBuf::from(path)) {
        Some(handle) => {
            let boxed_handle: Box<T::InputHandle> = Box::new(handle);
            Box::into_raw(boxed_handle) as aviutl2_sys::input2::INPUT_HANDLE
        }
        None => std::ptr::null_mut(),
    }
}
pub fn func_close<T: InputPlugin>(plugin: &T, ih: aviutl2_sys::input2::INPUT_HANDLE) -> bool {
    free_leaked_memory();
    let handle = *unsafe { Box::from_raw(ih as *mut T::InputHandle) };
    T::close(plugin, handle)
}
pub fn func_info_get<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    iip: *mut aviutl2_sys::input2::INPUT_INFO,
) -> bool {
    free_leaked_memory();
    let handle = unsafe { &*(ih as *mut T::InputHandle) };

    match T::get_input_info(plugin, handle) {
        Ok(info) => match (info.video, info.audio) {
            (Some(video_info), None) => {
                let image_format = video_info.image_format.into_raw();
                let image_format = Box::new(image_format);
                let image_format_ptr = Box::into_raw(image_format);
                WILL_FREE_ON_NEXT_CALL
                    .lock()
                    .unwrap()
                    .push(image_format_ptr as usize);
                unsafe {
                    (*iip).flag = aviutl2_sys::input2::INPUT_INFO::FLAG_VIDEO;
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
                true
            }
            _ => todo!(),
        },
        Err(_) => false,
    }
}
pub fn func_read_video<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    frame: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    free_leaked_memory();
    let handle = unsafe { &*(ih as *mut T::InputHandle) };
    match T::read_video(plugin, handle, frame) {
        Ok(image_buffer) => {
            let image_data = image_buffer.0;
            let len = image_data.len();
            let format_info = plugin
                .get_input_info(handle)
                .unwrap()
                .video
                .unwrap()
                .image_format;
            let width = format_info.width as usize;
            let height = format_info.height as usize;
            debug_assert!(len % (width * 4) == 0, "Image data length mismatch");
            if len > 0 {
                unsafe {
                    for y in 0..height {
                        for x in 0..width {
                            let idx = (y * width + x) * 4;
                            let r = image_data[idx];
                            let g = image_data[idx + 1];
                            let b = image_data[idx + 2];
                            let a = image_data[idx + 3];
                            // bgrA
                            let pixel = (b as u32)
                                | (g as u32) << 8
                                | (r as u32) << 16
                                | ((a as u32) << 24);
                            let dest_ptr =
                                buf.add(((height - 1 - y as usize) * width + x) * 4) as *mut u32;
                            dest_ptr.write(pixel);
                        }
                    }
                }
            }
            len as i32
        }
        Err(_) => 0,
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
            let audio_data = audio_buffer.0;
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
        Err(_) => 0,
    }
}

pub fn func_config<T: InputPlugin>(
    _plugin: &T,
    _hwnd: aviutl2_sys::input2::HWND,
    _dll_hinst: aviutl2_sys::input2::HINSTANCE,
) -> bool {
    free_leaked_memory();
    // Placeholder for configuration function
    false
}

fn into_large_string(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn leak_large_string(s: &str) -> *mut u16 {
    let mut vec = into_large_string(s);
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec); // Prevent Rust from deallocating the memory
    ptr
}

#[macro_export]
macro_rules! register_input_plugin {
    ($struct:ident) => {
        mod __au2_register_plugin {
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
                    Some(func_config),
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
