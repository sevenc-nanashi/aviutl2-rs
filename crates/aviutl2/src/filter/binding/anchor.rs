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
    pub(crate) inner: *mut aviutl2_sys::filter2::ANCHOR_SECTION,
    pub(crate) config_items: *const [crate::filter::FilterConfigItem],
    pub(crate) config_pointers: *const [*const aviutl2_sys::filter2::FILTER_ITEM],
}

impl<U: FilterUserdata> FilterProcAnchor<U> {
    /// 指定のトラックバー項目の値を設定する。
    ///
    /// # Arguments
    ///
    /// - `item_name`: トラックバー項目の名前。
    /// - `index`: 何番目の値を設定するか。例えば始点の値を設定する場合は0になります。
    /// - `value`: 設定する値。
    ///
    /// # Panics
    ///
    /// `item_name`に対応するトラックバー項目が見つからなかった場合にパニックします。
    ///
    /// # See Also
    ///
    /// [`crate::filter_config_field_name`]
    pub fn set_filter_item_track_value(&mut self, item_name: &str, index: usize, value: f32) {
        // SAFETY: bridge側でこの値が生存していることが保証されている
        let (items, pointers) = unsafe { (&*self.config_items, &*self.config_pointers) };
        let candidates = items
            .iter()
            .filter(|item| item.name() == item_name)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            panic!("Unknown config item name: {}", item_name);
        }
        let position = items
            .iter()
            .position(|item| matches!(item, crate::filter::FilterConfigItem::Track(track) if track.name == item_name))
            .unwrap_or_else(|| panic!("The config item is not a track: {}", item_name));

        let track = pointers[position]
            .cast::<aviutl2_sys::filter2::FILTER_ITEM_TRACK>()
            .cast_mut();
        unsafe { ((*self.inner).set_filter_item_track_value)(track.cast(), index as _, value) };
    }

    /// コールバック中に利用できる参照専用の編集セクション。
    pub fn read_section(&mut self) -> &crate::generic::ReadSection {
        &self.read_section
    }
}

define_bitflag! {
    /// アンカーのフラグ。
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    pub struct AnchorFlag: i32 {
        /// アンカーの座標をスクリーン座標系で指定する。
        screen_position: aviutl2_sys::filter2::ANCHOR_HANDLE::FLAG_SCREEN_POSITION,
        /// 小さいアンカーポイントにする。
        ///
        /// ドラッグ移動可能なアンカーを設定する場合にのみ有効です。
        small_point: aviutl2_sys::filter2::ANCHOR_HANDLE::FLAG_SMALL_POINT,
    }
}

/// 映像処理中だけ利用できるアンカー操作ハンドル。
pub struct AnchorHandle<'a, U: FilterUserdata> {
    inner: &'a aviutl2_sys::filter2::ANCHOR_HANDLE,
    callback: extern "C" fn(*mut aviutl2_sys::filter2::ANCHOR_SECTION),
    _phantom: std::marker::PhantomData<U>,
}

impl<U: FilterUserdata> FilterProcVideo<U> {
    /// アンカーを設定できる場合に操作ハンドルを取得します。
    pub fn anchor(&mut self) -> Option<AnchorHandle<'_, U>> {
        unsafe { (*self.inner).anchor.as_ref() }.map(|inner| AnchorHandle {
            inner,
            callback: self.proc_anchor,
            _phantom: std::marker::PhantomData,
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
    /// - `flag`: アンカーのフラグ。
    /// - `callback`: アンカーがドラッグされたときに呼び出されるコールバック関数。
    ///
    /// ドラッグ中はドラッグが終わるまで同じコールバック関数が呼び出されます。
    pub fn set_anchor_move_point<F>(
        &mut self,
        index: i32,
        position: [f32; 3],
        flag: AnchorFlag,
        callback: F,
    ) where
        F: (Fn(&[crate::filter::FilterConfigItem], &mut FilterProcAnchor<U>) -> crate::AnyResult<()>)
            + Copy
            + Send
            + Sync
            + 'static,
    {
        unsafe fn invoke<F, U: FilterUserdata>(
            param: *const std::ffi::c_void,
            config: &[crate::filter::FilterConfigItem],
            anchor: &mut FilterProcAnchor<U>,
        ) -> crate::AnyResult<()>
        where
            F: Fn(
                    &[crate::filter::FilterConfigItem],
                    &mut FilterProcAnchor<U>,
                ) -> crate::AnyResult<()>
                + Copy,
        {
            let param = unsafe { param.cast::<AnchorCallbackParam<F, U>>().read_unaligned() };
            (param.callback)(config, anchor)
        }
        let param = AnchorCallbackParam {
            header: AnchorCallbackHeader {
                invoke: invoke::<F, U>,
            },
            callback,
        };
        let param_size = std::mem::size_of_val(&param);
        unsafe {
            (self.inner.set_anchor_move_point_param)(
                index,
                position[0],
                position[1],
                position[2],
                flag.to_bits(),
                std::ptr::from_ref(&param).cast_mut().cast(),
                param_size as _,
                Some(self.callback),
            );
        }
    }

    /// アンカーのラインを設定する。
    pub fn set_anchor_line(
        &mut self,
        start: [f32; 3],
        end: [f32; 3],
        flag: AnchorFlag,
        color: Option<RgbaPixel>,
    ) {
        if let Some(color) = color {
            unsafe {
                (self.inner.set_anchor_line_color)(
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
        } else {
            unsafe {
                (self.inner.set_anchor_line)(
                    start[0],
                    start[1],
                    start[2],
                    end[0],
                    end[1],
                    end[2],
                    flag.to_bits(),
                );
            }
        }
    }

    /// アンカーの点を設定する。
    pub fn set_anchor_point(
        &mut self,
        position: [f32; 3],
        flag: AnchorFlag,
        color: Option<RgbaPixel>,
    ) {
        if let Some(color) = color {
            unsafe {
                (self.inner.set_anchor_point_color)(
                    position[0],
                    position[1],
                    position[2],
                    flag.to_bits(),
                    aviutl2_sys::filter2::PIXEL_RGBA {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: color.a,
                    },
                );
            }
        } else {
            unsafe {
                (self.inner.set_anchor_point)(
                    position[0],
                    position[1],
                    position[2],
                    flag.to_bits(),
                );
            }
        }
    }
}

#[repr(C)]
pub struct AnchorCallbackHeader<U: FilterUserdata> {
    pub invoke: unsafe fn(
        *const std::ffi::c_void,
        &[crate::filter::FilterConfigItem],
        &mut FilterProcAnchor<U>,
    ) -> crate::AnyResult<()>,
}

#[repr(C)]
struct AnchorCallbackParam<F, U: FilterUserdata> {
    header: AnchorCallbackHeader<U>,
    callback: F,
}
