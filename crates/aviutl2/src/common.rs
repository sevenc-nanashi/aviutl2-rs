pub use anyhow::Result as AnyResult;
use zerocopy::{Immutable, IntoBytes};

pub use half::{self, f16};
pub use num_rational::{self, Rational32};
pub use raw_window_handle::{self, Win32WindowHandle};

/// AviUtl2の情報。
#[derive(Debug, Clone)]
pub struct AviUtl2Info {
    /// AviUtl2のバージョン。
    pub version: AviUtl2Version,
}

/// 対応する最小のAviUtl2バージョン。
pub const MINIMUM_AVIUTL2_VERSION: AviUtl2Version = AviUtl2Version(2003001);

/// AviUtl2のバージョンがサポート範囲かを確認します。
pub fn ensure_minimum_aviutl2_version(version: AviUtl2Version) -> AnyResult<()> {
    anyhow::ensure!(
        version >= MINIMUM_AVIUTL2_VERSION,
        "AviUtl2 version {version} is not supported. {MINIMUM_AVIUTL2_VERSION} or higher is required.",
    );
    Ok(())
}

/// 実際に登録されるエフェクト名を返す。
///
/// # Note
///
/// `cfg!(debug_assertions)`が`true`の場合、エフェクト名の終端に` (Debug)`が追加されます。
pub fn registered_effect_name(base_name: &str) -> String {
    if cfg!(debug_assertions) {
        format!("{base_name} (Debug)")
    } else {
        base_name.to_string()
    }
}

/// AviUtl2のバージョン。
///
/// # Note
///
/// バージョン番号の形式は公式に定義されていないため、暫定的に以下のように解釈しています。
/// ```text
/// 2001500
/// || | |
/// || | +-- ビルドバージョン
/// || +---- ベータバージョン
/// |+------ マイナーバージョン
/// +------- メジャーバージョン
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AviUtl2Version(pub u32);
impl From<u32> for AviUtl2Version {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AviUtl2Version> for u32 {
    fn from(value: AviUtl2Version) -> Self {
        value.0
    }
}
impl AviUtl2Version {
    /// 新しいバージョンを作成します。
    ///
    /// # Panics
    ///
    /// `minor`、`patch`、`build`がそれぞれ100以上の場合にパニックします。
    pub fn new(major: u8, minor: u8, patch: u8, build: u8) -> Self {
        assert!(minor < 100, "minor version must be less than 100");
        assert!(patch < 100, "patch version must be less than 100");
        assert!(build < 100, "build version must be less than 100");
        Self(
            (major as u32) * 1_000_000
                + (minor as u32) * 10_000
                + (patch as u32) * 100
                + (build as u32),
        )
    }

    /// メジャーバージョンを取得します。
    pub fn major(self) -> u32 {
        self.0 / 1000000
    }

    /// マイナーバージョンを取得します。
    pub fn minor(self) -> u32 {
        (self.0 / 10000) % 100
    }

    /// ベータバージョンを取得します。
    pub fn patch(self) -> u32 {
        (self.0 / 100) % 100
    }

    /// ビルドバージョンを取得します。
    pub fn build(self) -> u32 {
        self.0 % 100
    }
}
impl std::fmt::Display for AviUtl2Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let build_str = if self.build() > 0 {
            ('a'..='z')
                .nth((self.build() - 1) as usize)
                .map(|c| c.to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        write!(
            f,
            "{}.{:0>2}beta{}{}",
            self.major(),
            self.minor(),
            self.patch(),
            build_str
        )
    }
}
#[cfg(feature = "aviutl2-alias")]
impl aviutl2_alias::FromTableValue for AviUtl2Version {
    type Err = std::num::ParseIntError;

    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        let v: u32 = value.parse()?;
        Ok(Self::from(v))
    }
}

/// ファイル選択ダイアログのフィルタを表す構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    /// フィルタの名前。
    pub name: String,
    /// フィルタが適用される拡張子のリスト。
    pub extensions: Vec<String>,
}

