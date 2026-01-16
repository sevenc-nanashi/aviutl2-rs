use std::num::NonZero;

use crate::common::Rational32;

/// オブジェクトへのハンドル。
#[derive(Debug, Clone, Copy)]
pub struct ObjectHandle {
    pub(crate) internal: aviutl2_sys::plugin2::OBJECT_HANDLE,
}
impl From<aviutl2_sys::plugin2::OBJECT_HANDLE> for ObjectHandle {
    fn from(value: aviutl2_sys::plugin2::OBJECT_HANDLE) -> Self {
        Self { internal: value }
    }
}
impl From<ObjectHandle> for aviutl2_sys::plugin2::OBJECT_HANDLE {
    fn from(value: ObjectHandle) -> Self {
        value.internal
    }
}

// 動いたし、このObjectHandleをグローバルに持っておくことも想定されてそうなので多分大丈夫なはず
unsafe impl Send for ObjectHandle {}
unsafe impl Sync for ObjectHandle {}

/// 編集情報構造体。
///
/// # Note
///
/// UI表示と異なり、フレーム番号・レイヤー番号は0始まりです。
#[derive(Debug, Clone, Copy)]
pub struct EditInfo {
    /// シーンの幅。
    pub width: usize,
    /// シーンの高さ。
    pub height: usize,
    /// フレームレート。
    pub fps: Rational32,
    /// サンプルレート。
    pub sample_rate: usize,
    /// 現在のカーソルのフレーム番号。
    pub frame: usize,
    /// 現在の選択レイヤー番号。
    pub layer: usize,
    /// オブジェクトが存在する最大のフレーム番号。
    pub frame_max: usize,
    /// オブジェクトが存在する最大のレイヤー番号。
    pub layer_max: usize,
    /// レイヤー編集で表示されているフレームの開始番号。
    pub display_frame_start: usize,
    /// レイヤー編集で表示されているレイヤーの開始番号。
    pub display_layer_start: usize,
    /// レイヤー編集で表示されているフレーム数。
    ///
    /// # Note
    ///
    /// この値は厳密な値ではありません。
    pub display_frame_num: usize,
    /// レイヤー編集で表示されているレイヤー数
    ///
    /// # Note
    ///
    /// この値は厳密な値ではありません。
    pub display_layer_num: usize,
    /// フレーム範囲選択の開始フレーム番号
    /// 未選択の場合は`None`になります。
    pub select_range_start: Option<usize>,
    /// フレーム範囲選択の終了フレーム番号。
    /// 未選択の場合は`None`になります。
    pub select_range_end: Option<usize>,
    /// グリッド（BPM）のテンポ。
    pub grid_bpm_tempo: f32,
    /// グリッド（BPM）の拍数。
    pub grid_bpm_beat: usize,
    /// グリッド（BPM）の基準時間。
    pub grid_bpm_offset: f32,
    /// シーンのID
    pub scene_id: i32,
}

impl EditInfo {
    /// # Safety
    ///
    /// `ptr`は有効な`EDIT_INFO`ポインタである必要があります。
    pub unsafe fn from_raw(ptr: *const aviutl2_sys::plugin2::EDIT_INFO) -> Self {
        let raw = unsafe { &*ptr };
        Self {
            width: raw.width as usize,
            height: raw.height as usize,
            fps: Rational32::new(raw.rate, raw.scale),
            sample_rate: raw.sample_rate as usize,
            frame: raw.frame as usize,
            layer: raw.layer as usize,
            frame_max: raw.frame_max as usize,
            layer_max: raw.layer_max as usize,
            display_frame_start: raw.display_frame_start as usize,
            display_layer_start: raw.display_layer_start as usize,
            display_frame_num: raw.display_frame_num as usize,
            display_layer_num: raw.display_layer_num as usize,

            // `as usize` はpanicしないけど一応気分的に安全にしておく、それはそうと
            // clippyはこれをunnecessary_lazy_evaluationsと判断するので無視する
            #[expect(clippy::unnecessary_lazy_evaluations)]
            select_range_start: (raw.select_range_start >= 0)
                .then(|| raw.select_range_start as usize),
            #[expect(clippy::unnecessary_lazy_evaluations)]
            select_range_end: (raw.select_range_end >= 0).then(|| raw.select_range_end as usize),

            grid_bpm_tempo: raw.grid_bpm_tempo,
            grid_bpm_beat: raw.grid_bpm_beat as usize,
            grid_bpm_offset: raw.grid_bpm_offset,
            scene_id: raw.scene_id,
        }
    }
}

