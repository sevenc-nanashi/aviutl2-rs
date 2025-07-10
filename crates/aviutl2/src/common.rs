pub use anyhow::Result as AnyResult;

pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

pub(crate) fn format_file_filters(file_filters: &[FileFilter]) -> String {
    let mut file_filter = String::new();
    for filter in file_filters {
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

    return file_filter;
}

pub(crate) fn load_large_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)) }
}

static WILL_FREE_ON_NEXT_CALL: std::sync::LazyLock<std::sync::Mutex<Vec<usize>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

pub(crate) fn leak_large_string(s: &str) -> *mut u16 {
    let mut will_free = WILL_FREE_ON_NEXT_CALL.lock().unwrap();
    let vec = s
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let ptr = vec.as_ptr() as *mut u16;
    will_free.push(ptr as usize);
    std::mem::forget(vec); // Prevent Rust from freeing the memory
    ptr
}

pub(crate) fn free_leaked_memory() {
    let mut will_free = WILL_FREE_ON_NEXT_CALL.lock().unwrap();
    for ptr in will_free.drain(..) {
        unsafe {
            let _ = Box::from_raw(ptr as *mut u16);
        }
    }
}

pub(crate) fn result_to_bool_with_dialog<T>(result: AnyResult<T>) -> bool {
    match result {
        Ok(_) => true,
        Err(e) => {
            alert_error(&e);
            false
        }
    }
}

pub(crate) fn alert_error(error: &anyhow::Error) {
    let _ = native_dialog::DialogBuilder::message()
        .set_title("エラー")
        .set_level(native_dialog::MessageLevel::Error)
        .set_text(&format!("エラーが発生しました: {}", error))
        .alert()
        .show();
}
