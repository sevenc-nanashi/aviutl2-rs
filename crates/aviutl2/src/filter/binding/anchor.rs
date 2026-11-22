use super::{FilterProcVideo, FilterUserdata, FilterUserdataHandle, RgbaPixel};

/// アンカーのドラッグ通知。座標はオブジェクト座標系です。
///
/// # Example
///
/// ```no_run
/// use aviutl2::filter::*;
/// #[filter_config_items]
/// struct Config {
///     #[data]
///     point: FilterConfigDataHandle<[f32; 3]>,
/// }
/// fn register(config: &[FilterConfigItem], video: &mut FilterProcVideo<()>) {
///     let position = *Config::from_config_items(config).point.read();
///     if let Some(mut anchor) = video.anchor() {
///         anchor.set_anchor_move_point(0, position, Default::default(), |config, anchor| {
///             let config = Config::from_config_items(config);
///             *config.point.write() = [anchor.x, anchor.y, anchor.z];
///             Ok(())
///         });
///     }
/// }
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct FilterProcAnchor<U: FilterUserdata> {
    /// 登録時のインデックス。
    pub index: i32,
    /// 移動先のX座標。
    pub x: f32,
    /// 移動先のY座標。
    pub y: f32,
    /// 移動先のZ座標。
    pub z: f32,
    /// 今回のX移動量。
    pub dx: f32,
    /// 今回のY移動量。
    pub dy: f32,
    /// 今回のZ移動量。
    pub dz: f32,
    /// ドラッグ開始時のX座標。
    pub origin_x: f32,
    /// ドラッグ開始時のY座標。
    pub origin_y: f32,
    /// ドラッグ開始時のZ座標。
    pub origin_z: f32,
    /// 自身のオブジェクト。
    pub object: crate::generic::ObjectHandle,
    /// 自身のエフェクト。
    pub effect: crate::generic::EffectHandle,
    /// エフェクト毎のユーザーデータ。
    pub userdata: FilterUserdataHandle<U>,
    pub(crate) read_section: crate::generic::ReadSection,
}

impl<U: FilterUserdata> FilterProcAnchor<U> {
    /// コールバック中に利用できる参照専用の編集セクション。
    pub fn read_section(&mut self) -> &crate::generic::ReadSection {
        &self.read_section
    }
}

define_bitflag! {
    /// アンカーポイントのフラグ。
    ///
    /// 現在は指定可能なフラグがありません。
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    pub struct AnchorMovePointFlag: i32 {}
}

define_bitflag! {
    /// アンカーラインのフラグ。
    ///
    /// 現在は指定可能なフラグがありません。
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    pub struct AnchorLineFlag: i32 {}
}

/// 映像処理中だけ利用できるアンカー操作ハンドル。
pub struct AnchorHandle<'a, U: FilterUserdata> {
    inner: &'a aviutl2_sys::filter2::ANCHOR_HANDLE,
    callbacks: &'a std::sync::Mutex<AnchorCallbackMap<U>>,
    effect_id: i64,
    callback: extern "C" fn(*mut aviutl2_sys::filter2::ANCHOR_SECTION),
}

impl<U: FilterUserdata> FilterProcVideo<U> {
    /// アンカーを設定できる場合に操作ハンドルを取得します。
    pub fn anchor(&mut self) -> Option<AnchorHandle<'_, U>> {
        // SAFETY: SDKのハンドルは現在の映像処理中に有効です。
        unsafe { (*self.inner).anchor.as_ref() }.map(|inner| AnchorHandle {
            inner,
            callback: self.proc_anchor,
            callbacks: &self.anchor_callbacks,
            effect_id: self.object.effect_id,
        })
    }
}

impl<U: FilterUserdata> AnchorHandle<'_, U> {
    ///ドラッグ移動可能なアンカーのポイントを設定する。
    ///
    /// # Arguments
    ///
    /// - `index`: アンカーのインデックス。アンカーを複数登録する場合に使用します。
    /// - `position`: アンカーの座標。オブジェクト座標系で指定します。
    /// - `flag`: アンカーのフラグ。現在は`Default::default()`を指定します。
    /// - `callback`: アンカーがドラッグされたときに呼び出されるコールバック関数。
    ///
    /// 現在、`callback`内では汎用データ項目を更新できます。
    pub fn set_anchor_move_point(
        &mut self,
        index: i32,
        position: [f32; 3],
        flag: AnchorMovePointFlag,
        callback: impl Fn(
            &[crate::filter::FilterConfigItem],
            &mut FilterProcAnchor<U>,
        ) -> crate::AnyResult<()>
        + Send
        + Sync
        + 'static,
    ) {
        let callback = std::sync::Arc::new(callback) as std::sync::Arc<Callback<U>>;
        let slot = {
            let mut callbacks = self.callbacks.lock().unwrap();
            std::sync::Arc::clone(callbacks.entry((self.effect_id, index)).or_insert_with(|| {
                std::sync::Arc::new(AnchorCallback {
                    callback: std::sync::RwLock::new(std::sync::Arc::clone(&callback)),
                })
            }))
        };
        let previous = std::mem::replace(&mut *slot.callback.write().unwrap(), callback);
        drop(previous);
        let param = std::sync::Arc::as_ptr(&slot).cast_mut().cast();

        unsafe {
            (self.inner.set_anchor_move_point)(
                index,
                position[0],
                position[1],
                position[2],
                flag.to_bits(),
                param,
                Some(self.callback),
            );
        }
    }

    /// アンカーのラインを設定する。
    ///
    /// `flag`には現在`Default::default()`を指定します。
    pub fn set_anchor_line(
        &mut self,
        start: [f32; 3],
        end: [f32; 3],
        flag: AnchorLineFlag,
        color: RgbaPixel,
    ) {
        unsafe {
            (self.inner.set_anchor_line)(
                start[0],
                start[1],
                start[2],
                end[0],
                end[1],
                end[2],
                flag.to_bits(),
                aviutl2_sys::filter2::PIXEL_RGBA {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color.a,
                },
            );
        }
    }
}

type Callback<U> = dyn Fn(&[crate::filter::FilterConfigItem], &mut FilterProcAnchor<U>) -> crate::AnyResult<()>
    + Send
    + Sync;

pub(crate) struct AnchorCallback<U: FilterUserdata> {
    pub(crate) callback: std::sync::RwLock<std::sync::Arc<Callback<U>>>,
}

impl<U: FilterUserdata> std::fmt::Debug for AnchorCallback<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnchorCallback").finish_non_exhaustive()
    }
}

// shortcut: スロットは終了まで保持する。SDKにアンカー破棄通知が追加されたら個別解放する。
type AnchorCallbackMap<U> =
    std::collections::HashMap<(i64, i32), std::sync::Arc<AnchorCallback<U>>>;
pub(crate) type AnchorCallbacks<U> = std::sync::Arc<std::sync::Mutex<AnchorCallbackMap<U>>>;