/// [`Vec<FileFilter>`]を簡単に作成するためのマクロ。
///
/// # Example
///
/// ```rust
/// let filters = aviutl2::file_filters! {
///     "Image Files" => ["png", "jpg"],
///     "All Files" => []
/// };
/// ```
#[macro_export]
macro_rules! file_filters {
    ($($name:expr => [$($ext:expr),* $(,)?] ),* $(,)?) => {
        vec![
            $(
                $crate::FileFilter {
                    name: $name.to_string(),
                    extensions: vec![$($ext.to_string()),*],
                }
            ),*
        ]
    };
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
        let display = format!(
            "{} ({})",
            filter.name,
            if filter.extensions.is_empty() {
                "*".to_string()
            } else {
                filter
                    .extensions
                    .iter()
                    .map(|ext| format!(".{ext}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
        file_filter.push_str(&display);
        file_filter.push('\x00');
        if filter.extensions.is_empty() {
            file_filter.push('*');
        } else {
            file_filter.push_str(
                &filter
                    .extensions
                    .iter()
                    .map(|ext| format!("*.{ext}"))
                    .collect::<Vec<_>>()
                    .join(";"),
            );
        }
        file_filter.push('\x00');
    }

    file_filter
}

pub(crate) enum LeakType {
    WideString,
    ValueVector { len: usize, name: String },
    Null,
    Other(String),
}

pub(crate) struct LeakManager {
    ptrs: std::sync::Mutex<Vec<(LeakType, usize)>>,
}

pub(crate) trait IntoLeakedPtr {
    fn into_leaked_ptr(self) -> (LeakType, usize);
}
pub(crate) trait LeakableValue {}

impl LeakManager {
    pub fn new() -> Self {
        Self {
            ptrs: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn leak<T: IntoLeakedPtr>(&self, value: T) -> *const T {
        log::debug!("Leaking memory for type {}", std::any::type_name::<T>());
        let mut ptrs = self.ptrs.lock().unwrap();
        let leaked = value.into_leaked_ptr();
        let ptr = leaked.1;
        ptrs.push(leaked);
        ptr as *const T
    }

    pub fn leak_as_wide_string(&self, s: &str) -> *const u16 {
        log::debug!("Leaking wide string: {}", s);
        let mut wide: Vec<u16> = s.encode_utf16().collect();
        wide.push(0); // Null-terminate the string
        let boxed = wide.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut u16 as usize;
        let mut ptrs = self.ptrs.lock().unwrap();
        ptrs.push((LeakType::WideString, ptr));
        ptr as *const u16
    }

    // pub fn leak_ptr_vec<T: IntoLeakedPtr>(&self, vec: Vec<T>) -> *const *const T {
    //     log::debug!("Leaking vector of type {}", std::any::type_name::<T>());
    //     let mut raw_ptrs = Vec::with_capacity(vec.len() + 1);
    //     for item in vec {
    //         let leaked = item.into_leaked_ptr();
    //         let ptr = leaked.1;
    //         raw_ptrs.push(ptr);
    //         let mut ptrs = self.ptrs.lock().unwrap();
    //         ptrs.push(leaked);
    //     }
    //     self.leak_value_vec(raw_ptrs) as _
    // }

    pub fn leak_value_vec<T: LeakableValue>(&self, vec: Vec<T>) -> *const T {
        log::debug!(
            "Leaking value vector of type {}",
            std::any::type_name::<T>()
        );
        let len = vec.len();
        let boxed = vec.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut T as usize;
        let mut ptrs = self.ptrs.lock().unwrap();
        ptrs.push((
            LeakType::ValueVector {
                len,
                name: std::any::type_name::<T>().to_string(),
            },
            ptr,
        ));
        ptr as *const T
    }

    pub fn free_leaked_memory(&self) {
        let mut ptrs = self.ptrs.lock().unwrap();
        while let Some((ptr_type, ptr)) = ptrs.pop() {
            match ptr_type {
                LeakType::WideString => unsafe {
                    let _ = Box::from_raw(ptr as *mut u16);
                },
                LeakType::ValueVector { len, name } => {
                    Self::free_leaked_memory_leakable_value(&name, ptr, len);
                }
                LeakType::Null => {
                    assert!(ptr == 0);
                }
                LeakType::Other(ref type_name) => {
                    Self::free_leaked_memory_other_ptr(type_name, ptr);
                }
            }
        }
    }
}
macro_rules! impl_leak_ptr {
    ($($t:ty),* $(,)?) => {
        $(
            impl IntoLeakedPtr for $t {
                fn into_leaked_ptr(self) -> (LeakType, usize) {
                    let boxed = Box::new(self);
                    let ptr = Box::into_raw(boxed) as usize;
                    (LeakType::Other(std::any::type_name::<$t>().to_string()), ptr)
                }
            }
        )*

        impl LeakManager {
            fn free_leaked_memory_other_ptr(ptr_type: &str, ptr: usize) {
                unsafe {
                    match ptr_type {
                        $(
                            t if t == std::any::type_name::<$t>() => {
                                let _ = Box::from_raw(ptr as *mut $t);
                            },
                        )*
                        _ => {
                            unreachable!("Unknown leaked pointer type: {}", ptr_type);
                        }
                    }
                }
            }
        }
    };
}
macro_rules! impl_leakable_value {
    ($($t:ty),* $(,)?) => {
        $(
            impl LeakableValue for $t {}
        )*
        impl LeakManager {
            fn free_leaked_memory_leakable_value(type_name: &str, ptr: usize, len: usize) {
                unsafe {
                    match type_name {
                        $(
                            t if t == std::any::type_name::<$t>() => {
                                let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut $t, len));
                            },
                        )*
                        _ => {
                            unreachable!("Unknown leaked value vector type: {}", type_name);
                        }
                    }
                }
            }
        }
    };
}
impl_leak_ptr!(
    aviutl2_sys::input2::WAVEFORMATEX,
    aviutl2_sys::output2::BITMAPINFOHEADER,
    aviutl2_sys::filter2::FILTER_ITEM,
);
impl_leakable_value!(
    aviutl2_sys::filter2::FILTER_ITEM_SELECT_ITEM,
    aviutl2_sys::module2::SCRIPT_MODULE_FUNCTION,
    usize
);

impl<T: IntoLeakedPtr> IntoLeakedPtr for Option<T> {
    fn into_leaked_ptr(self) -> (LeakType, usize) {
        match self {
            Some(value) => value.into_leaked_ptr(),
            None => (LeakType::Null, 0),
        }
    }
}

impl Drop for LeakManager {
    fn drop(&mut self) {
        self.free_leaked_memory();
    }
}

/// # Safety
///
/// - `ptr` は有効なLPCWSTRであること。
/// - `ptr` はNull Terminatedなu16文字列を指していること。
pub(crate) unsafe fn load_wide_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)) }
}

