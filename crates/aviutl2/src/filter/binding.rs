use parking_lot::lock_api::RawRwLock;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use super::{ErasedFilterConfigData, config};
use crate::common::{AnyResult, AviUtl2Info, Rational32};

/// 入力プラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct FilterPluginTable {
    /// プラグインの名前。
    pub name: String,
    /// ラベルの初期値。
    /// Noneの場合、デフォルトのラベルになります
    pub label: Option<String>,
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,

    /// 対応している機能のフラグ。
    pub flags: FilterPluginFlags,

    /// 設定項目。
    pub config_items: Vec<config::FilterConfigItem>,
}

define_bitflag! {
    /// フィルタプラグインのフラグ。
    ///
    /// # See Also
    ///
    /// - [`crate::bitflag!`]
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    pub struct FilterPluginFlags: i32 {
        /// 画像フィルタをサポートするかどうか。
        video: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_VIDEO,

        /// 音声フィルタをサポートするかどうか。
        audio: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_AUDIO,

        /// カスタムオブジェクトにするかどうか。
        /// `true` の場合、カスタムオブジェクトとして動作します。
        /// `false` の場合、フィルタ効果として動作します。
        as_object: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_INPUT,

        /// フィルタオブジェクトをサポートするかどうか。
        /// `true` の場合、フィルタオブジェクトとして使えるようになります。
        as_filter: aviutl2_sys::filter2::FILTER_PLUGIN_TABLE::FLAG_FILTER,
    }
}

/// フィルタプラグインのトレイト。
/// このトレイトを実装し、[`crate::register_filter_plugin!`] マクロを使用してプラグインを登録します。
pub trait FilterPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> FilterPluginTable;

    /// 画像フィルタ処理関数。
    ///
    /// # Note
    ///
    /// フィルタオブジェクトの場合、画像サイズは変更できません。
    fn proc_video(
        &self,
        _config: &[config::FilterConfigItem],
        _video: &mut FilterProcVideo,
    ) -> AnyResult<()> {
        anyhow::bail!("proc_video is not implemented");
    }

    /// 音声フィルタ処理関数。
    fn proc_audio(
        &self,
        _config: &[config::FilterConfigItem],
        _audio: &mut FilterProcAudio,
    ) -> AnyResult<()> {
        anyhow::bail!("proc_audio is not implemented");
    }

    /// シングルトンインスタンスを参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance<R>(f: impl FnOnce(&Self) -> R) -> R
    where
        Self: crate::filter::__bridge::FilterSingleton,
    {
        <Self as crate::filter::__bridge::FilterSingleton>::with_instance(f)
    }

    /// シングルトンインスタンスを可変参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance_mut<R>(f: impl FnOnce(&mut Self) -> R) -> R
    where
        Self: crate::filter::__bridge::FilterSingleton,
    {
        <Self as crate::filter::__bridge::FilterSingleton>::with_instance_mut(f)
    }
}

/// シーン情報。
#[derive(Debug, Clone, Copy)]
pub struct SceneInfo {
    /// 解像度（幅）。
    pub width: u32,
    /// 解像度（高さ）。
    pub height: u32,
    /// フレームレート。
    pub frame_rate: Rational32,
    /// サンプリングレート。
    pub sample_rate: u32,
}

/// オブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct ObjectInfo {
    /// ID。
    /// アプリ起動ごとの固有IDです。
    pub id: i64,
    /// オブジェクトの内の対象エフェクトのID。
    /// アプリ起動ごとの固有IDです。
    pub effect_id: i64,
    /// オブジェクトの現在のフレーム番号。
    pub frame: u32,
    /// オブジェクトの総フレーム数。
    pub frame_total: u32,
    /// オブジェクトの現在の時間（秒）。
    pub time: f64,
    /// オブジェクトの総時間（秒）。
    pub time_total: f64,
}

/// 画像フィルタのオブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct VideoObjectInfo {
    /// オブジェクトの現在の画像サイズの幅。
    pub width: u32,
    /// オブジェクトの現在の画像サイズの高さ。
    pub height: u32,
}

