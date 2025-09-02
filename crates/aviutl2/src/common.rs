pub use anyhow::Result as AnyResult;
use zerocopy::{Immutable, IntoBytes};

pub use half::{self, f16};
pub use num_rational::{self, Rational32};
pub use raw_window_handle::{self, Win32WindowHandle};

/// ファイル選択ダイアログのフィルタを表す構造体。
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// フィルタの名前。
    pub name: String,
    /// フィルタが適用される拡張子のリスト。
    pub extensions: Vec<String>,
}

/// YC48のピクセルフォーマットを表す構造体。
///
/// # See Also
/// <https://makiuchi-d.github.io/mksoft/doc/aviutlyc.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, Immutable)]
#[repr(C)]
pub struct Yc48 {
    /// Y成分の値。
    /// 0から4096までの値を取ります。
    pub y: i16,
    /// Cb成分の値。
    /// -2048から2048までの値を取ります。
    pub cb: i16,
    /// Cr成分の値。
    /// -2048から2048までの値を取ります。
    pub cr: i16,
}
impl Yc48 {
    /// YUV 4:2:2（YUY2）からYC48に変換します。
    pub fn from_yuy2(yuy2: (u8, u8, u8, u8)) -> (Self, Self) {
        let (y1, u, y2, v) = yuy2;
        let ny1 = ((y1 as u16 * 1197) >> 6) - 299;
        let ny2 = ((y2 as u16 * 1197) >> 6) - 299;
        let ncb = ((u as u16 - 128) * 4681 + 164) >> 8;
        let ncr = ((v as u16 - 128) * 4681 + 164) >> 8;
        (
            Self {
                y: ny1 as i16,
                cb: ncb as i16,
                cr: ncr as i16,
            },
            Self {
                y: ((ny1 + ny2) >> 1) as i16,
                cb: ncb as i16,
                cr: ncr as i16,
            },
        )
    }

    /// YC48からYUV 4:2:2（YUY2）に変換します。
    pub fn to_yuy2(self, other: Yc48) -> (u8, u8, u8, u8) {
        let y1 = ((self.y * 219 + 383) >> 12) + 16;
        let y2 = ((other.y * 219 + 383) >> 12) + 16;
        let u = (((self.cb + 2048) * 7 + 66) >> 7) + 16;
        let v = (((self.cr + 2048) * 7 + 66) >> 7) + 16;
        let y1 = y1.min(255) as u8;
        let y2 = y2.min(255) as u8;
        let u = u.min(255) as u8;
        let v = v.min(255) as u8;
        (y1, u, y2, v)
    }

    /// RGBからYC48に変換します。
    pub fn from_rgb(self, rgb: (u8, u8, u8)) -> Self {
        let (r, g, b) = rgb;
        let r = i16::from(r);
        let g = i16::from(g);
        let b = i16::from(b);
        let y = ((4918 * r + 354) >> 10) + ((9655 * g + 585) >> 10) + ((1875 * b + 523) >> 10);
        let cb = ((-2775 * r + 240) >> 10) + ((-5449 * g + 515) >> 10) + ((8224 * b + 256) >> 10);
        let cr = ((8224 * r + 256) >> 10) + ((-6887 * g + 110) >> 10) + ((-1337 * b + 646) >> 10);

        Yc48 { y, cb, cr }
    }

    /// YC48からRGBに変換します。
    pub fn to_rgb(self) -> (u8, u8, u8) {
        let y = self.y as i32;
        let cr = self.cr as i32;
        let cb = self.cr as i32;
        let r = (255 * y + ((((22881 * cr) >> 16) + 3) << 10)) >> 12;
        let g = (255 * y + ((((-5616 * cb) >> 16) + ((-11655 * cr) >> 16) + 3) << 10)) >> 12;
        let b = (255 * y + ((((28919 * cb) >> 16) + 3) << 10)) >> 12;

        let r = r.min(255) as u8;
        let g = g.min(255) as u8;
        let b = b.min(255) as u8;
        (r, g, b)
    }
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

pub(crate) enum LeakedPtrType {
    WideString,
    BitmapInfoHeader,
    WaveFormatEx,
}

pub(crate) struct LeakManager {
    ptrs: std::sync::Mutex<Vec<(LeakedPtrType, usize)>>,
}

pub(crate) trait IntoLeakedPtr {
    fn into_leaked_ptr(self) -> (LeakedPtrType, usize);
}

#[duplicate::duplicate_item(
    OriginalType                             EnumValue;
    [aviutl2_sys::input2::WAVEFORMATEX]      [WaveFormatEx];
    [aviutl2_sys::output2::BITMAPINFOHEADER] [BitmapInfoHeader];
)]
impl IntoLeakedPtr for OriginalType {
    fn into_leaked_ptr(self) -> (LeakedPtrType, usize) {
        let boxed = Box::new(self);
        let ptr = Box::into_raw(boxed) as usize;
        (LeakedPtrType::EnumValue, ptr)
    }
}

impl LeakManager {
    pub fn new() -> Self {
        Self {
            ptrs: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn leak<T: IntoLeakedPtr>(&self, value: T) -> *const T {
        let mut ptrs = self.ptrs.lock().unwrap();
        let leaked = value.into_leaked_ptr();
        let ptr = leaked.1;
        ptrs.push(leaked);
        ptr as *const T
    }

    pub fn leak_as_wide_string(&self, s: &str) -> *const u16 {
        let mut wide: Vec<u16> = s.encode_utf16().collect();
        wide.push(0); // Null-terminate the string
        let boxed = wide.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u16 as usize;
        let mut ptrs = self.ptrs.lock().unwrap();
        ptrs.push((LeakedPtrType::WideString, ptr));
        ptr as *const u16
    }

    pub fn free_leaked_memory(&self) {
        let mut ptrs = self.ptrs.lock().unwrap();
        for (ptr_type, ptr) in ptrs.drain(..) {
            unsafe {
                match ptr_type {
                    LeakedPtrType::WideString => {
                        let _ = Box::from_raw(ptr as *mut u16);
                    }
                    LeakedPtrType::BitmapInfoHeader => {
                        let _ = Box::from_raw(ptr as *mut aviutl2_sys::output2::BITMAPINFOHEADER);
                    }
                    LeakedPtrType::WaveFormatEx => {
                        let _ = Box::from_raw(ptr as *mut aviutl2_sys::input2::WAVEFORMATEX);
                    }
                }
            }
        }
    }
}

pub(crate) fn load_wide_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)) }
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
