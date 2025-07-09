use crate::common::format_file_filters;

use super::OutputPlugin;

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

pub fn create_table<T: OutputPlugin>(
    plugin: &T,
    func_output: extern "C" fn(aviutl2_sys::output2::OUTPUT_INFO) -> bool,
    func_config: Option<
        extern "C" fn(aviutl2_sys::output2::HWND, aviutl2_sys::output2::HINSTANCE) -> bool,
    >,
    func_get_config_text: Option<extern "C" fn() -> *mut u16>,
) -> aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    free_leaked_memory();
    let plugin_info = plugin.plugin_info();
    let name = leak_large_string(&plugin_info.name);
    let filefilter = format_file_filters(&plugin_info.file_filters);
    let filefilter = leak_large_string(&filefilter);
    let information = leak_large_string(&plugin_info.information);

    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
        flag: plugin_info.input_type.to_bits(),
        name,
        filefilter,
        information,
        func_output: Some(func_output),
        func_config,
        func_get_config_text,
    }
}

fn func_output<T: OutputPlugin>(
    plugin: &T,
    oip: aviutl2_sys::output2::OUTPUT_INFO,
) -> bool {
    plugin.output(output_info).is_ok()
}

fn func_config<T: OutputPlugin>(
    plugin: &T,
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    plugin.config(hwnd, dll_hinst).is_ok()
}

fn func_get_config_text<T: OutputPlugin>(plugin: &T) -> *mut u16 {
    let text = plugin.get_config_text();
    if text.is_empty() {
        std::ptr::null_mut()
    } else {
        leak_large_string(&text)
    }
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
macro_rules! register_output_plugin {
    ($struct:ident) => {
        mod __au2_register_plugin {
            use super::*;

            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

            #[unsafe(no_mangle)]
            extern "C" fn GetInputPluginTable() -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE {
                let table = $crate::output::__bridge::create_table::<$struct>(
                    &*PLUGIN,
                    func_output,
                    Some(func_config),
                    Some(func_get_config_text),
                );
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_output(oip: aviutl2::sys::output2::OUTPUT_INFO) -> bool {
                unsafe { $crate::output::__bridge::func_output(&*PLUGIN, ih) }
            }

            extern "C" fn func_config(
                hwnd: aviutl2::sys::output2::HWND,
                dll_hinst: aviutl2::sys::output2::HINSTANCE,
            ) -> bool {
                unsafe { $crate::output::__bridge::func_config(&*PLUGIN, hwnd, dll_hinst) }
            }

            extern "C" fn func_get_config_text() -> *mut u16 {
                unsafe { $crate::output::__bridge::func_get_config_text(&*PLUGIN) }
            }
        }
    };
}
