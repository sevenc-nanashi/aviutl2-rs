use std::num::NonZero;

use crate::common::Rational32;

/// オブジェクトへのハンドル。
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectHandle {
    pub(crate) internal: aviutl2_sys::plugin2::OBJECT_HANDLE,
}
impl std::fmt::Debug for ObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ObjectHandle").field(&self.internal).finish()
    }
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

/// エフェクトへのハンドル。
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct EffectHandle {
    pub(crate) internal: aviutl2_sys::plugin2::EFFECT_HANDLE,
}
impl std::fmt::Debug for EffectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EffectHandle").field(&self.internal).finish()
    }
}
impl From<aviutl2_sys::plugin2::EFFECT_HANDLE> for EffectHandle {
    fn from(value: aviutl2_sys::plugin2::EFFECT_HANDLE) -> Self {
        Self { internal: value }
    }
}
impl From<EffectHandle> for aviutl2_sys::plugin2::EFFECT_HANDLE {
    fn from(value: EffectHandle) -> Self {
        value.internal
    }
}

/// 編集情報構造体。
///
/// # Note
///
/// UI表示と異なり、フレーム番号・レイヤー番号は0始まりです。
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
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

            select_range_start: (raw.select_range_start >= 0)
                .then_some(raw.select_range_start as usize),
            select_range_end: (raw.select_range_end >= 0).then_some(raw.select_range_end as usize),

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

impl ObjectLayerFrame {
    /// フレームの範囲をRangeで取得する。
    pub fn frame_range(&self) -> std::ops::Range<usize> {
        self.start..(self.end + 1)
    }

    /// フレームの範囲をRangeInclusiveで取得する。
    pub fn frame_range_inclusive(&self) -> std::ops::RangeInclusive<usize> {
        self.start..=self.end
    }
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

#[derive(Debug, Clone)]
pub struct TrackInfo {
    /// トラックバーの移動モードの名称。
    pub mode: String,
    /// トラックバーの設定値。
    pub params: Vec<f64>,
    /// トラックバーの加速度が有効かどうか。
    pub accelerate: bool,
    /// トラックバーの減速度が有効かどうか。
    pub decelerate: bool,
    /// トラックバーの中間点無視が有効かどうか。
    pub twopoint: bool,
    /// トラックバーの時間制御が有効かどうか。
    pub timecontrol: bool,
    /// トラックバーグループのトラック数。
    /// グループ化が解除されている場合は1になります。
    pub group_num: usize,
    /// トラックバーグループ内でのインデックス。
    pub group_index: usize,
    /// 所属トラックバーグループ名。
    pub group_name: Option<String>,
}

/// パレット情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteInfo {
    /// パレット色。
    pub colors: [PaletteColor; Self::PALETTE_NUM],
}

impl PaletteInfo {
    /// パレットの色数。
    pub const PALETTE_NUM: usize = aviutl2_sys::plugin2::PALETTE_INFO::PALETTE_NUM;

    fn from_raw(raw: aviutl2_sys::plugin2::PALETTE_INFO) -> Self {
        Self {
            colors: raw.color.map(|color| PaletteColor {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            }),
        }
    }
}

/// パレット色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaletteColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// [`ReadSection::is_support_media_file`] のモード。
#[derive(Debug, Clone, Copy)]
pub enum MediaFileSupportMode {
    /// 拡張子が対応しているかどうかのみを確認します。
    ExtensionOnly,
    /// 実際にファイルを開いて対応しているかどうかを確認します。
    Strict,
}

/// BPM情報。
///
/// # Note
///
/// PartialOrdを実装しています。start順で並び替えられ、同じstartの場合はNoneになります。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BpmInfo {
    /// テンポ。
    pub tempo: f32,
    /// 拍子。
    pub beat: i32,
    /// 開始位置(秒)。
    pub start: f64,
    /// 拍子オフセット(秒)。
    pub offset: f32,
}
impl PartialOrd for BpmInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.start == other.start {
            None
        } else {
            self.start.partial_cmp(&other.start)
        }
    }
}
impl From<aviutl2_sys::plugin2::BPM_INFO> for BpmInfo {
    fn from(value: aviutl2_sys::plugin2::BPM_INFO) -> Self {
        Self {
            tempo: value.tempo,
            beat: value.beat,
            start: value.start,
            offset: value.offset,
        }
    }
}
impl From<BpmInfo> for aviutl2_sys::plugin2::BPM_INFO {
    fn from(value: BpmInfo) -> Self {
        Self {
            tempo: value.tempo,
            beat: value.beat,
            start: value.start,
            offset: value.offset,
        }
    }
}

/// [`EditSection`] 関連のエラー。
#[derive(thiserror::Error, Debug)]
pub enum EditSectionError {
    #[error("api call failed")]
    ApiCallFailed,
    #[error("object does not exist")]
    ObjectDoesNotExist,
    #[error("effect does not exist")]
    EffectDoesNotExist,
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

/// [`ReadSection::get_object_effect_item_parsed`] などのエラー。
#[derive(thiserror::Error, Debug)]
pub enum EditSectionParsedError<E: std::error::Error + Send + Sync + 'static> {
    #[error(transparent)]
    EditSectionError(#[from] EditSectionError),
    #[error("value parse error: {0}")]
    ParseError(E),
}

pub type EditSectionResult<T> = Result<T, EditSectionError>;

/// 読み取り専用の編集セクションハンドル。
#[derive(Debug)]
pub struct ReadSection {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
}

/// 編集セクションのハンドル。
#[derive(Debug)]
pub struct EditSection {
    /// 編集情報。
    pub info: EditInfo,

    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
    read_section: ReadSection,
}

impl std::ops::Deref for EditSection {
    type Target = ReadSection;

    fn deref(&self) -> &Self::Target {
        &self.read_section
    }
}

impl ReadSection {
    /// 生ポインタから `ReadSection` を作成する。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_raw(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self { internal: ptr }
    }

    /// 指定のフレーム番号以降にあるオブジェクトを検索する。
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

    /// オブジェクトに対象エフェクトが何個存在するかを取得する。
    pub fn count_object_effect(
        &self,
        object: ObjectHandle,
        effect: &str,
    ) -> EditSectionResult<usize> {
        self.ensure_object_exists(object)?;
        let c_effect = crate::common::CWString::new(effect)?;
        let count =
            unsafe { ((*self.internal).count_object_effect)(object.internal, c_effect.as_ptr()) };
        Ok(count.try_into()?)
    }

    /// 指定のオブジェクトのレイヤーとフレーム情報を取得する。
    pub fn get_object_layer_frame(
        &self,
        object: ObjectHandle,
    ) -> EditSectionResult<ObjectLayerFrame> {
        self.ensure_object_exists(object)?;
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        Ok(ObjectLayerFrame {
            layer: object.layer.try_into()?,
            start: object.start.try_into()?,
            end: object.end.try_into()?,
        })
    }