/// オブジェクトのレイヤーとフレーム情報。
#[derive(Debug, Clone, Copy)]
pub struct ObjectLayerFrame {
    pub layer: usize,
    pub start: usize,
    pub end: usize,
}

/// レイヤーとフレーム情報。
#[derive(Debug, Clone, Copy)]
pub struct LayerFrameData {
    pub layer: usize,
    pub frame: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MediaInfo {
    /// Videoトラック数。
    pub video_track_num: Option<NonZero<usize>>,
    /// Audioトラック数。
    pub audio_track_num: Option<NonZero<usize>>,
    /// 総時間（秒）。
    pub total_time: f64,
    /// 解像度の幅。
    pub width: usize,
    /// 解像度の高さ。
    pub height: usize,
}

/// [`EditSection::is_support_media_file`] のモード。
#[derive(Debug, Clone, Copy)]
pub enum MediaFileSupportMode {
    /// 拡張子が対応しているかどうかのみを確認します。
    ExtensionOnly,
    /// 実際にファイルを開いて対応しているかどうかを確認します。
    Strict,
}

/// [`EditSection`] 関連のエラー。
#[derive(thiserror::Error, Debug)]
pub enum EditSectionError {
    #[error("api call failed")]
    ApiCallFailed,
    #[error("object does not exist")]
    ObjectDoesNotExist,
    #[error("input utf-8 string contains null byte")]
    InputCstrContainsNull(#[from] std::ffi::NulError),
    #[error("input utf-16 string contains null byte")]
    InputCwstrContainsNull(#[from] crate::common::NullByteError),
    #[error("value is out of range")]
    ValueOutOfRange(#[from] std::num::TryFromIntError),
    #[error("api returned non-utf8 data")]
    NonUtf8Data(#[from] std::str::Utf8Error),

    #[cfg(feature = "aviutl2-alias")]
    #[error("alias parse error: {0}")]
    ParseFailed(#[from] aviutl2_alias::TableParseError),
}

pub type EditSectionResult<T> = Result<T, EditSectionError>;

/// 編集セクションのハンドル。
#[derive(Debug)]
pub struct EditSection {
    /// 編集情報。
    pub info: EditInfo,

    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
}

impl EditSection {
    /// 生ポインタから `EditSection` を作成します。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_raw(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self {
            internal: ptr,
            info: unsafe { EditInfo::from_raw((*ptr).info) },
        }
    }

    /// オブジェクトエイリアスから指定の位置にオブジェクトを作成します。
    ///
    /// # Arguments
    ///
    /// - `alias`：オブジェクトエイリアスのデータ。オブジェクトエイリアスと同じフォーマットで指定します。
    /// - `layer`：作成するオブジェクトのレイヤー番号（0始まり）。
    /// - `frame`：作成するオブジェクトのフレーム番号（0始まり）。
    /// - `length`：作成するオブジェクトの長さ（フレーム数）。
    ///
    /// lengthはエイリアスデータにフレーム情報が無い場合に利用されます。
    ///
    /// # Errors
    ///
    /// エイリアスの変換に失敗した場合、またはオブジェクトが既存のオブジェクトに重なる場合にエラー
    ///
    pub fn create_object_from_alias(
        &self,
        alias: &str,
        layer: usize,
        frame: usize,
        length: usize,
    ) -> EditSectionResult<ObjectHandle> {
        let c_alias = std::ffi::CString::new(alias)?;
        let object_handle = unsafe {
            ((*self.internal).create_object_from_alias)(
                c_alias.as_ptr(),
                layer.try_into()?,
                frame.try_into()?,
                length.try_into()?,
            )
        };
        if object_handle.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(ObjectHandle {
            internal: object_handle,
        })
    }

    /// 指定のフレーム番号以降にあるオブジェクトを検索します。
    ///
    /// # Arguments
    ///
    /// - `layer`：検索するレイヤー番号（0始まり）。
    /// - `frame`：検索を開始するフレーム番号（0始まり）。
    pub fn find_object_after(
        &self,
        layer: usize,
        frame: usize,
    ) -> EditSectionResult<Option<ObjectHandle>> {
        let object_handle =
            unsafe { ((*self.internal).find_object)(layer.try_into()?, frame.try_into()?) };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle {
                internal: object_handle,
            }))
        }
    }

