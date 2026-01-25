//! AviUtl2の設定関連機能へのインターフェースを提供します。

use crate::{CWString, NullByteError, load_wide_string};

/// フォント情報
#[derive(Debug, Clone, PartialEq)]
pub struct FontInfo {
    /// フォント名
    pub name: String,
    /// フォントサイズ
    pub size: f32,
}
impl FontInfo {
    /// 内部のFONT_INFOからFontInfoを作成する。
    ///
    /// # Safety
    ///
    /// `font_info_ptr`は有効なポインタである必要があります。
    unsafe fn from_raw(font_info_ptr: *mut aviutl2_sys::config2::FONT_INFO) -> Self {
        let font_info = unsafe { &*font_info_ptr };
        let name = unsafe { load_wide_string(font_info.name) };
        Self {
            name,
            size: font_info.size,
        }
    }
}

struct InternalConfigHandle {
    raw: *mut aviutl2_sys::config2::CONFIG_HANDLE,
}
unsafe impl Send for InternalConfigHandle {}

static CONFIG_HANDLE: std::sync::OnceLock<std::sync::Mutex<InternalConfigHandle>> =
    std::sync::OnceLock::new();

/// アプリケーションデータフォルダへのパスを取得する。
pub fn app_data_path() -> std::path::PathBuf {
    let path = unsafe {
        load_wide_string(
            CONFIG_HANDLE
                .get()
                .expect("Config handle not initialized")
                .lock()
                .unwrap()
                .raw
                .as_ref()
                .expect("Config handle raw pointer is null")
                .app_data_path,
        )
    };
    std::path::PathBuf::from(path)
}

/// 現在の言語設定で定義されているテキストを取得する。
///
/// 参照する言語設定のセクションはビルドしたプラグインのファイル名になります。
///
/// # Arguments
///
/// - `text`: 元のテキスト（.aul2ファイルのキー名）
pub fn translate(text: &str) -> Result<String, NullByteError> {
    let wide_text = CWString::new(text)?;
    let translated = unsafe {
        let handle = CONFIG_HANDLE
            .get()
            .expect("Config handle not initialized")
            .lock()
            .unwrap();
        (handle
            .raw
            .as_ref()
            .expect("Config handle raw pointer is null")
            .translate)(handle.raw, wide_text.as_ptr())
    };
    Ok(unsafe { load_wide_string(translated) })
}

/// 現在の言語設定で定義されているテキストを取得する。
///
/// 任意のセクションから取得出来ます。
///
/// # Arguments
///
/// - `section`: 言語設定のセクション（.aul2ファイルのセクション名）
/// - `text`: 元のテキスト（.aul2ファイルのキー名）
pub fn get_language_text(section: &str, text: &str) -> Result<String, NullByteError> {
    let wide_section = CWString::new(section)?;
    let wide_text = CWString::new(text)?;
    let translated = unsafe {
        let handle = CONFIG_HANDLE
            .get()
            .expect("Config handle not initialized")
            .lock()
            .unwrap();
        (handle
            .raw
            .as_ref()
            .expect("Config handle raw pointer is null")
            .get_language_text)(handle.raw, wide_section.as_ptr(), wide_text.as_ptr())
    };
    Ok(unsafe { load_wide_string(translated) })
}

/// 設定ファイルで定義されているフォント情報を取得する。
///
/// # Note
///
/// 取得出来ない場合はデフォルトのフォントが返却されます。
///
/// # Arguments
///
/// - `key`: 設定ファイル(style.conf)の[Font]のキー名
pub fn get_font_info(key: &str) -> Result<FontInfo, std::ffi::NulError> {
    let c_key = std::ffi::CString::new(key)?;
    let font_info_ptr = unsafe {
        let handle = CONFIG_HANDLE
            .get()
            .expect("Config handle not initialized")
            .lock()
            .unwrap();
        (handle
            .raw
            .as_ref()
            .expect("Config handle raw pointer is null")
            .get_font_info)(handle.raw, c_key.as_ptr())
    };
    Ok(unsafe { FontInfo::from_raw(font_info_ptr) })
}

/// 設定ファイルで定義されている色コードを取得する。
///
/// # Note
///
/// 取得出来ない場合は0が返却されます。
///
/// # Arguments
///
/// - `key`: 設定ファイル(style.conf)の[Color]のキー名
pub fn get_color_code(key: &str) -> Result<i32, std::ffi::NulError> {
    let c_key = std::ffi::CString::new(key)?;
    let color_code = unsafe {
        let handle = CONFIG_HANDLE
            .get()
            .expect("Config handle not initialized")
            .lock()
            .unwrap();
        (handle
            .raw
            .as_ref()
            .expect("Config handle raw pointer is null")
            .get_color_code)(handle.raw, c_key.as_ptr())
    };
    Ok(color_code)
}

/// 設定ファイルで定義されているレイアウトサイズを取得する。
///
/// # Note
///
/// 取得出来ない場合は0が返却されます。
///
/// # Arguments
///
/// - `key`: 設定ファイル(style.conf)の[Layout]のキー名
pub fn get_layout_size(key: &str) -> Result<i32, std::ffi::NulError> {
    let c_key = std::ffi::CString::new(key)?;
    let layout_size = unsafe {
        let handle = CONFIG_HANDLE
            .get()
            .expect("Config handle not initialized")
            .lock()
            .unwrap();
        (handle
            .raw
            .as_ref()
            .expect("Config handle raw pointer is null")
            .get_layout_size)(handle.raw, c_key.as_ptr())
    };
    Ok(layout_size)
}

#[doc(hidden)]
pub fn __initialize_config_handle(raw: *mut aviutl2_sys::config2::CONFIG_HANDLE) {
    CONFIG_HANDLE
        .set(std::sync::Mutex::new(InternalConfigHandle { raw }))
        .unwrap_or_else(|_| {
            panic!("Config handle is already initialized");
        });
}

#[doc(hidden)]
pub fn __initialize_config_handle_unwind(raw: *mut aviutl2_sys::config2::CONFIG_HANDLE) {
    if let Err(panic_info) =
        crate::__catch_unwind_with_panic_info(|| __initialize_config_handle(raw))
    {
        log::error!("Panic occurred during InitializeConfig: {}", panic_info);
        crate::common::alert_error(&panic_info);
    }
}
