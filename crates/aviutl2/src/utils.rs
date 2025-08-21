/// OutputDebugStringのラッパー関数。
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::utils::debug_print_impl(&message);
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

/// Vec<T>を2次元配列として捉え、上下に反転させる関数。
///
/// # Panics
///
/// `data.len()` が `width * height` でない場合にパニックします。
pub fn flip_vertical<T>(data: &mut [T], width: usize, height: usize) {
    assert!(data.len() == width * height);
    let row_size = width;
    for y in 0..(height / 2) {
        let top_row_start = y * row_size;
        let bottom_row_start = (height - 1 - y) * row_size;
        for x in 0..width {
            data.swap(top_row_start + x, bottom_row_start + x);
        }
    }
}

/// Vec<(u8, u8, u8)>をBGRの配列として捉え、RGBの配列に変換する関数。
pub fn bgr_to_rgb(data: &mut [(u8, u8, u8)]) {
    for pixel in data.iter_mut() {
        let (b, g, r) = *pixel;
        *pixel = (r, g, b);
    }
}

/// Vec<(u8, u8, u8, u8)>をBGRAの配列として捉え、RGBAの配列に変換する関数。
pub fn bgra_to_rgba(data: &mut [(u8, u8, u8, u8)]) {
    for pixel in data.iter_mut() {
        let (b, g, r, a) = *pixel;
        *pixel = (r, g, b, a);
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
#[doc(inline)]
#[cfg(feature = "env_logger")]
pub use ods_logger::debug_logger_target;