    /// オブジェクトに対象エフェクトが何個存在するかを取得します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect`：対象のエフェクト名。（エイリアスファイルの effect.name の値）
    ///
    /// # Returns
    ///
    /// 対象エフェクトの数。存在しない場合は0を返します。
    pub fn count_object_effect(
        &self,
        object: &ObjectHandle,
        effect: &str,
    ) -> EditSectionResult<usize> {
        self.ensure_object_exists(object)?;
        let c_effect = crate::common::CWString::new(effect)?;
        let count =
            unsafe { ((*self.internal).count_object_effect)(object.internal, c_effect.as_ptr()) };
        Ok(count.try_into()?)
    }

    /// 指定のオブジェクトのレイヤーとフレーム情報を取得します。
    pub fn get_object_layer_frame(
        &self,
        object: &ObjectHandle,
    ) -> EditSectionResult<ObjectLayerFrame> {
        self.ensure_object_exists(object)?;
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        Ok(ObjectLayerFrame {
            layer: object.layer.try_into()?,
            start: object.start.try_into()?,
            end: object.end.try_into()?,
        })
    }

    /// オブジェクトの情報をエイリアスデータとして取得します。
    pub fn get_object_alias(&self, object: &ObjectHandle) -> EditSectionResult<String> {
        self.ensure_object_exists(object)?;
        let alias_ptr = unsafe { ((*self.internal).get_object_alias)(object.internal) };
        if alias_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(alias_ptr) };
        let alias = c_str.to_str()?.to_owned();
        Ok(alias)
    }