    /// オブジェクトの情報をエイリアスデータとして取得する。
    pub fn get_object_alias(&self, object: ObjectHandle) -> EditSectionResult<String> {
        self.ensure_object_exists(object)?;
        let alias_ptr = unsafe { ((*self.internal).get_object_alias)(object.internal) };
        if alias_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(alias_ptr) };
        let alias = c_str.to_str()?.to_owned();
        Ok(alias)
    }

    /// オブジェクトの情報をエイリアスデータとして取得し、パースする。
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_object_alias_parsed(
        &self,
        object: ObjectHandle,
    ) -> Result<aviutl2_alias::Table, EditSectionParsedError<aviutl2_alias::TableParseError>> {
        let alias_str = self.get_object_alias(object)?;
        let alias_data = alias_str
            .parse()
            .map_err(EditSectionParsedError::ParseError)?;
        Ok(alias_data)
    }

    /// オブジェクト名を取得する。
    pub fn get_object_name(&self, object: ObjectHandle) -> EditSectionResult<Option<String>> {
        self.ensure_object_exists(object)?;
        let name_ptr = unsafe { ((*self.internal).get_object_name)(object.internal) };
        if name_ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { crate::common::load_wide_string(name_ptr) }))
    }

    /// オブジェクトの設定項目の値を文字列で取得する。
    pub fn get_object_effect_item(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> EditSectionResult<String> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
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

    /// オブジェクトの設定項目の値を取得し、パースする。
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_object_effect_item_parsed<T: aviutl2_alias::FromTableValue>(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> Result<T, EditSectionParsedError<<T as aviutl2_alias::FromTableValue>::Err>>
    where
        <T as aviutl2_alias::FromTableValue>::Err: std::error::Error + Sync + Send + 'static,
    {
        let value_str = self.get_object_effect_item(object, effect_name, effect_index, item)?;
        T::from_table_value(&value_str).map_err(EditSectionParsedError::ParseError)
    }

    /// 現在、オブジェクト設定ウィンドウで選択されているオブジェクトを取得する。
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

    /// 現在選択されているオブジェクトの一覧を取得する。
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

    /// 指定のメディアファイルがサポートされているかどうか調べる。
    pub fn is_support_media_file<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
        mode: MediaFileSupportMode,
    ) -> EditSectionResult<bool> {
        let c_file_path = crate::common::CWString::new(&file_path.as_ref().to_string_lossy())?;
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

    /// 指定のメディアファイルの情報を取得する。
    pub fn get_media_info<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
    ) -> EditSectionResult<MediaInfo> {
        let c_file_path = crate::common::CWString::new(&file_path.as_ref().to_string_lossy())?;
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

    /// レイヤーの名前を取得する。
    pub fn get_layer_name(&self, layer: usize) -> EditSectionResult<Option<String>> {
        let name_ptr = unsafe { ((*self.internal).get_layer_name)(layer.try_into()?) };
        if name_ptr.is_null() {
            return Ok(None);
        }
        Ok(Some(unsafe { crate::common::load_wide_string(name_ptr) }))
    }

    /// シーン名を取得する。
    pub fn get_scene_name(&self) -> EditSectionResult<String> {
        let name_ptr = unsafe { ((*self.internal).get_scene_name)() };
        if name_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(unsafe { crate::common::load_wide_string(name_ptr) })
    }

    /// レイヤーの表示・非表示を取得する。
    pub fn get_layer_enable(&self, layer: usize) -> EditSectionResult<bool> {
        let visible = unsafe { ((*self.internal).get_layer_enable)(layer.try_into()?) };
        Ok(visible)
    }

    /// レイヤーのロック状態を取得する。
    pub fn get_layer_lock(&self, layer: usize) -> EditSectionResult<bool> {
        let locked = unsafe { ((*self.internal).get_layer_lock)(layer.try_into()?) };
        Ok(locked)
    }

    /// オブジェクトの区間の数を取得する。
    pub fn get_object_section_num(&self, object: ObjectHandle) -> EditSectionResult<usize> {
        self.ensure_object_exists(object)?;
        let count = unsafe { ((*self.internal).get_object_section_num)(object.internal) };
        Ok(count.try_into()?)
    }

    /// オブジェクトの区間の開始フレーム番号を取得する。
    pub fn get_object_section_frame(
        &self,
        object: ObjectHandle,
        section: usize,
    ) -> EditSectionResult<Option<usize>> {
        self.ensure_object_exists(object)?;
        let frame = unsafe {
            ((*self.internal).get_object_section_frame)(object.internal, section.try_into()?)
        };
        if frame == -1 {
            Ok(None)
        } else {
            Ok(Some(frame.try_into()?))
        }
    }

    /// オブジェクトの区間のフレーム番号の一覧を取得する。
    pub fn get_object_section_frames(&self, object: ObjectHandle) -> EditSectionResult<Vec<usize>> {
        self.ensure_object_exists(object)?;
        let section_num = self.get_object_section_num(object)?;
        let mut frames = Vec::new();
        for section in 0..=section_num {
            frames.push(
                self.get_object_section_frame(object, section)?
                    .ok_or(EditSectionError::ApiCallFailed)?,
            );
        }
        Ok(frames)
    }

    /// 指定フレーム位置でのオブジェクトのトラックバー項目の値を取得する。
    pub fn get_object_track_value(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        frame: f64,
    ) -> EditSectionResult<f64> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
        let c_item = crate::common::CWString::new(item)?;
        let mut value = 0.0;
        let success = unsafe {
            ((*self.internal).get_object_track_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                frame,
                &mut value,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(value)
    }

    /// 指定フレーム位置でのオブジェクトのチェックボックス項目の値を取得する。
    pub fn get_object_check_value(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        frame: usize,
    ) -> EditSectionResult<bool> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
        let c_item = crate::common::CWString::new(item)?;
        let mut value = false;
        let success = unsafe {
            ((*self.internal).get_object_check_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                frame.try_into()?,
                &mut value,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(value)
    }

    /// オブジェクトのトラックバー項目の情報を取得する。
    ///
    /// # Note
    ///
    /// 移動なしのときは`None`を返します。
    /// そもそもトラックバーではない項目の場合はエラーになります。
    pub fn get_object_track_info(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> EditSectionResult<Option<TrackInfo>> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
        let c_item = crate::common::CWString::new(item)?;
        let mut info = std::mem::MaybeUninit::<aviutl2_sys::plugin2::TRACK_INFO>::uninit();
        let success = unsafe {
            ((*self.internal).get_object_track_info)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                info.as_mut_ptr(),
                std::mem::size_of::<aviutl2_sys::plugin2::TRACK_INFO>() as i32,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }

        let info = unsafe { info.assume_init() };
        let mode = if info.mode.is_null() {
            return Ok(None);
        } else {
            unsafe { crate::common::load_wide_string(info.mode) }
        };
        let param_num: usize = info.param_num.try_into()?;
        let params = if param_num == 0 {
            Vec::new()
        } else {
            if info.param.is_null() {
                return Err(EditSectionError::ApiCallFailed);
            }
            unsafe { std::slice::from_raw_parts(info.param, param_num) }.to_vec()
        };

        Ok(Some(TrackInfo {
            mode,
            params,
            accelerate: info.accelerate,
            decelerate: info.decelerate,
            twopoint: info.twopoint,
            timecontrol: info.timecontrol,
            group_num: info.group_num.try_into()?,
            group_index: info.group_index.try_into()?,
            group_name: if info.group_name.is_null() {
                None
            } else {
                Some(unsafe { crate::common::load_wide_string(info.group_name) })
            },
        }))
    }

    /// 現在のパレット名を取得する。
    pub fn get_palette_name(&self) -> EditSectionResult<String> {
        let name_ptr = unsafe { ((*self.internal).get_palette_name)() };
        if name_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(unsafe { crate::common::load_wide_string(name_ptr) })
    }

    /// 指定のパレット情報を取得する。
    pub fn get_palette_info(&self, name: &str) -> EditSectionResult<PaletteInfo> {
        let c_name = crate::common::CWString::new(name)?;
        let mut info = std::mem::MaybeUninit::<aviutl2_sys::plugin2::PALETTE_INFO>::uninit();
        let success = unsafe {
            ((*self.internal).get_palette_info)(
                c_name.as_ptr(),
                info.as_mut_ptr(),
                std::mem::size_of::<aviutl2_sys::plugin2::PALETTE_INFO>() as i32,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(PaletteInfo::from_raw(unsafe { info.assume_init() }))
    }

    /// 現在のパレット情報を取得する。
    pub fn get_current_palette_info(&self) -> EditSectionResult<PaletteInfo> {
        let name = self.get_palette_name()?;
        self.get_palette_info(&name)
    }

    /// 登録されているフォントのDirectWriteのフォントのポインタを取得する。
    pub fn get_font(&self, font_name: &str) -> EditSectionResult<*mut std::ffi::c_void> {
        let c_font_name = crate::common::CWString::new(font_name)?;
        let ptr = unsafe { ((*self.internal).get_font)(c_font_name.as_ptr()) };
        if ptr.is_null() {
            Err(EditSectionError::ApiCallFailed)
        } else {
            Ok(ptr)
        }
    }

    /// オブジェクトのトラックバーグループの所属アイテム名を取得する。
    pub fn get_object_track_group_names(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        group_name: &str,
    ) -> EditSectionResult<Vec<String>> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
        let c_group_name = crate::common::CWString::new(group_name)?;
        let item_num = unsafe {
            ((*self.internal).get_object_track_group_names)(
                object.internal,
                c_effect_name.as_ptr(),
                c_group_name.as_ptr(),
                std::ptr::null_mut(),
                0,
            )
        };
        let item_num: usize = item_num.try_into()?;
        if item_num == 0 {
            return Ok(Vec::new());
        }

        let mut item_names = vec![std::ptr::null(); item_num];
        let actual_item_num = unsafe {
            ((*self.internal).get_object_track_group_names)(
                object.internal,
                c_effect_name.as_ptr(),
                c_group_name.as_ptr(),
                item_names.as_mut_ptr(),
                item_names.len().try_into()?,
            )
        };
        let actual_item_num: usize = actual_item_num.try_into()?;
        if actual_item_num > item_names.len() {
            return Err(EditSectionError::ApiCallFailed);
        }
        item_names.truncate(actual_item_num);

        item_names
            .into_iter()
            .map(|item_name| {
                if item_name.is_null() {
                    Err(EditSectionError::ApiCallFailed)
                } else {
                    Ok(unsafe { crate::common::load_wide_string(item_name) })
                }
            })
            .collect()
    }

    /// 選択中オブジェクトの区間の位置を取得する。
    pub fn get_focus_object_section(&self) -> EditSectionResult<Option<usize>> {
        let section = unsafe { ((*self.internal).get_focus_object_section)() };
        if section == -1 {
            Ok(None)
        } else {
            Ok(Some(section.try_into()?))
        }
    }

    /// BPMグリッドのBPM情報の一覧を取得する。
    pub fn get_grid_bpm_list(&self) -> EditSectionResult<Vec<BpmInfo>> {
        let mut bpm_info_list = Vec::<aviutl2_sys::plugin2::BPM_INFO>::new();
        let bpm_size = std::mem::size_of::<aviutl2_sys::plugin2::BPM_INFO>().try_into()?;
        let bpm_num =
            unsafe { ((*self.internal).get_grid_bpm_list)(std::ptr::null_mut(), 0, bpm_size) };
        if bpm_num <= 0 {
            return Ok(vec![]);
        }
        bpm_info_list.resize_with(bpm_num as usize, || aviutl2_sys::plugin2::BPM_INFO {
            tempo: 0.0,
            beat: 0,
            start: 0.0,
            offset: 0.0,
        });
        let actual_bpm_num = unsafe {
            ((*self.internal).get_grid_bpm_list)(
                bpm_info_list.as_mut_ptr(),
                bpm_info_list.len() as i32,
                bpm_size,
            )
        };
        if actual_bpm_num != bpm_num {
            return Err(EditSectionError::ApiCallFailed);
        }
        let bpm_info_list = bpm_info_list.into_iter().map(BpmInfo::from).collect();
        Ok(bpm_info_list)
    }

    /// オブジェクトからエフェクトを検索する。
    pub fn find_effect(
        &self,
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
    ) -> EditSectionResult<EffectHandle> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
        let handle =
            unsafe { ((*self.internal).find_effect)(object.internal, c_effect_name.as_ptr()) };
        if handle.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(EffectHandle { internal: handle })
    }

    /// オブジェクトの先頭のエフェクトを取得する。
    pub fn get_first_effect(&self, object: ObjectHandle) -> EditSectionResult<EffectHandle> {
        self.ensure_object_exists(object)?;
        let handle = unsafe { ((*self.internal).find_effect)(object.internal, std::ptr::null()) };
        if handle.is_null() {
            // NOTE: 知っている限りでは先頭のエフェクトは常に存在するはず...
            Err(EditSectionError::ApiCallFailed)
        } else {
            Ok(EffectHandle { internal: handle })
        }
    }

    /// オブジェクトのエフェクト一覧を取得する。
    pub fn get_effects(&self, object: ObjectHandle) -> EditSectionResult<Vec<EffectHandle>> {
        self.ensure_object_exists(object)?;
        let mut effects = Vec::<aviutl2_sys::plugin2::EFFECT_HANDLE>::new();
        let num_effects =
            unsafe { ((*self.internal).get_effect_list)(object.internal, std::ptr::null_mut(), 0) };
        if num_effects <= 0 {
            return Err(EditSectionError::ApiCallFailed);
        }

        effects.resize_with(num_effects as usize, std::ptr::null_mut);
        let actual_num_effects = unsafe {
            ((*self.internal).get_effect_list)(
                object.internal,
                effects.as_mut_ptr(),
                effects.len() as i32,
            )
        };
        if actual_num_effects != num_effects {
            return Err(EditSectionError::ApiCallFailed);
        }
        let effect_handles = effects
            .into_iter()
            .map(|handle| EffectHandle { internal: handle })
            .collect();
        Ok(effect_handles)
    }

    /// エフェクトの有効・無効状態を取得する。
    pub fn get_effect_enable(&self, effect: EffectHandle) -> EditSectionResult<bool> {
        self.ensure_effect_exists(effect)?;
        let enabled = unsafe { ((*self.internal).get_effect_enable)(effect.internal) };
        Ok(enabled)
    }

    /// エフェクトのロック状態を取得する。
    pub fn get_effect_lock(&self, effect: EffectHandle) -> EditSectionResult<bool> {
        self.ensure_effect_exists(effect)?;
        let locked = unsafe { ((*self.internal).get_effect_lock)(effect.internal) };
        Ok(locked)
    }

    /// エフェクトの設定項目の値を文字列で取得する。
    pub fn get_effect_item_value(
        &self,
        effect: EffectHandle,
        item: &str,
    ) -> EditSectionResult<String> {
        self.ensure_effect_exists(effect)?;
        let c_item = crate::common::CWString::new(item)?;
        let value_ptr =
            unsafe { ((*self.internal).get_effect_item_value)(effect.internal, c_item.as_ptr()) };
        if value_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
        let value = c_str.to_str()?.to_owned();
        Ok(value)
    }

    /// エフェクトの設定項目の値を取得し、パースする。
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_effect_item_value_parsed<T: aviutl2_alias::FromTableValue>(
        &self,
        effect: EffectHandle,
        item: &str,
    ) -> Result<T, EditSectionParsedError<<T as aviutl2_alias::FromTableValue>::Err>>
    where
        <T as aviutl2_alias::FromTableValue>::Err: std::error::Error + Sync + Send + 'static,
    {
        let value_str = self.get_effect_item_value(effect, item)?;
        T::from_table_value(&value_str).map_err(EditSectionParsedError::ParseError)
    }

    /// エフェクトの指定フレーム位置でのトラックバー項目の値を取得する。
    pub fn get_effect_track_value(
        &self,
        effect: EffectHandle,
        item: &str,
        frame: f64,
    ) -> EditSectionResult<f64> {
        self.ensure_effect_exists(effect)?;
        let c_item = crate::common::CWString::new(item)?;
        let mut value = 0.0;
        let success = unsafe {
            ((*self.internal).get_effect_track_value)(
                effect.internal,
                c_item.as_ptr(),
                frame,
                &mut value,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(value)
    }

    /// エフェクトの指定フレーム位置でのチェックボックス項目の値を取得する。
    pub fn get_effect_check_value(
        &self,
        effect: EffectHandle,
        item: &str,
        frame: usize,
    ) -> EditSectionResult<bool> {
        self.ensure_effect_exists(effect)?;
        let c_item = crate::common::CWString::new(item)?;
        let mut value = false;
        let success = unsafe {
            ((*self.internal).get_effect_check_value)(
                effect.internal,
                c_item.as_ptr(),
                frame.try_into()?,
                &mut value,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(value)
    }

    /// エフェクトのトラックバー項目の情報を取得する。
    pub fn get_effect_track_info(
        &self,
        effect: EffectHandle,
        item: &str,
    ) -> EditSectionResult<Option<TrackInfo>> {
        self.ensure_effect_exists(effect)?;
        let c_item = crate::common::CWString::new(item)?;
        let mut info = std::mem::MaybeUninit::<aviutl2_sys::plugin2::TRACK_INFO>::uninit();
        let success = unsafe {
            ((*self.internal).get_effect_track_info)(
                effect.internal,
                c_item.as_ptr(),
                info.as_mut_ptr(),
                std::mem::size_of::<aviutl2_sys::plugin2::TRACK_INFO>() as i32,
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }

        let info = unsafe { info.assume_init() };
        let mode = if info.mode.is_null() {
            return Ok(None);
        } else {
            unsafe { crate::common::load_wide_string(info.mode) }
        };
        let param_num: usize = info.param_num.try_into()?;
        let params = if param_num == 0 {
            Vec::new()
        } else {
            if info.param.is_null() {
                return Err(EditSectionError::ApiCallFailed);
            }
            unsafe { std::slice::from_raw_parts(info.param, param_num) }.to_vec()
        };

        Ok(Some(TrackInfo {
            mode,
            params,
            accelerate: info.accelerate,
            decelerate: info.decelerate,
            twopoint: info.twopoint,
            timecontrol: info.timecontrol,
            group_num: info.group_num.try_into()?,
            group_index: info.group_index.try_into()?,
            group_name: if info.group_name.is_null() {
                None
            } else {
                Some(unsafe { crate::common::load_wide_string(info.group_name) })
            },
        }))
    }

    /// エフェクト名を取得する。
    pub fn get_effect_name(&self, effect: EffectHandle) -> EditSectionResult<String> {
        let name_ptr = unsafe { ((*self.internal).get_effect_name)(effect.internal) };
        if name_ptr.is_null() {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(unsafe { crate::common::load_wide_string(name_ptr) })
    }

    /// オブジェクトが存在するかどうか調べる。
    pub fn object_exists(&self, object: ObjectHandle) -> bool {
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        object.layer != -1
    }

    /// エフェクトが存在するかどうか調べる。
    pub fn effect_exists(&self, effect: EffectHandle) -> bool {
        let name_ptr = unsafe { ((*self.internal).get_effect_name)(effect.internal) };
        !name_ptr.is_null()
    }

    fn ensure_object_exists(&self, object: ObjectHandle) -> EditSectionResult<()> {
        if !self.object_exists(object) {
            return Err(EditSectionError::ObjectDoesNotExist);
        }
        Ok(())
    }

    fn ensure_effect_exists(&self, effect: EffectHandle) -> EditSectionResult<()> {
        if !self.effect_exists(effect) {
            return Err(EditSectionError::EffectDoesNotExist);
        }
        Ok(())
    }

    /// 特定のレイヤー内のオブジェクトをイテレータで取得する。
    pub fn objects_in_layer(
        &self,
        layer: usize,
    ) -> EditSectionLayerObjectsIterator<'_, ReadSection> {
        EditSectionLayerObjectsIterator::new(self, layer)
    }

    /// [EditSectionLayerCaller] を作成する。
    pub fn layer<'a>(&'a self, layer: usize) -> EditSectionLayerCaller<'a, ReadSection> {
        EditSectionLayerCaller::new(self, layer)
    }

    /// [EditSectionObjectCaller] を作成する。
    pub fn object<'a>(&'a self, object: ObjectHandle) -> EditSectionObjectCaller<'a, ReadSection> {
        EditSectionObjectCaller::new(self, object)
    }

    /// [EditSectionEffectCaller] を作成する。
    pub fn effect<'a>(&'a self, effect: EffectHandle) -> EditSectionEffectCaller<'a, ReadSection> {
        EditSectionEffectCaller::new(self, effect)
    }
}

impl EditSection {
    /// 生ポインタから `EditSection` を作成する。
    ///
    /// # Safety
    ///
    /// call_edit_section から取得した、有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_raw(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        let info = unsafe { EditInfo::from_raw((*ptr).info) };
        Self {
            internal: ptr,
            info,
            read_section: ReadSection { internal: ptr },
        }
    }

    /// オブジェクトエイリアスから指定の位置にオブジェクトを作成する。
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

    /// オブジェクト名を設定する。
    ///
    /// # Note
    ///
    /// `name`に`None`や空文字を指定すると、標準の名前になります。
    pub fn set_object_name(
        &self,
        object: ObjectHandle,
        name: Option<&str>,
    ) -> EditSectionResult<()> {
        self.read_section.ensure_object_exists(object)?;
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

    /// オブジェクトの設定項目の値を文字列で設定する。
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
        object: ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        value: &str,
    ) -> EditSectionResult<()> {
        self.read_section.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&effect_key(effect_name, effect_index))?;
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

    /// オブジェクトを移動する。
    pub fn move_object(
        &self,
        object: ObjectHandle,
        new_layer: usize,
        new_start_frame: usize,
    ) -> EditSectionResult<()> {
        self.read_section.ensure_object_exists(object)?;
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

    /// オブジェクトを削除する。
    pub fn delete_object(&self, object: ObjectHandle) -> EditSectionResult<()> {
        self.read_section.ensure_object_exists(object)?;
        unsafe { ((*self.internal).delete_object)(object.internal) };
        Ok(())
    }

    /// オブジェクト設定ウィンドウで指定のオブジェクトを選択状態にする。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self, object: ObjectHandle) -> EditSectionResult<()> {
        self.read_section.ensure_object_exists(object)?;
        unsafe { ((*self.internal).set_focus_object)(object.internal) };
        Ok(())
    }

    /// プロジェクトファイルのポインタを取得する。
    pub fn get_project_file<'handle>(
        &'handle self,
        edit_handle: &crate::generic::EditHandle,
    ) -> crate::generic::ProjectFile<'handle> {
        let pf_ptr = unsafe { ((*self.internal).get_project_file)(edit_handle.internal) };
        unsafe { crate::generic::ProjectFile::from_raw(pf_ptr) }
    }

    /// マウス座標のレイヤー・フレーム位置を取得する。
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

    /// 指定のスクリーン座標のレイヤー・フレーム位置を取得する。
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

    /// 指定の位置にメディアファイルからオブジェクトを作成する。
    ///
    /// # Arguments
    ///
    /// - `file_path`：メディアファイルのパス。
    /// - `layer`：作成するオブジェクトのレイヤー番号（0始まり）。
    /// - `frame`：作成するオブジェクトのフレーム番号（0始まり）。
    /// - `length`：作成するオブジェクトの長さ（フレーム数）。`None`を指定した場合、長さや追加位置は自動的に調整されます。
    pub fn create_object_from_media_file<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
        layer: usize,
        frame: usize,
        length: Option<usize>,
    ) -> EditSectionResult<ObjectHandle> {
        let c_file_path = crate::common::CWString::new(&file_path.as_ref().to_string_lossy())?;
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

    /// 指定の位置にオブジェクトを作成する。
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

    /// 現在のレイヤー・フレーム位置を設定する。
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

    /// レイヤー編集のレイヤー・フレームの表示開始位置を設定する。
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

    /// フレーム範囲選択を設定する。
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

    /// 選択されているフレーム範囲選択を解除する。
    pub fn clear_select_range(&self) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_select_range)(-1, -1);
        }
        Ok(())
    }

    /// BPMグリッドのBPM情報の一覧を設定する。
    pub fn set_grid_bpm_list(&self, bpm_info: &[BpmInfo]) -> EditSectionResult<()> {
        let mut raw_bpm_info = bpm_info
            .iter()
            .copied()
            .map(aviutl2_sys::plugin2::BPM_INFO::from)
            .collect::<Vec<_>>();
        unsafe {
            ((*self.internal).set_grid_bpm_list)(
                raw_bpm_info.as_mut_ptr(),
                raw_bpm_info.len().try_into()?,
                std::mem::size_of::<aviutl2_sys::plugin2::BPM_INFO>().try_into()?,
            );
        }
        Ok(())
    }

    /// レイヤーの名前を設定する。
    /// `name`に`None`や空文字を指定すると、標準の名前になります。
    pub fn set_layer_name(&self, layer: usize, name: Option<&str>) -> EditSectionResult<()> {
        match name {
            None => {
                unsafe { ((*self.internal).set_layer_name)(layer.try_into()?, std::ptr::null()) };
                Ok(())
            }
            Some(name) => {
                let c_name = crate::common::CWString::new(name)?;
                unsafe {
                    ((*self.internal).set_layer_name)(layer.try_into()?, c_name.as_ptr());
                }
                Ok(())
            }
        }
    }

    /// シーン名を設定する。
    ///
    /// <div class="warning">
    /// シーンの操作は現状Undoに対応していません。
    /// </div>
    pub fn set_scene_name(&self, name: &str) -> EditSectionResult<()> {
        let c_name = crate::common::CWString::new(name)?;
        unsafe { ((*self.internal).set_scene_name)(c_name.as_ptr()) };
        Ok(())
    }

    /// シーンの解像度を設定する。
    ///
    /// <div class="warning">
    /// シーンの操作は現状Undoに対応していません。
    /// </div>
    pub fn set_scene_size(&self, width: usize, height: usize) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_scene_size)(width.try_into()?, height.try_into()?);
        }
        Ok(())
    }

    /// シーンのフレームレートを設定する。
    ///
    /// <div class="warning">
    /// シーンの操作は現状Undoに対応していません。
    /// </div>
    pub fn set_scene_fps(&self, fps: Rational32) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_scene_frame_rate)(*fps.numer(), *fps.denom());
        }
        Ok(())
    }

    /// シーンのサンプルレートを設定する。
    ///
    /// <div class="warning">
    /// シーンの操作は現状Undoに対応していません。
    /// </div>
    pub fn set_scene_sample_rate(&self, sample_rate: usize) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_scene_sample_rate)(sample_rate.try_into()?);
        }
        Ok(())
    }

    /// レイヤーの表示・非表示を設定する。
    pub fn set_layer_enable(&self, layer: usize, enable: bool) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_layer_enable)(layer.try_into()?, enable);
        }
        Ok(())
    }

    /// レイヤーのロック状態を設定する。
    pub fn set_layer_lock(&self, layer: usize, lock: bool) -> EditSectionResult<()> {
        unsafe {
            ((*self.internal).set_layer_lock)(layer.try_into()?, lock);
        }
        Ok(())
    }

    /// エフェクトの有効・無効状態を設定する。
    pub fn set_effect_enable(&self, effect: EffectHandle, enable: bool) -> EditSectionResult<()> {
        self.read_section.ensure_effect_exists(effect)?;
        unsafe {
            ((*self.internal).set_effect_enable)(effect.internal, enable);
        }
        Ok(())
    }

    /// エフェクトのロック状態を設定する。
    pub fn set_effect_lock(&self, effect: EffectHandle, lock: bool) -> EditSectionResult<()> {
        self.read_section.ensure_effect_exists(effect)?;
        unsafe {
            ((*self.internal).set_effect_lock)(effect.internal, lock);
        }
        Ok(())
    }

    /// エフェクトの設定項目の値を文字列で設定する。
    pub fn set_effect_item_value(
        &self,
        effect: EffectHandle,
        item: &str,
        value: &str,
    ) -> EditSectionResult<()> {
        self.read_section.ensure_effect_exists(effect)?;
        let c_item = crate::common::CWString::new(item)?;
        let c_value = std::ffi::CString::new(value)?;
        let success = unsafe {
            ((*self.internal).set_effect_item_value)(
                effect.internal,
                c_item.as_ptr(),
                c_value.as_ptr(),
            )
        };
        if !success {
            return Err(EditSectionError::ApiCallFailed);
        }
        Ok(())
    }

    /// すべてのレイヤーをイテレータで取得する。
    pub fn layers(&self) -> EditSectionLayersIterator<'_> {
        EditSectionLayersIterator::new(self)
    }
    /// 特定のレイヤー内のオブジェクトをイテレータで取得する。
    pub fn objects_in_layer(
        &self,
        layer: usize,
    ) -> EditSectionLayerObjectsIterator<'_, EditSection> {
        EditSectionLayerObjectsIterator::new(self, layer)
    }

    /// [EditSectionLayerCaller] を作成する。
    pub fn layer<'a>(&'a self, layer: usize) -> EditSectionLayerCaller<'a, EditSection> {
        EditSectionLayerCaller::new(self, layer)
    }
    /// [EditSectionObjectCaller] を作成する。
    pub fn object<'a>(&'a self, object: ObjectHandle) -> EditSectionObjectCaller<'a, EditSection> {
        EditSectionObjectCaller::new(self, object)
    }

    /// [EditSectionEffectCaller] を作成する。
    pub fn effect<'a>(&'a self, effect: EffectHandle) -> EditSectionEffectCaller<'a, EditSection> {
        EditSectionEffectCaller::new(self, effect)
    }
}