/// 音声フィルタのオブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct AudioObjectInfo {
    /// オブジェクトの現在の音声サンプル位置。
    pub sample_index: u64,
    /// オブジェクトの総サンプル数。
    pub sample_total: u64,
    /// オブジェクトの現在の音声サンプル数。
    pub sample_num: u32,
    /// オブジェクトの現在の音声チャンネル数。
    /// 通常2になります。
    pub channel_num: u32,
}

/// RGBAのピクセル。
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout,
)]
pub struct RgbaPixel {
    /// 赤。
    pub r: u8,
    /// 緑。
    pub g: u8,
    /// 青。
    pub b: u8,
    /// アルファ。
    pub a: u8,
}

/// 画像フィルタ処理のための構造体。
#[derive(Debug)]
pub struct FilterProcVideo {
    /// シーン情報。
    pub scene: SceneInfo,
    /// オブジェクト情報。
    pub object: ObjectInfo,
    /// 画像フィルタ特有のオブジェクト情報。
    pub video_object: VideoObjectInfo,

    pub(crate) inner: *const aviutl2_sys::filter2::FILTER_PROC_VIDEO,
}
unsafe impl Send for FilterProcVideo {}
unsafe impl Sync for FilterProcVideo {}

impl FilterProcVideo {
    /// 現在の画像のデータを取得する。
    /// RGBA32bit で取得されます。
    ///
    /// # Panics
    ///
    /// `buffer` をバイト列に変換した際の長さが `width * height * 4` と一致しない場合、パニックします。
    /// 例えば[`u8`] の場合、`buffer` の長さは `width * height * 4` と一致する必要があり、
    /// [`RgbaPixel`] の場合、`buffer` の長さは `width * height` と一致する必要があります。
    ///
    /// # Note
    ///
    /// [`FilterPluginFlags::video`] が `true` の場合、この関数は何もせずに 0 を返します。
    pub fn get_image_data<T>(&mut self, buffer: &mut [T]) -> usize
    where
        T: Copy + FromBytes + Immutable,
    {
        if self.video_object.width == 0 || self.video_object.height == 0 {
            log::warn!("width or height is 0, perhaps the filter plugin is a custom object");
            return 0;
        }
        assert_eq!(
            std::mem::size_of_val(buffer),
            (self.video_object.width * self.video_object.height * 4) as usize,
            "buffer length as bytes does not match width * height * 4"
        );
        assert!(
            std::mem::align_of::<T>() >= std::mem::align_of::<aviutl2_sys::filter2::PIXEL_RGBA>(),
            "buffer alignment is not sufficient"
        );
        let width = self.video_object.width as usize;
        let height = self.video_object.height as usize;
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.get_image_data)(
                buffer.as_mut_ptr() as *mut u8 as *mut aviutl2_sys::filter2::PIXEL_RGBA
            )
        };

        width * height * 4
    }

    /// 現在の画像のデータを設定する。
    ///
    /// # Panics
    ///
    /// `data` をバイト列に変換した際の長さが `width * height * 4` と一致しない場合、パニックします。
    pub fn set_image_data<T: IntoBytes + Immutable>(
        &mut self,
        data: &[T],
        width: u32,
        height: u32,
    ) {
        let bytes = &data.as_bytes();
        assert_eq!(
            bytes.len(),
            (width * height * 4) as usize,
            "data length does not match width * height * 4"
        );
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_image_data)(
                bytes.as_ptr() as *const aviutl2_sys::filter2::PIXEL_RGBA,
                width as i32,
                height as i32,
            )
        };
    }

    /// 現在のオブジェクトの画像データのポインタをID3D11Texture2Dのポインタとして取得する。
    ///
    /// # Warning
    ///
    /// [`Self::set_image_data`] によって現在の画像が変更されるかフィルタ処理の終了まで有効です。
    pub fn get_image_texture2d(&mut self) -> *mut std::ffi::c_void {
        let inner = unsafe { &*self.inner };
        unsafe { (inner.get_image_texture2d)() }
    }

    /// 現在のフレームバッファの画像データのポインタをID3D11Texture2Dのポインタとして取得する。
    ///
    /// # Warning
    ///
    /// フィルタ処理の終了まで有効です。
    pub fn get_framebuffer_texture2d(&mut self) -> *mut std::ffi::c_void {
        let inner = unsafe { &*self.inner };
        unsafe { (inner.get_framebuffer_texture2d)() }
    }
}