    /// オブジェクト名を取得します。
    ///
    /// # Returns
    ///
    /// 標準の名前の場合は`None`を返します。
    pub fn get_object_name(&self, object: &ObjectHandle) -> EditSectionResult<Option<String>> {
        self.ensure_object_exists(object)?;
        let name_ptr = unsafe { ((*self.internal).get_object_name)(object.internal) };
        if name_ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { crate::common::load_wide_string(name_ptr) }))
    }

    /// オブジェクト名を設定します。
    ///
    /// # Note
    ///
    /// `name`に`None`や空文字を指定すると、標準の名前になります。
    pub fn set_object_name(
        &self,
        object: &ObjectHandle,
        name: Option<&str>,
    ) -> EditSectionResult<()> {
        self.ensure_object_exists(object)?;
        match name {
            None => {
                unsafe { ((*self.internal).set_object_name)(object.internal, std::ptr::null()) };
                Ok(())
            }
            Some(name) => {
                let c_name = crate::common::CWString::new(name)?;
                unsafe {
                    ((*self.internal).set_object_name)(object.internal, c_name.as_ptr());
                }
                Ok(())
            }
        }
    }

    /// オブジェクトの設定項目の値を文字列で取得します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    pub fn get_object_effect_item(
        &self,
        object: &ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> EditSectionResult<String> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(&effect_name, effect_index))?;
        let c_item = crate::common::CWString::new(item)?;
        let value_ptr = unsafe {
            ((*self.internal).get_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
            )
        };
        if value_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
        let value = c_str.to_str()?.to_owned();
        Ok(value)
    }

    /// オブジェクトの設定項目の値を文字列で設定します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    /// - `value`：設定する値。
    pub fn set_object_effect_item(
        &self,
        object: &ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        value: &str,
    ) -> EditSectionResult<()> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(&effect_name, effect_index))?;
        let c_item = crate::common::CWString::new(item)?;
        let c_value = std::ffi::CString::new(value)?;
        let success = unsafe {
            ((*self.internal).set_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                c_value.as_ptr(),
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(())
    }

    /// オブジェクトを移動します。
    pub fn move_object(
        &self,
        object: &ObjectHandle,
        new_layer: usize,
        new_start_frame: usize,
    ) -> EditSectionResult<()> {
        self.ensure_object_exists(object)?;
        let success = unsafe {
            ((*self.internal).move_object)(
                object.internal,
                new_layer.try_into()?,
                new_start_frame.try_into()?,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(())
    }

    /// オブジェクトを削除します。
    pub fn delete_object(&self, object: &ObjectHandle) -> EditSectionResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).delete_object)(object.internal) };
        Ok(())
    }

    /// 現在、オブジェクト設定ウィンドウで選択されているオブジェクトを取得します。
    pub fn get_focused_object(&self) -> EditSectionResult<Option<ObjectHandle>> {
        let object_handle = unsafe { ((*self.internal).get_focus_object)() };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle {
                internal: object_handle,
            }))
        }
    }

    /// 現在選択されているオブジェクトの一覧を取得します。
    pub fn get_selected_objects(&self) -> EditSectionResult<Vec<ObjectHandle>> {
        let mut handles = Vec::new();
        let num_objects = unsafe { ((*self.internal).get_selected_object_num)() };
        for i in 0..num_objects {
            let object_handle = unsafe { ((*self.internal).get_selected_object)(i) };
            if object_handle.is_null() {
                return Err(EditSectionError::ApiCallFailed);
            }
            handles.push(ObjectHandle {
                internal: object_handle,
            });
        }
        Ok(handles)
    }

    /// オブジェクト設定ウィンドウで指定のオブジェクトを選択状態にします。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self, object: &ObjectHandle) -> EditSectionResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).set_focus_object)(object.internal) };
        Ok(())
    }

    /// プロジェクトファイルのポインタを取得します。
    pub fn get_project_file<'handle>(
        &'handle self,
        edit_handle: &crate::generic::EditHandle,
    ) -> crate::generic::ProjectFile<'handle> {
        let pf_ptr = unsafe { ((*self.internal).get_project_file)(edit_handle.internal) };
        unsafe { crate::generic::ProjectFile::from_raw(pf_ptr) }
    }

    /// マウス座標のレイヤー・フレーム位置を取得します。
    ///
    /// # Returns
    ///
    /// マウスがレイヤー編集エリア内にある場合は `Some` を返し、
    /// そうでない場合は `None` を返します。
    pub fn get_mouse_layer_frame(&self) -> EditSectionResult<Option<LayerFrameData>> {
        let mut layer = 0;
        let mut frame = 0;
        let on_layer_edit =
            unsafe { ((*self.internal).get_mouse_layer_frame)(&mut layer, &mut frame) };
        if on_layer_edit {
            Ok(Some(LayerFrameData {
                layer: layer.try_into()?,
                frame: frame.try_into()?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 指定のスクリーン座標のレイヤー・フレーム位置を取得します。
    pub fn pos_to_layer_frame(&self, x: i32, y: i32) -> EditSectionResult<Option<LayerFrameData>> {
        let mut layer = 0;
        let mut frame = 0;
        let on_layer_edit =
            unsafe { ((*self.internal).pos_to_layer_frame)(x, y, &mut layer, &mut frame) };
        if on_layer_edit {
            Ok(Some(LayerFrameData {
                layer: layer.try_into()?,
                frame: frame.try_into()?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 指定のメディアファイルがサポートされているかどうか調べます。
    pub fn is_support_media_file<P: AsRef<str>>(
        &self,
        file_path: P,
        mode: MediaFileSupportMode,
    ) -> EditSectionResult<bool> {
        let c_file_path = crate::common::CWString::new(file_path.as_ref())?;
        let is_supported = unsafe {
            match mode {
                MediaFileSupportMode::ExtensionOnly => {
                    ((*self.internal).is_support_media_file)(c_file_path.as_ptr(), false)
                }
                MediaFileSupportMode::Strict => {
                    ((*self.internal).is_support_media_file)(c_file_path.as_ptr(), true)
                }
            }
        };
        Ok(is_supported)
    }

    /// 指定のメディアファイルの情報を取得します。
    ///
    /// # Note
    ///
    /// 動画、音声、画像ファイル以外では取得出来ません。
    pub fn get_media_info<P: AsRef<str>>(&self, file_path: P) -> EditSectionResult<MediaInfo> {
        let c_file_path = crate::common::CWString::new(file_path.as_ref())?;
        let mut media_info = std::mem::MaybeUninit::<aviutl2_sys::plugin2::MEDIA_INFO>::uninit();
        let success = unsafe {
            ((*self.internal).get_media_info)(
                c_file_path.as_ptr(),
                media_info.as_mut_ptr(),
                std::mem::size_of::<aviutl2_sys::plugin2::MEDIA_INFO>() as i32,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        let media_info = unsafe { media_info.assume_init() };
        Ok(MediaInfo {
            video_track_num: NonZero::new(media_info.video_track_num.try_into()?),
            audio_track_num: NonZero::new(media_info.audio_track_num.try_into()?),
            total_time: media_info.total_time,
            width: media_info.width.try_into()?,
            height: media_info.height.try_into()?,
        })
    }

    /// 指定の位置にメディアファイルからオブジェクトを作成します。
    ///
    /// # Arguments
    ///
    /// - `file_path`：メディアファイルのパス。
    /// - `layer`：作成するオブジェクトのレイヤー番号（0始まり）。
    /// - `frame`：作成するオブジェクトのフレーム番号（0始まり）。
    /// - `length`：作成するオブジェクトの長さ（フレーム数）。`None`を指定した場合、長さや追加位置は自動的に調整されます。
    pub fn create_object_from_media_file<P: AsRef<str>>(
        &self,
        file_path: P,
        layer: usize,
        frame: usize,
        length: Option<usize>,
    ) -> EditSectionResult<ObjectHandle> {
        let c_file_path = crate::common::CWString::new(file_path.as_ref())?;
        let object_handle = unsafe {
            ((*self.internal).create_object_from_media_file)(
                c_file_path.as_ptr(),
                layer.try_into()?,
                frame.try_into()?,
                length.unwrap_or(0).try_into()?,
            )
        };
        if object_handle.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(ObjectHandle {
            internal: object_handle,
        })
    }

    /// 指定の位置にオブジェクトを作成します。
    ///
    /// # Arguments
    ///
    /// - `effect`：エフェクト名。（エイリアスファイルの effect.name の値）
    /// - `layer`：作成するオブジェクトのレイヤー番号（0始まり）。
    /// - `frame`：作成するオブジェクトのフレーム番号（0始まり）。
    /// - `length`：作成するオブジェクトの長さ（フレーム数）。`None`を指定した場合、長さや追加位置は自動的に調整されます。
    pub fn create_object(
        &self,
        effect: &str,
        layer: usize,
        frame: usize,
        length: Option<usize>,
    ) -> EditSectionResult<ObjectHandle> {
        let c_effect = crate::common::CWString::new(effect)?;
        let object_handle = unsafe {
            ((*self.internal).create_object)(
                c_effect.as_ptr(),
                layer.try_into()?,
                frame.try_into()?,
                length.unwrap_or(0).try_into()?,
            )
        };
        if object_handle.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(ObjectHandle {
            internal: object_handle,
        })
    }

    /// 現在のレイヤー・フレーム位置を設定します。
    ///
    /// # Note
    ///
    /// 設定出来る範囲に調整されます。
    pub fn set_cursor_layer_frame(&self, layer: usize, frame: usize) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_cursor_layer_frame)(layer.try_into()?, frame.try_into()?);
        }
        Ok(())
    }

    /// レイヤー編集のレイヤー・フレームの表示開始位置を設定します。
    ///
    /// # Note
    ///
    /// 設定出来る範囲に調整されます。
    pub fn set_display_layer_frame(&self, layer: usize, frame: usize) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_display_layer_frame)(layer.try_into()?, frame.try_into()?);
        }
        Ok(())
    }

    /// フレーム範囲選択を設定します。
    ///
    /// # Note
    ///
    /// 設定出来る範囲に調整されます。
    pub fn set_select_range(&self, start: usize, end: usize) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_select_range)(start.try_into()?, end.try_into()?);
        }
        Ok(())
    }

    /// 選択されているフレーム範囲選択を解除します。
    pub fn clear_select_range(&self) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_select_range)(-1, -1);
        }
        Ok(())
    }

    /// グリッド（BPM）を設定します。
    pub fn set_grid_bpm(&self, tempo: f32, beat: usize, offset: f32) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_grid_bpm)(tempo, beat.try_into()?, offset);
        }
        Ok(())
    }

    /// オブジェクトが存在するかどうか調べます。
    ///
    /// # Note
    ///
    /// 内部的には、get_object_layer_frame を呼び出してレイヤー番号が -1 でないかを確認しています。
    pub fn object_exists(&self, object: &ObjectHandle) -> bool {
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        object.layer != -1
    }

    fn ensure_object_exists(&self, object: &ObjectHandle) -> EditSectionResult<()> {
        if !self.object_exists(object) {
            return Err(EditSectionError::ObjectDoesNotExist);
        }
        Ok(())
    }

    /// すべてのレイヤーをイテレータで取得します。
    pub fn layers(&self) -> EditSectionLayersIterator<'_> {
        EditSectionLayersIterator::new(self)
    }

    /// [EditSectionLayerCaller] を作成します。
    pub fn layer<'a>(&'a self, layer: usize) -> EditSectionLayerCaller<'a> {
        EditSectionLayerCaller::new(self, layer)
    }
    /// [EditSectionObjectCaller] を作成します。
    pub fn object<'a>(&'a self, object: &'a ObjectHandle) -> EditSectionObjectCaller<'a> {
        EditSectionObjectCaller::new(self, object)
    }
}

