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

/// bitflagを簡単に初期化するためのマクロ。
///
/// # Example
///
/// ```rust
/// # use aviutl2::bitflag;
///
/// let flag = bitflag!(
///     aviutl2::filter::FilterPluginFlags {
///         video: true,
///     }
/// );
///
/// assert!(flag.video);
/// assert_eq!(flag.to_bits(), aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_VIDEO);
/// ```
#[macro_export]
macro_rules! bitflag {
    ($ty:ty { $($name:ident : $bit:expr),* $(,)? }) => {
        {
            let mut value: $ty = ::std::default::Default::default();
            $(
                value.$name = $bit;
            )*
            value
        }
    }
}

pub(crate) fn catch_unwind_with_panic_info<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(err) => {
            if let Some(s) = err.downcast_ref::<&str>() {
                Err(s.to_string())
            } else if let Some(s) = err.downcast_ref::<String>() {
                Err(s.clone())
            } else {
                Err("<unknown panic".to_string())
            }
        }
    }
}

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