/// 音声フィルタ処理のための構造体。
#[derive(Debug)]
pub struct FilterProcAudio {
    /// シーン情報。
    pub scene: SceneInfo,
    /// オブジェクト情報。
    pub object: ObjectInfo,
    /// 音声フィルタ特有のオブジェクト情報。
    pub audio_object: AudioObjectInfo,

    pub(crate) inner: *const aviutl2_sys::filter2::FILTER_PROC_AUDIO,
}

unsafe impl Send for FilterProcAudio {}
unsafe impl Sync for FilterProcAudio {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    Left,
    Right,
    Any(i32),
}
impl From<i32> for AudioChannel {
    fn from(value: i32) -> Self {
        match value {
            0 => AudioChannel::Left,
            1 => AudioChannel::Right,
            v => AudioChannel::Any(v),
        }
    }
}
impl From<AudioChannel> for i32 {
    fn from(value: AudioChannel) -> Self {
        match value {
            AudioChannel::Left => 0,
            AudioChannel::Right => 1,
            AudioChannel::Any(v) => v,
        }
    }
}

impl FilterProcAudio {
    /// 現在の音声のデータを取得する。
    /// `channel` は 0 が左チャンネル、1 が右チャンネルです。
    ///
    /// # Panics
    ///
    /// `buffer` の長さが `sample_num` と一致しない場合、パニックします。
    pub fn get_sample_data(&mut self, channel: AudioChannel, buffer: &mut [f32]) -> usize {
        let sample_num = self.audio_object.sample_num as usize;
        assert_eq!(
            buffer.len(),
            sample_num,
            "buffer length does not match sample_num"
        );
        let inner = unsafe { &*self.inner };
        unsafe { (inner.get_sample_data)(buffer.as_mut_ptr(), channel.into()) };
        sample_num
    }

    /// 現在の音声のデータを設定する。
    /// `channel` は 0 が左チャンネル、1 が右チャンネルです。
    ///
    /// # Panics
    ///
    /// `data` の長さが `sample_num` と一致しない場合、パニックします。
    pub fn set_sample_data(&mut self, channel: AudioChannel, data: &[f32]) {
        let sample_num = self.audio_object.sample_num as usize;
        assert_eq!(
            data.len(),
            sample_num,
            "data length does not match sample_num"
        );
        let inner = unsafe { &*self.inner };
        unsafe { (inner.set_sample_data)(data.as_ptr(), channel.into()) };
    }
}

/// フィルタプラグインでのデータを使うためのハンドル。
/// RwLockのような仕組みで安全にデータを扱うことができます。
#[derive(Debug)]
pub struct FilterConfigDataHandle<T: Copy> {
    pub(crate) inner: *mut T,
}

unsafe impl<T: Send + Sync + Copy> Send for FilterConfigDataHandle<T> {}
unsafe impl<T: Send + Sync + Copy> Sync for FilterConfigDataHandle<T> {}

static HANDLES: std::sync::LazyLock<dashmap::DashMap<usize, parking_lot::RawRwLock>> =
    std::sync::LazyLock::new(dashmap::DashMap::new);
static OWNED_REFERENCES: std::sync::LazyLock<
    std::sync::Arc<dashmap::DashMap<usize, std::sync::atomic::AtomicUsize>>,
> = std::sync::LazyLock::new(|| std::sync::Arc::new(dashmap::DashMap::new()));

impl<T: Copy> Clone for FilterConfigDataHandle<T> {
    fn clone(&self) -> Self {
        if !self.inner.is_null() {
            let addr = self.inner as usize;
            if OWNED_REFERENCES.contains_key(&addr) {
                let entry = OWNED_REFERENCES.get(&addr).unwrap();
                entry.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
        Self { inner: self.inner }
    }
}
impl<T: Copy> Drop for FilterConfigDataHandle<T> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            let addr = self.inner as usize;
            if let Some(entry) = OWNED_REFERENCES.get(&addr) {
                let prev = entry.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                if prev == 1 {
                    unsafe {
                        let _boxed = Box::from_raw(self.inner);
                    }
                    // NOTE: ここでdropしないとdeadlockする
                    drop(entry);
                    OWNED_REFERENCES.remove(&addr);
                }
            }
        }
    }
}