pub(crate) fn alert_error<T: std::fmt::Display>(error: T) {
    let _ = native_dialog::DialogBuilder::message()
        .set_title("エラー")
        .set_level(native_dialog::MessageLevel::Error)
        .set_text(format!("エラーが発生しました: {error}"))
        .alert()
        .show();
}

#[doc(hidden)]
#[expect(private_bounds)]
pub fn __output_log_if_error<T: MenuCallbackReturn>(result: T) {
    if let Some(err_msg) = result.into_optional_error() {
        let _ = crate::logger::write_error_log(&err_msg);
    }
}

#[doc(hidden)]
#[expect(private_bounds)]
pub fn __alert_if_error<T: MenuCallbackReturn>(result: T) {
    if let Some(err_msg) = result.into_optional_error() {
        alert_error(err_msg);
    }
}

trait MenuCallbackReturn {
    fn into_optional_error(self) -> Option<String>;
}
impl<E> MenuCallbackReturn for Result<(), E>
where
    Box<dyn std::error::Error>: From<E>,
{
    fn into_optional_error(self) -> Option<String> {
        match self {
            Ok(_) => None,
            Err(e) => {
                let boxed: Box<dyn std::error::Error> = e.into();
                Some(format!("{}", boxed))
            }
        }
    }
}
impl MenuCallbackReturn for () {
    fn into_optional_error(self) -> Option<String> {
        None
    }
}

