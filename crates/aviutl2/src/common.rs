pub use anyhow::Result as AnyResult;

/// ファイル選択ダイアログのフィルタを表す構造体。
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// フィルタの名前。
    pub name: String,
    /// フィルタが適用される拡張子のリスト。
    pub extensions: Vec<String>,
}

pub(crate) fn format_file_filters(file_filters: &[FileFilter]) -> String {
    let mut file_filter = String::new();
    for filter in file_filters {
        if !file_filter.is_empty() {
            file_filter.push('\x00');
        }
        let display = format!(
            "{} ({})",
            filter.name,
            filter
                .extensions
                .iter()
                .map(|ext| {
                    if ext.is_empty() {
                        "*".to_string()
                    } else {
                        ext.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", "),
        );
        file_filter.push_str(&display);
        file_filter.push('\x00');
        file_filter.push_str(
            &filter
                .extensions
                .iter()
                .map(|ext| {
                    if ext.is_empty() {
                        "*".to_string()
                    } else {
                        format!("*.{ext}")
                    }
                })
                .collect::<Vec<_>>()
                .join(";"),
        );
        file_filter.push('\x00');
    }

    file_filter
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
        .set_text(format!("エラーが発生しました: {error}"))
        .alert()
        .show();
}

/// OutputDebugStringのラッパー関数。
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::debug_print_impl(&message);
    };
}

/// OutputDebugStringに出力する[`dbg!`]マクロ。
///
/// # See Also
/// <https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/macros.rs#L352>
#[macro_export]
macro_rules! odbg {
    () => {
        $crate::debug_print!("[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!());
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::debug_print!("[{}:{}:{}] {} = {:#?}",
                    ::std::file!(),
                    ::std::line!(),
                    ::std::column!(),
                    ::std::stringify!($val),
                    &&tmp as &dyn ::std::fmt::Debug,
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::odbg!($val)),+,)
    };
}

// pub(crate) fn result_to_bool_with_debug_print<T>(result: AnyResult<T>) -> bool {
//     match result {
//         Ok(_) => true,
//         Err(e) => {
//             debug_print!("Error: {e}");
//             false
//         }
//     }
// }

#[doc(hidden)]
pub fn debug_print_impl(message: &str) {
    let mut cstr = message.encode_utf16().collect::<Vec<u16>>();
    cstr.push(0); // Null-terminate the string
    unsafe {
        let ptr = cstr.as_ptr();
        windows::Win32::System::Diagnostics::Debug::OutputDebugStringW(
            windows::core::PCWSTR::from_raw(ptr),
        );
    }
}

#[cfg(feature = "env_logger")]
mod ods_logger {
    /// [`env_logger::fmt::Target`]の実装。
    /// OutputDebugStringを使用してログを出力します。
    ///
    /// # Example
    ///
    /// ```rust
    /// use aviutl2_rs::common::debug_logger_target;
    ///
    /// env_logger::Builder::new()
    ///     .parse_filters("info")
    ///     .target(debug_logger_target())
    ///     .init();
    /// ```
    pub fn debug_logger_target() -> env_logger::fmt::Target {
        let write_target = OdsWriter::new();
        env_logger::fmt::Target::Pipe(Box::new(write_target))
    }

    struct OdsWriter {
        buffer: Vec<u8>,
    }

    impl OdsWriter {
        pub fn new() -> Self {
            Self { buffer: Vec::new() }
        }
    }

    impl std::io::Write for OdsWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
                let line = &self.buffer[..=pos];
                if let Ok(line_str) = std::str::from_utf8(line) {
                    super::debug_print_impl(line_str);
                }
                self.buffer.drain(..=pos);
            }
            Ok(())
        }
    }
}
pub use ods_logger::debug_logger_target;