trait ReadSectionProvider {
    fn as_read_section(&self) -> &ReadSection;
}

impl ReadSectionProvider for ReadSection {
    fn as_read_section(&self) -> &ReadSection {
        self
    }
}

impl ReadSectionProvider for EditSection {
    fn as_read_section(&self) -> &ReadSection {
        &self.read_section
    }
}

/// オブジェクト主体で関数を呼び出すための構造体。
/// EditSection と ObjectHandle の組をまとめ、対象オブジェクトに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionObjectCaller<'a, S> {
    edit_section: &'a S,
    pub handle: ObjectHandle,
}
impl<'a, S> EditSectionObjectCaller<'a, S> {
    pub fn new(edit_section: &'a S, object: ObjectHandle) -> Self {
        Self {
            edit_section,
            handle: object,
        }
    }
}

impl<S> std::ops::Deref for EditSectionObjectCaller<'_, S> {
    type Target = ObjectHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

#[expect(private_bounds)]
impl<S> EditSectionObjectCaller<'_, S>
where
    S: ReadSectionProvider,
{
    fn read_section(&self) -> &ReadSection {
        self.edit_section.as_read_section()
    }

    /// オブジェクトのレイヤーとフレーム情報を取得する。
    pub fn get_layer_frame(&self) -> EditSectionResult<ObjectLayerFrame> {
        self.read_section().get_object_layer_frame(self.handle)
    }

    /// オブジェクトの情報をエイリアスデータとして取得する。
    pub fn get_alias(&self) -> EditSectionResult<String> {
        self.read_section().get_object_alias(self.handle)
    }

    /// オブジェクトの情報をエイリアスデータとして取得し、パースする。
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_alias_parsed(&self) -> EditSectionResult<aviutl2_alias::Table> {
        self.read_section()
            .get_object_alias(self.handle)?
            .parse()
            .map_err(Into::into)
    }

    /// オブジェクトに対象エフェクトが何個存在するかを取得する。
    ///
    /// # Arguments
    ///
    /// - `effect`：対象のエフェクト名。（エイリアスファイルの effect.name の値）
    ///
    /// # Returns
    ///
    /// 対象エフェクトの数。存在しない場合は0を返します。
    pub fn count_effect(&self, effect: &str) -> EditSectionResult<usize> {
        self.read_section().count_object_effect(self.handle, effect)
    }

    /// オブジェクトからエフェクトを検索する。
    pub fn find_effect(
        &self,
        effect_name: &str,
        effect_index: usize,
    ) -> EditSectionResult<EffectHandle> {
        self.read_section()
            .find_effect(self.handle, effect_name, effect_index)
    }

    /// オブジェクトの先頭のエフェクトを取得する。
    pub fn get_first_effect(&self) -> EditSectionResult<EffectHandle> {
        self.read_section().get_first_effect(self.handle)
    }

    /// オブジェクトのエフェクト一覧を取得する。
    pub fn get_effects(&self) -> EditSectionResult<Vec<EffectHandle>> {
        self.read_section().get_effects(self.handle)
    }

    /// オブジェクトの設定項目の値を文字列で取得する。
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
        self.read_section()
            .get_object_effect_item(self.handle, effect_name, effect_index, item)
    }

    /// オブジェクトの設定項目の値を取得し、パースする。
    ///
    /// # See Also
    ///
    /// [`EditSectionObjectCaller::get_effect_item`]
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_effect_item_parsed<T: aviutl2_alias::FromTableValue>(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> Result<T, EditSectionParsedError<<T as aviutl2_alias::FromTableValue>::Err>>
    where
        <T as aviutl2_alias::FromTableValue>::Err: std::error::Error + Sync + Send + 'static,
    {
        let value_str = self.get_effect_item(effect_name, effect_index, item)?;
        T::from_table_value(&value_str).map_err(EditSectionParsedError::ParseError)
    }

    /// このオブジェクトが存在するかどうか調べる。
    pub fn exists(&self) -> bool {
        self.read_section().object_exists(self.handle)
    }

    /// オブジェクトの名前を取得する。
    ///
    /// # Returns
    ///
    /// 標準の名前の場合は`None`を返します。
    pub fn get_name(&self) -> EditSectionResult<Option<String>> {
        self.read_section().get_object_name(self.handle)
    }

    /// オブジェクトの区間の数を取得する。
    pub fn get_section_num(&self) -> EditSectionResult<usize> {
        self.read_section().get_object_section_num(self.handle)
    }

    /// オブジェクトの区間の開始フレーム番号を取得する。
    pub fn get_section_frame(&self, section: usize) -> EditSectionResult<Option<usize>> {
        self.read_section()
            .get_object_section_frame(self.handle, section)
    }

    /// オブジェクトの区間のフレーム番号の一覧を取得する。
    pub fn get_section_frames(&self) -> EditSectionResult<Vec<usize>> {
        self.read_section().get_object_section_frames(self.handle)
    }

    /// 指定フレーム位置でのトラックバー項目の値を取得する。
    pub fn get_track_value(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        frame: f64,
    ) -> EditSectionResult<f64> {
        self.read_section().get_object_track_value(
            self.handle,
            effect_name,
            effect_index,
            item,
            frame,
        )
    }

    /// 指定フレーム位置でのチェックボックス項目の値を取得する。
    pub fn get_check_value(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        frame: usize,
    ) -> EditSectionResult<bool> {
        self.read_section().get_object_check_value(
            self.handle,
            effect_name,
            effect_index,
            item,
            frame,
        )
    }

    /// トラックバー項目の情報を取得する。
    pub fn get_track_info(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> EditSectionResult<Option<TrackInfo>> {
        self.read_section()
            .get_object_track_info(self.handle, effect_name, effect_index, item)
    }

    /// オブジェクトのトラックバーグループの所属アイテム名を取得する。
    pub fn get_track_group_names(
        &self,
        effect_name: &str,
        effect_index: usize,
        group_name: &str,
    ) -> EditSectionResult<Vec<String>> {
        self.read_section().get_object_track_group_names(
            self.handle,
            effect_name,
            effect_index,
            group_name,
        )
    }
}

impl EditSectionObjectCaller<'_, EditSection> {
    /// オブジェクトの設定項目の値を文字列で設定する。
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

    /// オブジェクトを移動する。
    ///
    /// # Arguments
    ///
    /// - `new_layer`：移動先のレイヤー番号（0始まり）。
    /// - `new_start_frame`：移動先の開始フレーム番号（0始まり）。
    pub fn move_object(&self, new_layer: usize, new_start_frame: usize) -> EditSectionResult<()> {
        self.edit_section
            .move_object(self.handle, new_layer, new_start_frame)
    }

    /// オブジェクトを削除する。
    pub fn delete_object(&self) -> EditSectionResult<()> {
        self.edit_section.delete_object(self.handle)
    }

    /// オブジェクト設定ウィンドウでこのオブジェクトを選択状態にする。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self) -> EditSectionResult<()> {
        self.edit_section.focus_object(self.handle)
    }

    /// オブジェクトの名前を設定する。
    /// `name`に`None`や空文字を指定すると、標準の名前になります。
    pub fn set_name(&self, name: Option<&str>) -> EditSectionResult<()> {
        self.edit_section.set_object_name(self.handle, name)
    }
}

