//! AviUtl2のキャッシュ関連機能へのインターフェースを提供します。
//!
//! # Panics
//!
//! キャッシュハンドルが初期化されていない場合panicします。

use std::{ffi::c_void, path::Path, sync::Arc};

use parking_lot::lock_api::RawRwLock;

use crate::{common::CWString, filter::RgbaPixel};

struct InternalCacheHandle(*mut aviutl2_sys::cache2::CACHE_HANDLE);
unsafe impl Send for InternalCacheHandle {}

static CACHE_HANDLE: std::sync::OnceLock<std::sync::Mutex<InternalCacheHandle>> =
    std::sync::OnceLock::new();

static CACHE_LOCKS: std::sync::LazyLock<dashmap::DashMap<usize, Arc<parking_lot::RawRwLock>>> =
    std::sync::LazyLock::new(dashmap::DashMap::new);

#[doc(hidden)]
pub struct GlobalCacheHandle;

/// CacheHandleをキャッシュの識別子として使用するための構造体。
pub const GLOBAL_CACHE_HANDLE: GlobalCacheHandle = GlobalCacheHandle;

impl AsCacheIdentifier for GlobalCacheHandle {
    fn as_cache_identifier(&self) -> *mut c_void {
        with_cache_handle(|handle| handle as *mut c_void).expect("Cache handle is not initialized")
    }
}

#[doc(hidden)]
pub fn __initialize_cache(handle: *mut aviutl2_sys::cache2::CACHE_HANDLE) {
    CACHE_HANDLE
        .set(std::sync::Mutex::new(InternalCacheHandle(handle)))
        .ok()
        .expect("Cache handle is already initialized");
}

#[doc(hidden)]
pub fn __initialize_cache_unwind(handle: *mut aviutl2_sys::cache2::CACHE_HANDLE) {
    if let Err(panic_info) = crate::__catch_unwind_with_panic_info(|| __initialize_cache(handle)) {
        tracing::error!("Panic occurred during InitializeCache: {}", panic_info);
        let _ = crate::logger::write_error_log(&panic_info);
    }
}

/// キャッシュ操作のエラー。
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("api call failed")]
    ApiCallFailed,
    #[error("input utf-16 string contains null byte")]
    InputCwstrContainsNull(#[from] crate::common::NullByteError),
    #[error("value is out of range")]
    ValueOutOfRange,
    #[error("invalid audio channel")]
    InvalidAudioChannel,
}

/// キャッシュの識別子を提供するためのトレイト。
pub trait AsCacheIdentifier {
    fn as_cache_identifier(&self) -> *mut c_void;
}

impl AsCacheIdentifier for *mut c_void {
    fn as_cache_identifier(&self) -> *mut c_void {
        *self
    }
}

impl AsCacheIdentifier for *const c_void {
    fn as_cache_identifier(&self) -> *mut c_void {
        *self as *mut c_void
    }
}

impl AsCacheIdentifier for usize {
    fn as_cache_identifier(&self) -> *mut c_void {
        *self as *mut c_void
    }
}

enum CacheAccessMode {
    Read,
    Write,
}

struct CacheAccessGuard {
    lock: Arc<parking_lot::RawRwLock>,
    mode: CacheAccessMode,
}

impl CacheAccessGuard {
    fn read(key: usize) -> Self {
        let lock = CACHE_LOCKS
            .entry(key)
            .or_insert_with(|| Arc::new(parking_lot::RawRwLock::INIT))
            .clone();
        lock.lock_shared();
        Self {
            lock,
            mode: CacheAccessMode::Read,
        }
    }

    fn write(key: usize) -> Self {
        let lock = CACHE_LOCKS
            .entry(key)
            .or_insert_with(|| Arc::new(parking_lot::RawRwLock::INIT))
            .clone();
        lock.lock_exclusive();
        Self {
            lock,
            mode: CacheAccessMode::Write,
        }
    }
}

impl Drop for CacheAccessGuard {
    fn drop(&mut self) {
        unsafe {
            match self.mode {
                CacheAccessMode::Read => self.lock.unlock_shared(),
                CacheAccessMode::Write => self.lock.unlock_exclusive(),
            }
        }
    }
}

/// 画像キャッシュデータの読み取りガード。
pub struct CacheImageReadGuard {
    _access: CacheAccessGuard,
    raw: aviutl2_sys::cache2::CACHE_IMAGE,
    len: usize,
}

