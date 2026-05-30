//! AviUtl2のキャッシュ関連機能へのインターフェースを提供します。
//!
//! # Panics
//!
//! キャッシュハンドルが初期化されていない場合panicします。

use std::{ffi::c_void, path::Path, sync::Arc};

use num_rational::Rational32;
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

/// メディアファイルの画像キャッシュデータの読み取りガード。
pub struct CacheFileImageReadGuard {
    _access: CacheAccessGuard,
    raw: aviutl2_sys::cache2::CACHE_FILE_IMAGE,
    byte_len: usize,
}

unsafe impl Send for CacheImageReadGuard {}
unsafe impl Send for CacheImageWriteGuard {}
unsafe impl Send for CacheAudioReadGuard {}
unsafe impl Send for CacheAudioWriteGuard {}
unsafe impl Send for CacheFileImageReadGuard {}

/// メディアファイルの画像キャッシュのピクセルフォーマット。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheImagePixelFormat {
    /// DXGI_FORMAT_R8G8B8A8_UNORM。
    Rgba,
    /// DXGI_FORMAT_B8G8R8A8_UNORM。
    Bgra,
    /// DXGI_FORMAT_B8G8R8X8_UNORM。
    Bgr,
    /// DXGI_FORMAT_R16G16B16A16_UNORM。
    Pa64,
    /// DXGI_FORMAT_R16G16B16A16_FLOAT。
    Hf64,
    /// DXGI_FORMAT_YUY2。
    Yuy2,
    /// DXGI_FORMAT_R16G16B16A16_SNORM。
    Yc48,
}

impl From<aviutl2_sys::filter2::INPUT_PIXEL_FORMAT> for CacheImagePixelFormat {
    fn from(value: aviutl2_sys::filter2::INPUT_PIXEL_FORMAT) -> Self {
        match value {
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::RGBA => Self::Rgba,
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::BGRA => Self::Bgra,
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::BGR => Self::Bgr,
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::PA64 => Self::Pa64,
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::HF64 => Self::Hf64,
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::YUY2 => Self::Yuy2,
            aviutl2_sys::filter2::INPUT_PIXEL_FORMAT::YC48 => Self::Yc48,
        }
    }
}

/// メディアファイルのビデオ情報。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VideoFileInfo {
    /// 総時間。
    pub total_time: f64,
    /// 総フレーム数。
    pub frame_num: usize,
    /// トラック数。
    pub track_num: usize,
    /// 解像度。
    pub width: usize,
    /// 解像度。
    pub height: usize,
    /// フレームレート。
    pub fps: Rational32
}

/// メディアファイルのオーディオ情報。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioFileInfo {
    /// 総時間。
    pub total_time: f64,
    /// 総サンプル数。
    pub sample_num: usize,
    /// トラック数。
    pub track_num: usize,
    /// サンプリングレート。
    pub rate: i32,
    /// チャンネル数。
    pub channel: usize,
}

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

impl CacheFileImageReadGuard {
    /// 画像の幅。
    pub fn width(&self) -> usize {
        self.raw.width as usize
    }

    /// 画像の高さ。
    pub fn height(&self) -> usize {
        self.raw.height as usize
    }

    /// 画像データの横1ラインのバイト数。
    pub fn pitch(&self) -> usize {
        self.raw.pitch as usize
    }

    /// ピクセルフォーマット。
    pub fn format(&self) -> CacheImagePixelFormat {
        self.raw.format.into()
    }