/// エフェクト主体で関数を呼び出すための構造体。
/// EditSection と EffectHandle の組をまとめ、対象エフェクトに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionEffectCaller<'a, S> {
    edit_section: &'a S,
    pub handle: EffectHandle,
}
impl<'a, S> EditSectionEffectCaller<'a, S> {
    pub fn new(edit_section: &'a S, effect: EffectHandle) -> Self {
        Self {
            edit_section,
            handle: effect,
        }
    }
}

impl<S> std::ops::Deref for EditSectionEffectCaller<'_, S> {
    type Target = EffectHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

#[expect(private_bounds)]
impl<S> EditSectionEffectCaller<'_, S>
where
    S: ReadSectionProvider,
{
    fn read_section(&self) -> &ReadSection {
        self.edit_section.as_read_section()
    }

    /// このエフェクトが存在するかどうか調べる。
    pub fn exists(&self) -> bool {
        self.read_section().effect_exists(self.handle)
    }

    /// エフェクト名を取得する。
    pub fn get_name(&self) -> EditSectionResult<String> {
        self.read_section().get_effect_name(self.handle)
    }

    /// エフェクトの有効・無効状態を取得する。
    pub fn get_enable(&self) -> EditSectionResult<bool> {
        self.read_section().get_effect_enable(self.handle)
    }

    /// エフェクトのロック状態を取得する。
    pub fn get_lock(&self) -> EditSectionResult<bool> {
        self.read_section().get_effect_lock(self.handle)
    }

    /// エフェクトの設定項目の値を文字列で取得する。
    pub fn get_item_value(&self, item: &str) -> EditSectionResult<String> {
        self.read_section().get_effect_item_value(self.handle, item)
    }

    /// エフェクトの設定項目の値を取得し、パースする。
    #[cfg(feature = "aviutl2-alias")]
    pub fn get_item_value_parsed<T: aviutl2_alias::FromTableValue>(
        &self,
        item: &str,
    ) -> Result<T, EditSectionParsedError<<T as aviutl2_alias::FromTableValue>::Err>>
    where
        <T as aviutl2_alias::FromTableValue>::Err: std::error::Error + Sync + Send + 'static,
    {
        let value_str = self.get_item_value(item)?;
        T::from_table_value(&value_str).map_err(EditSectionParsedError::ParseError)
    }

    /// 指定フレーム位置でのトラックバー項目の値を取得する。
    pub fn get_track_value(&self, item: &str, frame: f64) -> EditSectionResult<f64> {
        self.read_section()
            .get_effect_track_value(self.handle, item, frame)
    }

    /// 指定フレーム位置でのチェックボックス項目の値を取得する。
    pub fn get_check_value(&self, item: &str, frame: usize) -> EditSectionResult<bool> {
        self.read_section()
            .get_effect_check_value(self.handle, item, frame)
    }

    /// トラックバー項目の情報を取得する。
    pub fn get_track_info(&self, item: &str) -> EditSectionResult<Option<TrackInfo>> {
        self.read_section().get_effect_track_info(self.handle, item)
    }
}