/// 画像キャッシュデータの書き込みガード。
pub struct CacheImageWriteGuard {
    _access: CacheAccessGuard,
    raw: aviutl2_sys::cache2::CACHE_IMAGE,
    len: usize,
}

/// 音声キャッシュデータの読み取りガード。
pub struct CacheAudioReadGuard {
    _access: CacheAccessGuard,
    raw: aviutl2_sys::cache2::CACHE_AUDIO,
    len: usize,
}

/// 音声キャッシュデータの書き込みガード。
pub struct CacheAudioWriteGuard {
    _access: CacheAccessGuard,
    raw: aviutl2_sys::cache2::CACHE_AUDIO,
    len: usize,
}

unsafe impl Send for CacheImageReadGuard {}
unsafe impl Send for CacheImageWriteGuard {}
unsafe impl Send for CacheAudioReadGuard {}
unsafe impl Send for CacheAudioWriteGuard {}

impl CacheImageReadGuard {
    /// 画像の幅。
    pub fn width(&self) -> usize {
        self.raw.width as usize
    }

    /// 画像の高さ。
    pub fn height(&self) -> usize {
        self.raw.height as usize
    }

    /// RGBAピクセル列として取得する。
    pub fn as_slice(&self) -> &[RgbaPixel] {
        unsafe { std::slice::from_raw_parts(self.raw.buffer as *const RgbaPixel, self.len) }
    }

    /// RGBAピクセル列へのポインタを取得する。
    pub fn as_ptr(&self) -> *const RgbaPixel {
        self.raw.buffer as *const RgbaPixel
    }

    /// `u8`スライスとして取得する。
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.raw.buffer as *const u8,
                self.len * std::mem::size_of::<RgbaPixel>(),
            )
        }
    }

    /// `u8`スライスへのポインタを取得する。
    pub fn as_u8_ptr(&self) -> *const u8 {
        self.raw.buffer as *const u8
    }
}

impl CacheImageWriteGuard {
    /// 画像の幅。
    pub fn width(&self) -> usize {
        self.raw.width as usize
    }

    /// 画像の高さ。
    pub fn height(&self) -> usize {
        self.raw.height as usize
    }

    /// RGBAピクセル列として取得する。
    pub fn as_slice(&self) -> &[RgbaPixel] {
        unsafe { std::slice::from_raw_parts(self.raw.buffer as *const RgbaPixel, self.len) }
    }

    /// RGBAピクセル列として可変取得する。
    pub fn as_slice_mut(&mut self) -> &mut [RgbaPixel] {
        unsafe { std::slice::from_raw_parts_mut(self.raw.buffer as *mut RgbaPixel, self.len) }
    }

    /// RGBAピクセル列への可変ポインタを取得する。
    pub fn as_mut_ptr(&mut self) -> *mut RgbaPixel {
        self.raw.buffer as *mut RgbaPixel
    }

    /// `u8`スライスとして取得する。
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.raw.buffer as *const u8,
                self.len * std::mem::size_of::<RgbaPixel>(),
            )
        }
    }

    /// `u8`スライスとして可変取得する。
    pub fn as_u8_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.raw.buffer as *mut u8,
                self.len * std::mem::size_of::<RgbaPixel>(),
            )
        }
    }

    /// RGBAピクセル列への可変ポインタを取得する。
    pub fn as_mut_u8_ptr(&mut self) -> *mut u8 {
        self.raw.buffer as *mut u8
    }
}

impl CacheAudioReadGuard {
    /// 音声キャッシュのサンプル数。
    pub fn sample_num(&self) -> usize {
        self.len
    }

    /// 音声キャッシュのチャンネル数。
    pub fn channel_num(&self) -> usize {
        self.raw.channel_num as usize
    }

    /// 指定チャンネルのサンプル列を取得する。
    pub fn channel(&self, channel: crate::filter::AudioChannel) -> Option<&[f32]> {
        let buffer = self.channel_ptr(channel)?;
        Some(unsafe { std::slice::from_raw_parts(buffer, self.len) })
    }

    fn channel_ptr(&self, channel: crate::filter::AudioChannel) -> Option<*const f32> {
        match i32::from(channel) {
            0 if !self.raw.buffer0.is_null() => Some(self.raw.buffer0),
            1 if self.raw.channel_num >= 2 && !self.raw.buffer1.is_null() => Some(self.raw.buffer1),
            _ => None,
        }
    }
}

