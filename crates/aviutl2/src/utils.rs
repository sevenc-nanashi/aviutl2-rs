/// OutputDebugStringに出力する[`dbg!`]マクロ。
///
/// # See Also
/// <https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/macros.rs#L352>
#[macro_export]
macro_rules! odbg {
    () => {
        $crate::oprintln!("[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!());
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::oprintln!("[{}:{}:{}] {} = {:#?}",
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

/// OutputDebugStringに出力する[`println!`]マクロ。
#[macro_export]
macro_rules! oprintln {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::utils::debug_println_impl(&message);
    };
}

#[doc(hidden)]
pub fn debug_println_impl(message: &str) {
    let mut cstr = format!("{message}\n").encode_utf16().collect::<Vec<u16>>();
    cstr.push(0); // Null-terminate the string
    unsafe {
        let ptr = cstr.as_ptr();
        windows::Win32::System::Diagnostics::Debug::OutputDebugStringW(
            windows::core::PCWSTR::from_raw(ptr),
        );
    }
}

/// `Vec<T>`を2次元配列として捉え、上下に反転させる関数。
///
/// # Panics
///
/// `data.len()` が `width * height` でない場合にパニックします。
pub fn flip_vertical<T>(data: &mut [T], width: usize, height: usize) {
    assert!(data.len() == width * height);
    let data_ptr = data.as_mut_ptr();
    let row_size = width;
    unsafe {
        for y in 0..(height / 2) {
            let top_row_start = data_ptr.add(y * row_size);
            let bottom_row_start = data_ptr.add((height - 1 - y) * row_size);
            std::ptr::swap_nonoverlapping(top_row_start, bottom_row_start, row_size);
        }
    }
}

/// Vec<(u8, u8, u8)>をBGRの配列として捉え、RGBの配列に変換する関数。
/// エイリアスとして [`rgb_to_bgr`] も提供されます。
pub fn bgr_to_rgb(data: &mut [(u8, u8, u8)]) {
    for pixel in data.iter_mut() {
        let (b, g, r) = *pixel;
        *pixel = (r, g, b);
    }
}

/// [`bgr_to_rgb`]のエイリアス。
#[inline]
pub fn rgb_to_bgr(data: &mut [(u8, u8, u8)]) {
    bgr_to_rgb(data);
}

/// `Vec<u8>`をBGRの配列として捉え、RGBの配列に変換する関数。
/// エイリアスとして [`rgb_to_bgr_bytes`] も提供されます。
///
/// # Panics
///
/// `data.len()` が3の倍数でない場合にパニックします。
pub fn bgr_to_rgb_bytes(data: &mut [u8]) {
    assert!(data.len().is_multiple_of(3));
    for chunk in data.chunks_exact_mut(3) {
        chunk.swap(0, 2);
    }
}

/// [`bgr_to_rgb_bytes`]のエイリアス。
#[inline]
pub fn rgb_to_bgr_bytes(data: &mut [u8]) {
    bgr_to_rgb_bytes(data);
}

/// `Vec<(u8, u8, u8, u8)>`をRGBAの配列として捉え、BGRAの配列に変換する関数。
/// エイリアスとして [`bgra_to_rgba`] も提供されます。
pub fn rgba_to_bgra(data: &mut [(u8, u8, u8, u8)]) {
    for pixel in data.iter_mut() {
        let (b, g, r, a) = *pixel;
        *pixel = (r, g, b, a);
    }
}

/// [`rgba_to_bgra`]のエイリアス。
#[inline]
pub fn bgra_to_rgba(data: &mut [(u8, u8, u8, u8)]) {
    rgba_to_bgra(data);
}

/// `Vec<u8>`をRGBAの配列として捉え、BGRAの配列に変換する関数。
/// エイリアスとして [`bgra_to_rgba_bytes`] も提供されます。
///
/// # Panics
///
/// `data.len()` が4の倍数でない場合にパニックします。
pub fn rgba_to_bgra_bytes(data: &mut [u8]) {
    assert!(data.len().is_multiple_of(4));
    for chunk in data.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
}

/// [`rgba_to_bgra_bytes`]のエイリアス。
#[inline]
pub fn bgra_to_rgba_bytes(data: &mut [u8]) {
    rgba_to_bgra_bytes(data);
}

#[cfg(feature = "env_logger")]
mod ods_logger {
    /// [`env_logger::fmt::Target`]の実装。
    /// OutputDebugStringを使用してログを出力します。
    ///
    /// # Example
    ///
    /// ```rust
    /// use aviutl2::utils::debug_logger_target;
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
        fn new() -> Self {
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
                    super::debug_println_impl(line_str);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flip_vertical() {
        let mut data = vec![
            1, 2, 3, 4, 5, 6, // Row 0
            7, 8, 9, 10, 11, 12, // Row 1
            13, 14, 15, 16, 17, 18, // Row 2
        ];
        flip_vertical(&mut data, 6, 3);
        assert_eq!(
            data,
            vec![
                13, 14, 15, 16, 17, 18, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6
            ]
        );
    }

    #[test]
    fn test_bgr_to_rgb() {
        let mut data = vec![(0, 0, 255), (0, 255, 0), (255, 0, 0)];
        bgr_to_rgb(&mut data);
        assert_eq!(data, vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)]);
    }

    #[test]
    fn test_rgb_to_bgr() {
        let mut data = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        rgb_to_bgr(&mut data);
        assert_eq!(data, vec![(0, 0, 255), (0, 255, 0), (255, 0, 0)]);
    }

    #[test]
    fn test_bgr_to_rgb_bytes() {
        let mut data = vec![0, 0, 255, 0, 255, 0, 255, 0, 0];
        bgr_to_rgb_bytes(&mut data);
        assert_eq!(data, vec![255, 0, 0, 0, 255, 0, 0, 0, 255]);
    }

    #[test]
    fn test_rgb_to_bgr_bytes() {
        let mut data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        rgb_to_bgr_bytes(&mut data);
        assert_eq!(data, vec![0, 0, 255, 0, 255, 0, 255, 0, 0]);
    }

    #[test]
    fn test_rgba_to_bgra() {
        let mut data = vec![(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255)];
        rgba_to_bgra(&mut data);
        assert_eq!(
            data,
            vec![(0, 0, 255, 255), (0, 255, 0, 255), (255, 0, 0, 255)]
        );
    }

    #[test]
    fn test_bgra_to_rgba() {
        let mut data = vec![(0, 0, 255, 255), (0, 255, 0, 255), (255, 0, 0, 255)];
        bgra_to_rgba(&mut data);
        assert_eq!(
            data,
            vec![(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255)]
        );
    }

    #[test]
    fn test_rgba_to_bgra_bytes() {
        let mut data = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
        rgba_to_bgra_bytes(&mut data);
        assert_eq!(data, vec![0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255]);
    }

    #[test]
    fn test_bgra_to_rgba_bytes() {
        let mut data = vec![0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255];
        bgra_to_rgba_bytes(&mut data);
        assert_eq!(data, vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]);
    }
}