impl EditSectionEffectCaller<'_, EditSection> {
    /// エフェクトの有効・無効状態を設定する。
    pub fn set_enable(&self, enable: bool) -> EditSectionResult<()> {
        self.edit_section.set_effect_enable(self.handle, enable)
    }

    /// エフェクトのロック状態を設定する。
    pub fn set_lock(&self, lock: bool) -> EditSectionResult<()> {
        self.edit_section.set_effect_lock(self.handle, lock)
    }

    /// エフェクトの設定項目の値を文字列で設定する。
    pub fn set_item_value(&self, item: &str, value: &str) -> EditSectionResult<()> {
        self.edit_section
            .set_effect_item_value(self.handle, item, value)
    }
}

/// レイヤー主体で関数を呼び出すための構造体。
/// EditSection と レイヤー番号 の組をまとめ、対象レイヤーに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionLayerCaller<'a, S> {
    edit_section: &'a S,
    pub index: usize,
}
impl<'a, S> EditSectionLayerCaller<'a, S> {
    pub fn new(edit_section: &'a S, layer: usize) -> Self {
        Self {
            edit_section,
            index: layer,
        }
    }
}

#[expect(private_bounds)]
impl<S> EditSectionLayerCaller<'_, S>
where
    S: ReadSectionProvider,
{
    fn read_section(&self) -> &ReadSection {
        self.edit_section.as_read_section()
    }

    /// 指定のフレーム番号以降にあるオブジェクトを検索する。
    ///
    /// # Arguments
    ///
    /// - `frame`：検索を開始するフレーム番号（0始まり）。
    pub fn find_object_after(&self, frame: usize) -> EditSectionResult<Option<ObjectHandle>> {
        self.read_section().find_object_after(self.index, frame)
    }

    /// レイヤーの名前を取得する。
    pub fn get_name(&self) -> EditSectionResult<Option<String>> {
        self.read_section().get_layer_name(self.index)
    }

    /// レイヤーの表示・非表示を取得する。
    pub fn get_enable(&self) -> EditSectionResult<bool> {
        self.read_section().get_layer_enable(self.index)
    }

    /// レイヤーのロック状態を取得する。
    pub fn get_lock(&self) -> EditSectionResult<bool> {
        self.read_section().get_layer_lock(self.index)
    }

    /// このレイヤーに存在するすべてのオブジェクトを、
    /// 開始フレームの昇順で走査するイテレータを返す。
    pub fn objects(&self) -> EditSectionLayerObjectsIterator<'_, S> {
        EditSectionLayerObjectsIterator::new(self.edit_section, self.index)
    }
}