    /// 画像データを `u8` スライスとして取得する。
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.raw.buffer as *const u8, self.byte_len) }
    }

    /// 画像データへのポインタを取得する。
    pub fn as_ptr(&self) -> *const c_void {
        self.raw.buffer
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

/// メディアファイルのビデオ情報を取得する。
pub fn get_video_file_info(path: impl AsRef<Path>) -> Result<Option<VideoFileInfo>, CacheError> {
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let mut raw = std::mem::MaybeUninit::<aviutl2_sys::cache2::VIDEO_INFO>::uninit();
    let success = with_cache_handle(|handle| unsafe {
        ((*handle).get_video_file_info)(
            file.as_ptr(),
            raw.as_mut_ptr(),
            std::mem::size_of::<aviutl2_sys::cache2::VIDEO_INFO>() as i32,
        )
    })?;
    if !success {
        return Ok(None);
    }
    let raw = unsafe { raw.assume_init() };
    Ok(Some(VideoFileInfo {
        total_time: raw.total_time,
        frame_num: usize::try_from(raw.frame_num).map_err(|_| CacheError::ValueOutOfRange)?,
        track_num: usize::try_from(raw.track_num).map_err(|_| CacheError::ValueOutOfRange)?,
        width: usize::try_from(raw.width).map_err(|_| CacheError::ValueOutOfRange)?,
        height: usize::try_from(raw.height).map_err(|_| CacheError::ValueOutOfRange)?,
        fps: Rational32::new(raw.rate, raw.scale),
    }))
}

/// メディアファイルのオーディオ情報を取得する。
pub fn get_audio_file_info(path: impl AsRef<Path>) -> Result<Option<AudioFileInfo>, CacheError> {
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let mut raw = std::mem::MaybeUninit::<aviutl2_sys::cache2::AUDIO_INFO>::uninit();
    let success = with_cache_handle(|handle| unsafe {
        ((*handle).get_audio_file_info)(
            file.as_ptr(),
            raw.as_mut_ptr(),
            std::mem::size_of::<aviutl2_sys::cache2::AUDIO_INFO>() as i32,
        )
    })?;
    if !success {
        return Ok(None);
    }
    let raw = unsafe { raw.assume_init() };
    Ok(Some(AudioFileInfo {
        total_time: raw.total_time,
        sample_num: usize::try_from(raw.sample_num).map_err(|_| CacheError::ValueOutOfRange)?,
        track_num: usize::try_from(raw.track_num).map_err(|_| CacheError::ValueOutOfRange)?,
        rate: raw.rate,
        channel: usize::try_from(raw.channel).map_err(|_| CacheError::ValueOutOfRange)?,
    }))
}

/// 画像ファイルから画像データをキャッシュ経由で取得する。
pub fn get_image_file_cache(
    path: impl AsRef<Path>,
) -> Result<Option<CacheFileImageReadGuard>, CacheError> {
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let raw =
        with_cache_handle(|handle| unsafe { ((*handle).get_image_file_cache)(file.as_ptr()) })?;
    if raw.buffer.is_null() {
        Ok(None)
    } else {
        Ok(Some(file_image_read_guard(raw)?))
    }
}

/// メディアファイルから指定フレームの画像データをキャッシュ経由で取得する。
pub fn get_video_file_cache(
    path: impl AsRef<Path>,
    track: usize,
    frame: usize,
) -> Result<Option<CacheFileImageReadGuard>, CacheError> {
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let track = i32::try_from(track).map_err(|_| CacheError::ValueOutOfRange)?;
    let frame = i32::try_from(frame).map_err(|_| CacheError::ValueOutOfRange)?;
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).get_video_file_cache)(file.as_ptr(), track, frame)
    })?;
    if raw.buffer.is_null() {
        Ok(None)
    } else {
        Ok(Some(file_image_read_guard(raw)?))
    }
}

/// メディアファイルから指定時間の画像データをキャッシュ経由で取得する。
pub fn get_video_file_cache_by_time(
    path: impl AsRef<Path>,
    track: usize,
    time: f64,
) -> Result<Option<CacheFileImageReadGuard>, CacheError> {
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let track = i32::try_from(track).map_err(|_| CacheError::ValueOutOfRange)?;
    let raw = with_cache_handle(|handle| unsafe {
        ((*handle).get_video_file_cache_by_time)(file.as_ptr(), track, time)
    })?;
    if raw.buffer.is_null() {
        Ok(None)
    } else {
        Ok(Some(file_image_read_guard(raw)?))
    }
}

/// メディアファイルから音声データをキャッシュ経由で取得する。
///
/// # Panics
///
/// `buffer0` と `buffer1` の長さが異なる場合にpanicします。
///
/// # Returns
///
/// 戻り値は実際に取得したサンプル数です。
pub fn get_audio_file_data(
    path: impl AsRef<Path>,
    track: usize,
    sample_index: usize,
    buffer0: &mut [f32],
    buffer1: &mut [f32],
) -> Result<usize, CacheError> {
    assert_eq!(
        buffer0.len(),
        buffer1.len(),
        "Audio buffers must have the same length"
    );
    let file = CWString::new(&path.as_ref().to_string_lossy())?;
    let track = i32::try_from(track).map_err(|_| CacheError::ValueOutOfRange)?;
    let sample_index = i64::try_from(sample_index).map_err(|_| CacheError::ValueOutOfRange)?;
    let sample_num = i32::try_from(buffer0.len()).map_err(|_| CacheError::ValueOutOfRange)?;
    let read = with_cache_handle(|handle| unsafe {
        ((*handle).get_audio_file_data)(
            file.as_ptr(),
            track,
            sample_index,
            sample_num,
            buffer0.as_mut_ptr(),
            buffer1.as_mut_ptr(),
        )
    })?;
    usize::try_from(read).map_err(|_| CacheError::ValueOutOfRange)
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

fn file_image_read_guard(
    raw: aviutl2_sys::cache2::CACHE_FILE_IMAGE,
) -> Result<CacheFileImageReadGuard, CacheError> {
    let byte_len = file_image_byte_len(&raw)?;
    let access = CacheAccessGuard::read(raw.buffer as usize);
    Ok(CacheFileImageReadGuard {
        _access: access,
        raw,
        byte_len,
    })
}

fn image_len(raw: &aviutl2_sys::cache2::CACHE_IMAGE) -> Result<usize, CacheError> {
    let width = usize::try_from(raw.width).map_err(|_| CacheError::ValueOutOfRange)?;
    let height = usize::try_from(raw.height).map_err(|_| CacheError::ValueOutOfRange)?;
    width.checked_mul(height).ok_or(CacheError::ValueOutOfRange)
}

fn file_image_byte_len(raw: &aviutl2_sys::cache2::CACHE_FILE_IMAGE) -> Result<usize, CacheError> {
    let pitch = usize::try_from(raw.pitch).map_err(|_| CacheError::ValueOutOfRange)?;
    let height = usize::try_from(raw.height).map_err(|_| CacheError::ValueOutOfRange)?;
    pitch.checked_mul(height).ok_or(CacheError::ValueOutOfRange)
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