/// オブジェクト主体で関数を呼び出すための構造体。
/// EditSection と ObjectHandle の組をまとめ、対象オブジェクトに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionObjectCaller<'a> {
    edit_section: &'a EditSection,
    pub handle: &'a ObjectHandle,
}
impl<'a> EditSectionObjectCaller<'a> {
    pub fn new(edit_section: &'a EditSection, object: &'a ObjectHandle) -> Self {
        Self {
            edit_section,
            handle: object,
        }
    }

    /// オブジェクトのレイヤーとフレーム情報を取得します。
    pub fn get_layer_frame(&self) -> EditSectionResult<ObjectLayerFrame> {
        self.edit_section.get_object_layer_frame(self.handle)
    }

    /// オブジェクトの情報をエイリアスデータとして取得します。
    pub fn get_alias(&self) -> EditSectionResult<String> {
        self.edit_section.get_object_alias(self.handle)
    }

    /// オブジェクトの情報をエイリアスデータとして取得し、パースします。
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_alias_parsed(&self) -> EditSectionResult<aviutl2_alias::Table> {
        self.edit_section
            .get_object_alias(self.handle)?
            .parse()
            .map_err(Into::into)
    }

    /// オブジェクトに対象エフェクトが何個存在するかを取得します。
    ///
    /// # Arguments
    ///
    /// - `effect`：対象のエフェクト名。（エイリアスファイルの effect.name の値）
    ///
    /// # Returns
    ///
    /// 対象エフェクトの数。存在しない場合は0を返します。
    pub fn count_effect(&self, effect: &str) -> EditSectionResult<usize> {
        self.edit_section.count_object_effect(self.handle, effect)
    }