impl CacheAudioWriteGuard {
    /// 音声キャッシュのサンプル数。
    pub fn sample_num(&self) -> usize {
        self.len
    }

    /// 音声キャッシュのチャンネル数。
    pub fn channel_num(&self) -> usize {
        self.raw.channel_num as usize
    }

    /// 指定チャンネルのサンプル列を取得する。
    pub fn channel(&self, channel: crate::filter::AudioChannel) -> Option<&[f32]> {
        let buffer = self.channel_ptr(channel)?;
        Some(unsafe { std::slice::from_raw_parts(buffer, self.len) })
    }

    /// 指定チャンネルのサンプル列を可変取得する。
    pub fn channel_mut(&mut self, channel: crate::filter::AudioChannel) -> Option<&mut [f32]> {
        let buffer = self.channel_ptr_mut(channel)?;
        Some(unsafe { std::slice::from_raw_parts_mut(buffer, self.len) })
    }

    fn channel_ptr(&self, channel: crate::filter::AudioChannel) -> Option<*const f32> {
        match i32::from(channel) {
            0 if !self.raw.buffer0.is_null() => Some(self.raw.buffer0),
            1 if self.raw.channel_num >= 2 && !self.raw.buffer1.is_null() => Some(self.raw.buffer1),
            _ => None,
        }
    }

    fn channel_ptr_mut(&mut self, channel: crate::filter::AudioChannel) -> Option<*mut f32> {
        match i32::from(channel) {
            0 if !self.raw.buffer0.is_null() => Some(self.raw.buffer0),
            1 if self.raw.channel_num >= 2 && !self.raw.buffer1.is_null() => Some(self.raw.buffer1),
            _ => None,
        }
    }
}

/// 画像キャッシュデータを取得する。
pub fn get_image_cache(
    identifier: &impl AsCacheIdentifier,
    name: &str,
) -> Result<Option<CacheImageReadGuard>, CacheError> {
    let wide_name = CWString::new(name)?;
    let identifier = identifier.as_cache_identifier();
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).get_image_cache)(identifier, wide_name.as_ptr())
    })?;
    if raw.buffer.is_null() {
        Ok(None)
    } else {
        Ok(Some(image_read_guard(raw)?))
    }
}

/// 画像キャッシュデータを取得し、可変取得する。
pub fn get_image_cache_mut(
    identifier: &impl AsCacheIdentifier,
    name: &str,
) -> Result<Option<CacheImageWriteGuard>, CacheError> {
    let wide_name = CWString::new(name)?;
    let identifier = identifier.as_cache_identifier();
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).get_image_cache)(identifier, wide_name.as_ptr())
    })?;
    if raw.buffer.is_null() {
        Ok(None)
    } else {
        Ok(Some(image_write_guard(raw)?))
    }
}

/// 画像キャッシュデータを作成し、可変取得する。
pub fn create_image_cache(
    identifier: &impl AsCacheIdentifier,
    name: &str,
    width: usize,
    height: usize,
) -> Result<CacheImageWriteGuard, CacheError> {
    let wide_name = CWString::new(name)?;
    let width = i32::try_from(width).map_err(|_| CacheError::ValueOutOfRange)?;
    let height = i32::try_from(height).map_err(|_| CacheError::ValueOutOfRange)?;
    let identifier = identifier.as_cache_identifier();
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).create_image_cache)(identifier, wide_name.as_ptr(), width, height)
    })?;
    image_write_guard(raw)
}

/// 音声キャッシュデータを取得する。
pub fn get_audio_cache(
    identifier: &impl AsCacheIdentifier,
    name: &str,
) -> Result<Option<CacheAudioReadGuard>, CacheError> {
    let wide_name = CWString::new(name)?;
    let identifier = identifier.as_cache_identifier();
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).get_audio_cache)(identifier, wide_name.as_ptr())
    })?;
    if raw.buffer0.is_null() {
        Ok(None)
    } else {
        Ok(Some(audio_read_guard(raw)?))
    }
}

/// 音声キャッシュデータを取得し、可変取得する。
pub fn get_audio_cache_mut(
    identifier: &impl AsCacheIdentifier,
    name: &str,
) -> Result<Option<CacheAudioWriteGuard>, CacheError> {
    let wide_name = CWString::new(name)?;
    let identifier = identifier.as_cache_identifier();
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).get_audio_cache)(identifier, wide_name.as_ptr())
    })?;
    if raw.buffer0.is_null() {
        Ok(None)
    } else {
        Ok(Some(audio_write_guard(raw)?))
    }
}