impl<T: Copy> FilterConfigDataHandle<T> {
    #[doc(hidden)]
    pub fn __generics_default_value() -> T
    where
        T: Default,
    {
        T::default()
    }

    #[doc(hidden)]
    pub fn __from_erased(erased: &ErasedFilterConfigData) -> Self {
        Self {
            inner: erased.value.map_or(std::ptr::null_mut(), |v| v.as_ptr()) as *mut T,
        }
    }

    #[doc(hidden)]
    pub fn __new_owned(value: T) -> Self {
        let boxed = Box::new(value);
        let pointer = Box::into_raw(boxed);
        let addr = pointer as *mut () as usize;
        OWNED_REFERENCES.insert(addr, std::sync::atomic::AtomicUsize::new(1));
        Self { inner: pointer }
    }

    /// データを読み取るためのロックを取得する。
    pub fn read<'handle>(&'handle self) -> FilterConfigDataReadGuard<'handle, T> {
        let addr = self.inner as *mut () as usize;
        let lock = HANDLES
            .entry(addr)
            .or_insert_with(|| parking_lot::RawRwLock::INIT);
        let lock = lock.value();

        lock.lock_shared();
        FilterConfigDataReadGuard::new(self.inner)
    }

    /// データを読み取るためのロックの取得を試みる。
    /// ロックが取得できなかった場合は `None` を返します。
    pub fn try_read<'handle>(&'handle self) -> Option<FilterConfigDataReadGuard<'handle, T>> {
        let addr = self.inner as *mut () as usize;
        let lock = HANDLES
            .entry(addr)
            .or_insert_with(|| parking_lot::RawRwLock::INIT);
        let lock = lock.value();

        if lock.try_lock_shared() {
            Some(FilterConfigDataReadGuard::new(self.inner))
        } else {
            None
        }
    }

    /// データを書き込むためのロックを取得する。
    pub fn write<'handle>(&'handle self) -> FilterConfigDataWriteGuard<'handle, T> {
        let addr = self.inner as *mut () as usize;
        let lock = HANDLES
            .entry(addr)
            .or_insert_with(|| parking_lot::RawRwLock::INIT);
        let lock = lock.value();
        lock.lock_exclusive();
        FilterConfigDataWriteGuard::new(self.inner)
    }

    /// データを書き込むためのロックの取得を試みる。
    /// ロックが取得できなかった場合は `None` を返します。
    pub fn try_write<'handle>(&'handle self) -> Option<FilterConfigDataWriteGuard<'handle, T>> {
        let addr = self.inner as *mut () as usize;
        let lock = HANDLES
            .entry(addr)
            .or_insert_with(|| parking_lot::RawRwLock::INIT);
        let lock = lock.value();
        if lock.try_lock_exclusive() {
            Some(FilterConfigDataWriteGuard::new(self.inner))
        } else {
            None
        }
    }

    /// 内部のポインタを取得する。
    ///
    /// # Warning
    ///
    /// このポインタを直接操作するとデータ競合が発生する可能性があります。
    pub fn as_ptr(&self) -> *mut T {
        self.inner
    }
}

/// フィルタプラグインのデータを読み取るためのガード。
pub struct FilterConfigDataReadGuard<'handle, T: Copy> {
    pub(crate) inner: *mut T,
    _handle: std::marker::PhantomData<&'handle FilterConfigDataHandle<T>>,
}
unsafe impl<T: Send + Sync + Copy> Send for FilterConfigDataReadGuard<'_, T> {}
unsafe impl<T: Send + Sync + Copy> Sync for FilterConfigDataReadGuard<'_, T> {}
impl<T: Copy> FilterConfigDataReadGuard<'_, T> {
    fn new<'handle>(inner: *mut T) -> FilterConfigDataReadGuard<'handle, T> {
        FilterConfigDataReadGuard {
            inner,
            _handle: std::marker::PhantomData,
        }
    }
}
impl<T: Copy> Drop for FilterConfigDataReadGuard<'_, T> {
    fn drop(&mut self) {
        let addr = self.inner as *mut () as usize;
        if let Some(entry) = HANDLES.get(&addr) {
            let lock = entry.value();
            unsafe { lock.unlock_shared() };
        }
    }
}
impl<T: Copy> std::convert::AsRef<T> for FilterConfigDataReadGuard<'_, T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.inner }
    }
}
impl<T: Copy> std::ops::Deref for FilterConfigDataReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// フィルタプラグインのデータを書き込むためのガード。
pub struct FilterConfigDataWriteGuard<'handle, T: Copy> {
    pub(crate) inner: *mut T,
    _handle: std::marker::PhantomData<&'handle FilterConfigDataHandle<T>>,
}