    /// オブジェクトの設定項目の値を文字列で取得します。
    ///
    /// # Arguments
    ///
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    pub fn get_effect_item(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> EditSectionResult<String> {
        self.edit_section
            .get_object_effect_item(self.handle, effect_name, effect_index, item)
    }

    /// オブジェクトの設定項目の値を文字列で設定します。
    ///
    /// # Arguments
    ///
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    /// - `value`：設定する値。
    pub fn set_effect_item(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        value: &str,
    ) -> EditSectionResult<()> {
        self.edit_section.set_object_effect_item(
            self.handle,
            effect_name,
            effect_index,
            item,
            value,
        )
    }

    /// オブジェクトを移動します。
    ///
    /// # Arguments
    ///
    /// - `new_layer`：移動先のレイヤー番号（0始まり）。
    /// - `new_start_frame`：移動先の開始フレーム番号（0始まり）。
    pub fn move_object(&self, new_layer: usize, new_start_frame: usize) -> EditSectionResult<()> {
        self.edit_section
            .move_object(self.handle, new_layer, new_start_frame)
    }

    /// オブジェクトを削除します。
    pub fn delete_object(&self) -> EditSectionResult<()> {
        self.edit_section.delete_object(self.handle)
    }

    /// オブジェクト設定ウィンドウでこのオブジェクトを選択状態にします。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self) -> EditSectionResult<()> {
        self.edit_section.focus_object(self.handle)
    }