/// 音声キャッシュデータを作成し、可変取得する。
pub fn create_audio_cache(
    identifier: &impl AsCacheIdentifier,
    name: &str,
    sample_num: usize,
    channel_num: usize,
) -> Result<CacheAudioWriteGuard, CacheError> {
    if !(1..=2).contains(&channel_num) {
        return Err(CacheError::InvalidAudioChannel);
    }
    let wide_name = CWString::new(name)?;
    let sample_num = i32::try_from(sample_num).map_err(|_| CacheError::ValueOutOfRange)?;
    let channel_num = i32::try_from(channel_num).map_err(|_| CacheError::ValueOutOfRange)?;
    let identifier = identifier.as_cache_identifier();
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).create_audio_cache)(identifier, wide_name.as_ptr(), sample_num, channel_num)
    })?;
    audio_write_guard(raw)
}

/// 画像ファイルから画像キャッシュデータを取得する。
pub fn get_image_file_cache(
    path: impl AsRef<Path>,
) -> Result<Option<CacheImageReadGuard>, CacheError> {
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let raw =
        with_cache_handle(|handle| unsafe { ((*handle).get_image_file_cache)(file.as_ptr()) })?;
    if raw.buffer.is_null() {
        Ok(None)
    } else {
        Ok(Some(image_read_guard(raw)?))
    }
}

fn image_read_guard(
    raw: aviutl2_sys::cache2::CACHE_IMAGE,
) -> Result<CacheImageReadGuard, CacheError> {
    let len = image_len(&raw)?;
    let access = CacheAccessGuard::read(raw.buffer as usize);
    Ok(CacheImageReadGuard {
        _access: access,
        raw,
        len,
    })
}

fn image_write_guard(
    raw: aviutl2_sys::cache2::CACHE_IMAGE,
) -> Result<CacheImageWriteGuard, CacheError> {
    if raw.buffer.is_null() {
        return Err(CacheError::ApiCallFailed);
    }
    let len = image_len(&raw)?;
    let access = CacheAccessGuard::write(raw.buffer as usize);
    Ok(CacheImageWriteGuard {
        _access: access,
        raw,
        len,
    })
}

fn audio_read_guard(
    raw: aviutl2_sys::cache2::CACHE_AUDIO,
) -> Result<CacheAudioReadGuard, CacheError> {
    let len = audio_len(&raw)?;
    let access = CacheAccessGuard::read(raw.buffer0 as usize);
    Ok(CacheAudioReadGuard {
        _access: access,
        raw,
        len,
    })
}

fn audio_write_guard(
    raw: aviutl2_sys::cache2::CACHE_AUDIO,
) -> Result<CacheAudioWriteGuard, CacheError> {
    if raw.buffer0.is_null() {
        return Err(CacheError::ApiCallFailed);
    }
    let len = audio_len(&raw)?;
    let access = CacheAccessGuard::write(raw.buffer0 as usize);
    Ok(CacheAudioWriteGuard {
        _access: access,
        raw,
        len,
    })
}

fn image_len(raw: &aviutl2_sys::cache2::CACHE_IMAGE) -> Result<usize, CacheError> {
    let width = usize::try_from(raw.width).map_err(|_| CacheError::ValueOutOfRange)?;
    let height = usize::try_from(raw.height).map_err(|_| CacheError::ValueOutOfRange)?;
    width.checked_mul(height).ok_or(CacheError::ValueOutOfRange)
}

fn audio_len(raw: &aviutl2_sys::cache2::CACHE_AUDIO) -> Result<usize, CacheError> {
    usize::try_from(raw.sample_num).map_err(|_| CacheError::ValueOutOfRange)
}

fn with_cache_handle<T>(
    f: impl FnOnce(*mut aviutl2_sys::cache2::CACHE_HANDLE) -> T,
) -> Result<T, CacheError> {
    let handle = CACHE_HANDLE.get().expect("Cache handle is not initialized");
    let handle = handle.lock().unwrap();
    if handle.0.is_null() {
        panic!("Cache handle raw pointer is null");
    }
    Ok(f(handle.0))
}
