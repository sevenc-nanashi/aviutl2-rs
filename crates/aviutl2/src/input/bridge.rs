
use super::InputPlugin;

pub fn create_table<T: InputPlugin>(
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
    return aviutl2_sys::input2::INPUT_PLUGIN_TABLE {
        flag: T::PLUGIN_TYPE.to_bits(),
        name: leak_large_string(T::PLUGIN_NAME),
        filefilter: leak_large_string(
            &T::PLUGIN_FILE_FILTER
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>()
                .join(";"),
        ),
        information: leak_large_string(T::PLUGIN_INFORMATION),
        func_open: Some(func_open),
        func_close: Some(func_close),
        func_info_get: Some(func_info_get),
        func_read_video: Some(func_read_video),
        func_read_audio: Some(func_read_audio),
        func_config,
    };
}
fn func_open<T: InputPlugin>(
    plugin: &T,
    file: aviutl2_sys::input2::LPCWSTR,
) -> aviutl2_sys::input2::INPUT_HANDLE {
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
            let boxed_handle: Box<dyn std::any::Any + Send + Sync> = Box::new(handle);
            Box::into_raw(boxed_handle) as aviutl2_sys::input2::INPUT_HANDLE
        }
        None => std::ptr::null_mut(),
    }
}
fn func_close<T: InputPlugin>(plugin: &T, ih: aviutl2_sys::input2::INPUT_HANDLE) -> bool {
    let boxed_handle = unsafe { Box::from_raw(ih as *mut dyn std::any::Any) };
    let mut handle = boxed_handle.downcast::<T::InputHandle>().unwrap();
    T::close(plugin, &mut handle)
}
fn func_info_get<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    iip: *mut aviutl2_sys::input2::INPUT_INFO,
) -> bool {
    let boxed_handle = unsafe { Box::from_raw(ih as *mut dyn std::any::Any) };
    let handle = boxed_handle.downcast_ref::<T::InputHandle>().unwrap();
    match T::get_info(plugin, handle) {
        Ok(info) => {
            unsafe {
                (*iip).flag = info.flag.to_bits();
                (*iip).rate = info.fps;
                (*iip).scale = info.scale;
                (*iip).n = info.num_frames;
                (*iip).format = std::ptr::null_mut(); // Placeholder for format
                (*iip).format_size = 0; // Placeholder for format size
                (*iip).audio_n = info.num_samples;
                (*iip).audio_format = std::ptr::null_mut(); // Placeholder for audio format
                (*iip).audio_format_size = 0; // Placeholder for audio format size
            }
            true
        }
        Err(_) => false,
    }
}
fn func_read_video<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    frame: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    let boxed_handle = unsafe { Box::from_raw(ih as *mut dyn std::any::Any) };
    let handle = boxed_handle.downcast_ref::<T::InputHandle>().unwrap();
    match T::read_video(plugin, handle, frame) {
        Ok(image_buffer) => {
            let image_data = image_buffer.0;
            let len = image_data.len();
            if len > 0 {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        image_data.as_ptr() as *const u8,
                        buf as *mut u8,
                        len,
                    );
                }
            }
            len as i32
        }
        Err(_) => -1,
    }
}

fn func_read_audio<T: InputPlugin>(
    plugin: &T,
    ih: aviutl2_sys::input2::INPUT_HANDLE,
    start: i32,
    length: i32,
    buf: *mut std::ffi::c_void,
) -> i32 {
    let boxed_handle = unsafe { Box::from_raw(ih as *mut dyn std::any::Any) };
    let handle = boxed_handle.downcast_ref::<T::InputHandle>().unwrap();
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
        Err(_) => -1,
    }
}

fn func_config<T: InputPlugin>(
    _plugin: &T,
    _hwnd: aviutl2_sys::input2::HWND,
    _dll_hinst: aviutl2_sys::input2::HINSTANCE,
) -> bool {
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
            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

            #[no_mangle]
            extern "C" fn GetInputPluginTable() -> *mut aviutl2::sys::INPUT_PLUGIN_TABLE {
                let table = $crate::create_table::<$struct>(
                    func_open::<$struct>,
                    func_close::<$struct>,
                    func_info_get::<$struct>,
                    func_read_video::<$struct>,
                    func_read_audio::<$struct>,
                    Some(func_config::<$struct>),
                );
                Box::into_raw(Box::new(table))
            }

            #[no_mangle]
            extern "C" fn func_open(
                file: aviutl2_sys::input2::LPCWSTR,
            ) -> aviutl2_sys::input2::INPUT_HANDLE {
                unsafe { $crate::__bridge::func_open(&PLUGIN.get(), file) }
            }

            #[no_mangle]
            extern "C" fn func_close(
                ih: aviutl2_sys::input2::INPUT_HANDLE,
            ) -> bool {
                unsafe { $crate::__bridge::func_close(&PLUGIN.get(), ih) }
            }

            #[no_mangle]
            extern "C" fn func_info_get(
                ih: aviutl2_sys::input2::INPUT_HANDLE,
                iip: *mut aviutl2_sys::input2::INPUT_INFO,
            ) -> bool {
                unsafe { $crate::__bridge::func_info_get(&PLUGIN.get(), ih, iip) }
            }

            #[no_mangle]
            extern "C" fn func_read_video(
                ih: aviutl2_sys::input2::INPUT_HANDLE,
                frame: i32,
                buf: *mut std::ffi::c_void,
            ) -> i32 {
                unsafe { $crate::__bridge::func_read_video(&PLUGIN.get(), ih, frame, buf) }
            }

            #[no_mangle]
            extern "C" fn func_read_audio(
                ih: aviutl2_sys::input2::INPUT_HANDLE,
                start: i32,
                length: i32,
                buf: *mut std::ffi::c_void,
            ) -> i32 {
                unsafe { $crate::__bridge::func_read_audio(&PLUGIN.get(), ih, start, length, buf) }
            }

            #[no_mangle]
            extern "C" fn func_config(
                hwnd: aviutl2_sys::input2::HWND,
                dll_hinst: aviutl2_sys::input2::HINSTANCE,
            ) -> bool {
                unsafe { $crate::__bridge::func_config(&PLUGIN.get(), hwnd, dll_hinst) }
            }
        }
    };
}