    /// このオブジェクトが存在するかどうか調べます。
    pub fn exists(&self) -> bool {
        self.edit_section.object_exists(self.handle)
    }
}

/// レイヤー主体で関数を呼び出すための構造体。
/// EditSection と レイヤー番号 の組をまとめ、対象レイヤーに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionLayerCaller<'a> {
    edit_section: &'a EditSection,
    pub index: usize,
}
impl<'a> EditSectionLayerCaller<'a> {
    pub fn new(edit_section: &'a EditSection, layer: usize) -> Self {
        Self {
            edit_section,
            index: layer,
        }
    }

    /// 指定のフレーム番号以降にあるオブジェクトを検索します。
    ///
    /// # Arguments
    ///
    /// - `frame`：検索を開始するフレーム番号（0始まり）。
    pub fn find_object_after(&self, frame: usize) -> EditSectionResult<Option<ObjectHandle>> {
        self.edit_section.find_object_after(self.index, frame)
    }

    /// 指定の位置にメディアファイルからオブジェクトを作成します。
    ///
    /// # See Also
    ///
    /// [`EditSection::create_object_from_media_file`]
    pub fn create_object_from_media_file<P: AsRef<str>>(
        &self,
        file_path: P,
        frame: usize,
        length: Option<usize>,
    ) -> EditSectionResult<ObjectHandle> {
        self.edit_section
            .create_object_from_media_file(file_path, self.index, frame, length)
    }

    /// 指定の位置にオブジェクトを作成します。
    ///
    /// # See Also
    ///
    /// [`EditSection::create_object`]
    pub fn create_object(
        &self,
        effect: &str,
        frame: usize,
        length: Option<usize>,
    ) -> EditSectionResult<ObjectHandle> {
        self.edit_section
            .create_object(effect, self.index, frame, length)
    }

    /// このレイヤーに存在するすべてのオブジェクトを、
    /// 開始フレームの昇順で走査するイテレータを返します。
    pub fn objects(&self) -> EditSectionLayerObjectsIterator<'a> {
        EditSectionLayerObjectsIterator::new(self.edit_section, self.index)
    }
}

/// レイヤーのイテレータ。
#[derive(Debug, Clone)]
pub struct EditSectionLayersIterator<'a> {
    edit_section: &'a EditSection,
    current: usize,
    total: usize,
}

impl<'a> EditSectionLayersIterator<'a> {
    fn new(edit_section: &'a EditSection) -> Self {
        Self {
            edit_section,
            current: 0,
            total: edit_section.info.layer_max,
        }
    }
}

impl<'a> Iterator for EditSectionLayersIterator<'a> {
    type Item = EditSectionLayerCaller<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.total {
            return None;
        }
        let layer = self.current;
        self.current += 1;
        Some(EditSectionLayerCaller::new(self.edit_section, layer))
    }
}

/// レイヤー内のオブジェクトを走査するイテレータ。
/// アイテムは `(オブジェクトのレイヤー・フレーム情報, ハンドル)` の組です。
#[derive(Debug, Clone)]
pub struct EditSectionLayerObjectsIterator<'a> {
    edit_section: &'a EditSection,
    layer: usize,
    next_frame: usize,
}

impl<'a> EditSectionLayerObjectsIterator<'a> {
    fn new(edit_section: &'a EditSection, layer: usize) -> Self {
        Self {
            edit_section,
            layer,
            next_frame: 0,
        }
    }
}

impl<'a> Iterator for EditSectionLayerObjectsIterator<'a> {
    type Item = (ObjectLayerFrame, ObjectHandle);

    fn next(&mut self) -> Option<Self::Item> {
        // 検索・取得でエラーが出た場合は None を返して終了する。
        let Ok(Some(handle)) = self
            .edit_section
            .find_object_after(self.layer, self.next_frame)
        else {
            return None;
        };

        let lf = match self.edit_section.get_object_layer_frame(&handle) {
            Ok(lf) => lf,
            Err(_) => return None,
        };

        // 次の検索開始位置を、いま見つかったオブジェクトの末尾+1 に進める。
        self.next_frame = lf.end.saturating_add(1);

        Some((lf, handle))
    }
}

fn effect_key(effect_name: &str, effect_index: usize) -> String {
    format!("{effect_name}:{effect_index}")
}