unsafe impl<T: Send + Sync + Copy> Send for FilterConfigDataWriteGuard<'_, T> {}
unsafe impl<T: Send + Sync + Copy> Sync for FilterConfigDataWriteGuard<'_, T> {}
impl<T: Copy> FilterConfigDataWriteGuard<'_, T> {
    fn new<'handle>(inner: *mut T) -> FilterConfigDataWriteGuard<'handle, T> {
        FilterConfigDataWriteGuard {
            inner,
            _handle: std::marker::PhantomData,
        }
    }
}
impl<T: Copy> Drop for FilterConfigDataWriteGuard<'_, T> {
    fn drop(&mut self) {
        let addr = self.inner as *mut () as usize;
        if let Some(entry) = HANDLES.get(&addr) {
            let lock = entry.value();
            unsafe { lock.unlock_exclusive() };
        }
    }
}
impl<T: Copy> std::convert::AsMut<T> for FilterConfigDataWriteGuard<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner }
    }
}
impl<T: Copy> std::ops::Deref for FilterConfigDataWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}
impl<T: Copy> std::ops::DerefMut for FilterConfigDataWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_config_data_handle_reads_initial_value() {
        let handle = FilterConfigDataHandle::<u32>::__new_owned(42);
        let read_guard = handle.read();
        assert_eq!(*read_guard, 42);
    }

    #[test]
    fn filter_config_data_handle_writes_and_reads_updated_value() {
        let handle = FilterConfigDataHandle::<u32>::__new_owned(42);
        {
            let mut write_guard = handle.write();
            *write_guard = 100;
        }
        let read_guard = handle.read();
        assert_eq!(*read_guard, 100);
    }

    #[test]
    fn filter_config_data_handle_try_read_fails_when_locked_for_write() {
        let handle = FilterConfigDataHandle::<u32>::__new_owned(42);
        let _write_guard = handle.write();
        let try_read_guard = handle.try_read();
        assert!(try_read_guard.is_none());
    }

    #[test]
    fn filter_config_data_handle_try_write_fails_when_locked_for_read() {
        let handle = FilterConfigDataHandle::<u32>::__new_owned(42);
        let _read_guard = handle.read();
        let try_write_guard = handle.try_write();
        assert!(try_write_guard.is_none());
    }

    #[test]
    fn filter_config_data_handle_clone_shares_state() {
        let handle = FilterConfigDataHandle::<u32>::__new_owned(42);
        let cloned_handle = handle.clone();
        {
            let mut write_guard = handle.write();
            *write_guard = 100;
        }
        let read_guard = cloned_handle.read();
        assert_eq!(*read_guard, 100);
    }

    #[test]
    fn filter_config_data_handle_never_drops_data_for_borrowed() {
        let mut data =
            crate::filter::ErasedFilterConfigData::with_default_value("test".to_string(), 42);
        let data_ptr = Box::into_raw(Box::new(42u32));
        data.value = Some(std::ptr::NonNull::new(data_ptr as _).unwrap());
        let handle = FilterConfigDataHandle::<u32>::__from_erased(&data);
        drop(handle);

        assert_eq!(unsafe { *data_ptr }, 42);
    }

    #[test]
    fn filter_config_data_handle_reads_value_from_erased_data() {
        let boxed = Box::new(77u32);
        let ptr = std::ptr::NonNull::from(boxed.as_ref());
        let data = crate::filter::FilterConfigData {
            name: "test".to_string(),
            value: Some(ptr),
            default_value: 0,
        };
        let erased = data.erase_type();
        let handle = FilterConfigDataHandle::<u32>::__from_erased(&erased);
        let read_guard = handle.read();

        assert_eq!(*read_guard, 77);
        assert_eq!(erased.value.unwrap().as_ptr() as *mut u32, ptr.as_ptr());

        drop(read_guard);
        drop(handle);
        drop(boxed);
    }
}