/// ワイド文字列(LPCWSTR)を所有するRust文字列として扱うためのラッパー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CWString(Vec<u16>);

/// ヌルバイトを含む文字列を作成しようとした際のエラー。
#[derive(thiserror::Error, Debug)]
#[error("String contains null byte")]
pub struct NullByteError {
    position: usize,
    u16_seq: Vec<u16>,
}
impl NullByteError {
    pub fn nul_position(&self) -> usize {
        self.position
    }

    pub fn into_vec(self) -> Vec<u16> {
        self.u16_seq
    }
}

impl CWString {
    /// 新しいCWStringを作成します。
    ///
    /// # Errors
    ///
    /// `string`にヌルバイトが含まれている場合、`NullByteError`を返します。
    pub fn new(string: &str) -> Result<Self, NullByteError> {
        let mut wide: Vec<u16> = string.encode_utf16().collect();
        if let Some((pos, _)) = wide.iter().enumerate().find(|&(_, &c)| c == 0) {
            return Err(NullByteError {
                position: pos,
                u16_seq: wide,
            });
        }
        wide.push(0); // Null-terminate the string
        Ok(Self(wide))
    }

    /// 内部のポインタを取得します。
    ///
    /// # Warning
    ///
    /// <div class="warning">
    ///
    /// この`CWString`がDropされた後にこのポインタを使用すると未定義動作を引き起こします。
    ///
    /// </div>
    pub fn as_ptr(&self) -> *const u16 {
        self.0.as_ptr()
    }
}
impl std::fmt::Display for CWString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::from_utf16_lossy(&self.0[..self.0.len() - 1]);
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_filters() {
        let filters = vec![
            FileFilter {
                name: "Image Files".to_string(),
                extensions: vec!["png".to_string(), "jpg".to_string()],
            },
            FileFilter {
                name: "All Files".to_string(),
                extensions: vec![],
            },
        ];
        let formatted = format_file_filters(&filters);
        let expected = "Image Files (.png, .jpg)\x00*.png;*.jpg\x00All Files (*)\x00*\x00";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_file_filters_macro() {
        let filters = file_filters! {
            "Image Files" => ["png", "jpg"],
            "All Files" => []
        };
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].name, "Image Files");
        assert_eq!(
            filters[0].extensions,
            vec!["png".to_string(), "jpg".to_string()]
        );
        assert_eq!(filters[1].name, "All Files");
        assert_eq!(filters[1].extensions, Vec::<String>::new());
    }

    #[test]
    fn test_aviutl2_version() {
        let version = AviUtl2Version::new(2, 0, 15, 0);
        assert_eq!(version.0, 2001500);
        assert_eq!(version.major(), 2);
        assert_eq!(version.minor(), 0);
        assert_eq!(version.patch(), 15);
        assert_eq!(version.build(), 0);

        let version_from_u32: AviUtl2Version = 2001500u32.into();
        assert_eq!(version_from_u32, version);

        let u32_from_version: u32 = version.into();
        assert_eq!(u32_from_version, 2001500);
    }

    #[test]
    fn test_cwstring_new() {
        let s = "Hello, world!";
        let cwstring = CWString::new(s).unwrap();
        assert_eq!(unsafe { load_wide_string(cwstring.as_ptr()) }, s);

        let s_with_nul = "Hello\0world!";
        let err = CWString::new(s_with_nul).unwrap_err();
        assert_eq!(err.nul_position(), 5);
    }
}