impl EditSectionLayerCaller<'_, EditSection> {
    /// オブジェクトエイリアスから指定の位置にオブジェクトを作成する。
    ///
    /// # See Also
    ///
    /// [`EditSection::create_object_from_alias`]
    pub fn create_object_from_alias<P: AsRef<std::path::Path>>(
        &self,
        alias: &str,
        frame: usize,
        length: usize,
    ) -> EditSectionResult<ObjectHandle> {
        self.edit_section
            .create_object_from_alias(alias, self.index, frame, length)
    }

    /// 指定の位置にメディアファイルからオブジェクトを作成する。
    ///
    /// # See Also
    ///
    /// [`EditSection::create_object_from_media_file`]
    pub fn create_object_from_media_file<P: AsRef<std::path::Path>>(
        &self,
        file_path: P,
        frame: usize,
        length: Option<usize>,
    ) -> EditSectionResult<ObjectHandle> {
        self.edit_section
            .create_object_from_media_file(file_path, self.index, frame, length)
    }

    /// 指定の位置にオブジェクトを作成する。
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

    /// レイヤーの名前を設定する。
    /// `name`に`None`や空文字を指定すると、標準の名前になります。
    pub fn set_name(&self, name: Option<&str>) -> EditSectionResult<()> {
        self.edit_section.set_layer_name(self.index, name)
    }

    /// レイヤーの表示・非表示を設定する。
    pub fn set_enable(&self, enable: bool) -> EditSectionResult<()> {
        self.edit_section.set_layer_enable(self.index, enable)
    }

    /// レイヤーのロック状態を設定する。
    pub fn set_lock(&self, lock: bool) -> EditSectionResult<()> {
        self.edit_section.set_layer_lock(self.index, lock)
    }
}

/// レイヤーのイテレータ。
#[derive(Debug, Clone)]
pub struct EditSectionLayersIterator<'a> {
    edit_section: &'a EditSection,
    current: usize,
    end: usize,
}

impl<'a> EditSectionLayersIterator<'a> {
    fn new(edit_section: &'a EditSection) -> Self {
        Self {
            edit_section,
            current: 0,
            end: edit_section.info.layer_max.saturating_add(1),
        }
    }
}

impl<'a> Iterator for EditSectionLayersIterator<'a> {
    type Item = EditSectionLayerCaller<'a, EditSection>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let layer = self.current;
        self.current += 1;
        Some(EditSectionLayerCaller::new(self.edit_section, layer))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end.saturating_sub(self.current);
        (len, Some(len))
    }
}

impl<'a> DoubleEndedIterator for EditSectionLayersIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        self.end -= 1;
        Some(EditSectionLayerCaller::new(self.edit_section, self.end))
    }
}

impl ExactSizeIterator for EditSectionLayersIterator<'_> {}

/// レイヤー内のオブジェクトを走査するイテレータ。
/// アイテムは `(オブジェクトのレイヤー・フレーム情報, ハンドル)` の組です。
#[derive(Debug, Clone)]
pub struct EditSectionLayerObjectsIterator<'a, S> {
    edit_section: &'a S,
    layer: usize,
    next_frame: usize,
}

impl<'a, S> EditSectionLayerObjectsIterator<'a, S> {
    fn new(edit_section: &'a S, layer: usize) -> Self {
        Self {
            edit_section,
            layer,
            next_frame: 0,
        }
    }
}

impl<S> Iterator for EditSectionLayerObjectsIterator<'_, S>
where
    S: ReadSectionProvider,
{
    type Item = (ObjectLayerFrame, ObjectHandle);

    fn next(&mut self) -> Option<Self::Item> {
        let read_section = self.edit_section.as_read_section();
        // 検索・取得でエラーが出た場合は None を返して終了する。
        let Ok(Some(handle)) = read_section.find_object_after(self.layer, self.next_frame) else {
            return None;
        };

        let lf = match read_section.get_object_layer_frame(handle) {
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
