use crate::common::LeakManager;
use aviutl2_sys::plugin2::EDIT_SECTION;
use std::mem::MaybeUninit;
use std::{ffi::c_void, ptr::NonNull};

/// [`Vec<FilterConfigItem>`] と相互変換するためのトレイト。
/// 基本的にはこのトレイトを手動で実装する必要はありません。
/// [`macro@filter_config_items`] マクロを使用してください。
///
/// <div class="warning">
///
/// このcrateは[`Vec<FilterConfigItem>`]との相互変換が可能であれば十分に機能します。
/// このトレイトを手動で実装する必要はありません。
///
/// </div>
///
/// # See Also
///
/// [`macro@filter_config_items`]
pub trait FilterConfigItems: Sized {
    /// [`Vec<FilterConfigItem>`] に変換します。
    fn to_config_items() -> Vec<FilterConfigItem>;

    /// [`Vec<FilterConfigItem>`] から変換します。
    ///
    /// # Panics
    ///
    /// `items` の内容が不正な場合、パニックします。
    fn from_config_items(items: &[FilterConfigItem]) -> Self;
}
#[doc(inline)]
pub use aviutl2_macros::filter_config_items;

/// `&[FilterConfigItem]` に対する拡張トレイト。
pub trait FilterConfigItemSliceExt {
    /// `&[FilterConfigItem]` から指定した構造体に変換します。
    fn to_struct<T: FilterConfigItems>(&self) -> T;
}

impl FilterConfigItemSliceExt for &[FilterConfigItem] {
    fn to_struct<T: FilterConfigItems>(&self) -> T {
        T::from_config_items(self)
    }
}

/// フィルタの設定。
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum FilterConfigItem {
    /// トラックバー。
    Track(FilterConfigTrack),
    /// チェックボックス。
    Checkbox(FilterConfigCheckbox),
    /// 色選択。
    Color(FilterConfigColor),
    /// 選択リスト。
    Select(FilterConfigSelect),
    /// ファイル選択。
    File(FilterConfigFile),
    /// 文字列。
    String(FilterConfigString),
    /// テキスト。
    Text(FilterConfigText),
    /// フォルダ選択。
    Folder(FilterConfigFolder),
    /// 汎用データ。
    Data(ErasedFilterConfigData),
    /// グループ。
    Group(FilterConfigGroup),
    /// ボタン。
    Button(FilterConfigButton),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterConfigItemValue {
    Track(f64),
    Checkbox(bool),
    Color(FilterConfigColorValue),
    Select(i32),
    File(String),
    String(String),
    Text(String),
    Folder(String),
    Data {
        value: *mut std::ffi::c_void,
        size: usize,
    },
    Group,
    Button,
}

impl FilterConfigItem {
    /// 設定名を取得します。
    ///
    /// # Note
    ///
    /// `FilterConfigItem::Group` の場合、`name` が `None` のときは空文字列を返します。
    pub fn name(&self) -> &str {
        match self {
            FilterConfigItem::Track(item) => &item.name,
            FilterConfigItem::Checkbox(item) => &item.name,
            FilterConfigItem::Color(item) => &item.name,
            FilterConfigItem::Select(item) => &item.name,
            FilterConfigItem::File(item) => &item.name,
            FilterConfigItem::String(item) => &item.name,
            FilterConfigItem::Text(item) => &item.name,
            FilterConfigItem::Folder(item) => &item.name,
            FilterConfigItem::Data(item) => &item.name,
            FilterConfigItem::Group(item) => item.name.as_deref().unwrap_or(""),
            FilterConfigItem::Button(item) => &item.name,
        }
    }

    pub(crate) fn to_raw(&self, leak_manager: &LeakManager) -> aviutl2_sys::filter2::FILTER_ITEM {
        match self {
            FilterConfigItem::Track(item) => {
                let step: f64 = item.step.into();
                aviutl2_sys::filter2::FILTER_ITEM {
                    track: aviutl2_sys::filter2::FILTER_ITEM_TRACK {
                        r#type: leak_manager.leak_as_wide_string("track"),
                        name: leak_manager.leak_as_wide_string(&item.name),
                        value: item.value,
                        s: *item.range.start(),
                        e: *item.range.end(),
                        step,
                    },
                }
            }
            FilterConfigItem::Checkbox(item) => aviutl2_sys::filter2::FILTER_ITEM {
                checkbox: aviutl2_sys::filter2::FILTER_ITEM_CHECKBOX {
                    r#type: leak_manager.leak_as_wide_string("check"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: item.value,
                },
            },
            FilterConfigItem::Color(item) => aviutl2_sys::filter2::FILTER_ITEM {
                color: aviutl2_sys::filter2::FILTER_ITEM_COLOR {
                    r#type: leak_manager.leak_as_wide_string("color"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: item.value.into(),
                },
            },
            FilterConfigItem::Select(item) => {
                let mut raw_items: Vec<aviutl2_sys::filter2::FILTER_ITEM_SELECT_ITEM> = item
                    .items
                    .iter()
                    .map(|i| aviutl2_sys::filter2::FILTER_ITEM_SELECT_ITEM {
                        name: leak_manager.leak_as_wide_string(&i.name),
                        value: i.value,
                    })
                    .collect();
                raw_items.push(aviutl2_sys::filter2::FILTER_ITEM_SELECT_ITEM {
                    name: std::ptr::null(),
                    value: 0,
                }); // 終端用
                let raw_items_ptrs = leak_manager.leak_value_vec(raw_items);
                aviutl2_sys::filter2::FILTER_ITEM {
                    select: aviutl2_sys::filter2::FILTER_ITEM_SELECT {
                        r#type: leak_manager.leak_as_wide_string("select"),
                        name: leak_manager.leak_as_wide_string(&item.name),
                        value: item.value,
                        items: raw_items_ptrs,
                    },
                }
            }
            FilterConfigItem::File(item) => {
                let raw_filters = crate::common::format_file_filters(&item.filters);
                aviutl2_sys::filter2::FILTER_ITEM {
                    file: aviutl2_sys::filter2::FILTER_ITEM_FILE {
                        r#type: leak_manager.leak_as_wide_string("file"),
                        name: leak_manager.leak_as_wide_string(&item.name),
                        value: leak_manager.leak_as_wide_string(&item.value),
                        filefilter: leak_manager.leak_as_wide_string(&raw_filters),
                    },
                }
            }
            FilterConfigItem::String(item) => aviutl2_sys::filter2::FILTER_ITEM {
                string: aviutl2_sys::filter2::FILTER_ITEM_STRING {
                    r#type: leak_manager.leak_as_wide_string("string"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: leak_manager.leak_as_wide_string(&item.value),
                },
            },
            FilterConfigItem::Text(item) => aviutl2_sys::filter2::FILTER_ITEM {
                text: aviutl2_sys::filter2::FILTER_ITEM_TEXT {
                    r#type: leak_manager.leak_as_wide_string("text"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: leak_manager.leak_as_wide_string(&item.value),
                },
            },
            FilterConfigItem::Folder(item) => aviutl2_sys::filter2::FILTER_ITEM {
                folder: aviutl2_sys::filter2::FILTER_ITEM_FOLDER {
                    r#type: leak_manager.leak_as_wide_string("folder"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: leak_manager.leak_as_wide_string(&item.value),
                },
            },
            FilterConfigItem::Data(item) => {
                let mut data = aviutl2_sys::filter2::FILTER_ITEM_DATA {
                    r#type: leak_manager.leak_as_wide_string("data"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: std::ptr::null_mut(),
                    size: item.size as i32,
                    default_value: [MaybeUninit::new(0); 1024],
                };
                assert!(item.size <= 1024, "FilterConfigData size must be <= 1024");
                unsafe {
                    // SAFETY:
                    // - item.size <= 1024かつ、
                    // - item.default_value()はitem.size分のデータを持っている
                    std::ptr::copy_nonoverlapping(
                        item.default_value().as_ptr(),
                        data.default_value.as_mut_ptr() as *mut u8,
                        item.size,
                    );
                }

                aviutl2_sys::filter2::FILTER_ITEM { data }
            }
            FilterConfigItem::Group(item) => aviutl2_sys::filter2::FILTER_ITEM {
                group: aviutl2_sys::filter2::FILTER_ITEM_GROUP {
                    r#type: leak_manager.leak_as_wide_string("group"),
                    name: leak_manager.leak_as_wide_string(item.name.as_deref().unwrap_or("")),
                    default_visible: item.opened,
                },
            },
            FilterConfigItem::Button(item) => aviutl2_sys::filter2::FILTER_ITEM {
                button: aviutl2_sys::filter2::FILTER_ITEM_BUTTON {
                    r#type: leak_manager.leak_as_wide_string("button"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    callback: item.callback,
                },
            },
        }
    }

    /// # Safety
    ///
    /// `raw` は有効なポインタである必要があります。
    pub(crate) unsafe fn get_value(
        raw: *const aviutl2_sys::filter2::FILTER_ITEM,
    ) -> FilterConfigItemValue {
        let item_type = unsafe {
            crate::common::load_wide_string(
                // SAFETY: aviutl2_sys::filter2::FILTER_ITEM の最初のメンバーはLPCWSTRなので問題ないはず
                *(raw.cast::<aviutl2_sys::common::LPCWSTR>()),
            )
        };
        match item_type.as_str() {
            "track" => {
                let raw_track = unsafe { &(*raw).track };
                FilterConfigItemValue::Track(raw_track.value)
            }
            "check" => {
                let raw_checkbox = unsafe { &(*raw).checkbox };
                FilterConfigItemValue::Checkbox(raw_checkbox.value)
            }
            "color" => {
                let raw_color = unsafe { &(*raw).color };
                FilterConfigItemValue::Color(raw_color.value.into())
            }
            "select" => {
                let raw_select = unsafe { &(*raw).select };
                FilterConfigItemValue::Select(raw_select.value)
            }
            "file" => {
                let raw_file = unsafe { &(*raw).file };
                let value = unsafe { crate::common::load_wide_string(raw_file.value) };
                FilterConfigItemValue::File(value)
            }
            "string" => {
                let raw_string = unsafe { &(*raw).string };
                let value = unsafe { crate::common::load_wide_string(raw_string.value) };
                FilterConfigItemValue::String(value)
            }
            "text" => {
                let raw_text = unsafe { &(*raw).text };
                let value = unsafe { crate::common::load_wide_string(raw_text.value) };
                FilterConfigItemValue::Text(value)
            }
            "folder" => {
                let raw_folder = unsafe { &(*raw).folder };
                let value = unsafe { crate::common::load_wide_string(raw_folder.value) };
                FilterConfigItemValue::Folder(value)
            }
            "data" => {
                // NOTE:
                // `&(*raw).data`だと最後の方がアクセス違反になりかねないメモリを指す可能性があるのでしない
                let raw_size = unsafe { (*raw).data.size };
                let raw_data = unsafe { (*raw).data.value };
                let size =
                    usize::try_from(raw_size).expect("FILTER_ITEM_DATA size must not be negative");
                assert!(
                    size <= 1024,
                    "FILTER_ITEM_DATA size must be 1024 bytes or less"
                );
                FilterConfigItemValue::Data {
                    value: raw_data,
                    size,
                }
            }
            "group" => FilterConfigItemValue::Group,
            "button" => FilterConfigItemValue::Button,
            _ => panic!("Unknown filter config item type: {}", item_type),
        }
    }

    /// # Safety
    ///
    /// `raw` は有効なポインタである必要があります。
    pub(crate) unsafe fn should_apply_from_raw(
        &self,
        raw: *const aviutl2_sys::filter2::FILTER_ITEM,
    ) -> bool {
        let value = unsafe { Self::get_value(raw) };
        match (self, value) {
            (FilterConfigItem::Track(item), FilterConfigItemValue::Track(v)) => item.value != v,
            (FilterConfigItem::Checkbox(item), FilterConfigItemValue::Checkbox(v)) => {
                item.value != v
            }
            (FilterConfigItem::Color(item), FilterConfigItemValue::Color(v)) => item.value != v,
            (FilterConfigItem::Select(item), FilterConfigItemValue::Select(v)) => item.value != v,
            (FilterConfigItem::File(item), FilterConfigItemValue::File(v)) => item.value != v,
            (FilterConfigItem::String(item), FilterConfigItemValue::String(v)) => item.value != v,
            (FilterConfigItem::Text(item), FilterConfigItemValue::Text(v)) => item.value != v,
            (FilterConfigItem::Folder(item), FilterConfigItemValue::Folder(v)) => item.value != v,
            (FilterConfigItem::Data(item), FilterConfigItemValue::Data { value, size }) => {
                let size_changed = item.size != size;
                let ptr_changed = match (item.value, NonNull::new(value)) {
                    (Some(old), Some(new)) => old != new,
                    (None, None) => false,
                    _ => true,
                };

                size_changed || ptr_changed
            }
            (FilterConfigItem::Group(_), FilterConfigItemValue::Group) => false,
            (FilterConfigItem::Button(_), FilterConfigItemValue::Button) => false,
            _ => {
                panic!("Mismatched filter config item type");
            }
        }
    }

    /// # Safety
    ///
    /// `raw` は有効なポインタである必要があります。
    pub(crate) unsafe fn apply_from_raw(&mut self, raw: *const aviutl2_sys::filter2::FILTER_ITEM) {
        let value = unsafe { Self::get_value(raw) };
        match (self, value) {
            (FilterConfigItem::Track(item), FilterConfigItemValue::Track(v)) => {
                item.value = v;
            }
            (FilterConfigItem::Checkbox(item), FilterConfigItemValue::Checkbox(v)) => {
                item.value = v;
            }
            (FilterConfigItem::Color(item), FilterConfigItemValue::Color(v)) => {
                item.value = v;
            }
            (FilterConfigItem::Select(item), FilterConfigItemValue::Select(v)) => {
                item.value = v;
            }
            (FilterConfigItem::File(item), FilterConfigItemValue::File(v)) => {
                item.value = v;
            }
            (FilterConfigItem::String(item), FilterConfigItemValue::String(v)) => {
                item.value = v;
            }
            (FilterConfigItem::Text(item), FilterConfigItemValue::Text(v)) => {
                item.value = v;
            }
            (FilterConfigItem::Folder(item), FilterConfigItemValue::Folder(v)) => {
                item.value = v;
            }
            (FilterConfigItem::Data(item), FilterConfigItemValue::Data { value, size }) => {
                item.size = size;
                item.value = NonNull::new(value);
            }
            (FilterConfigItem::Group(_), FilterConfigItemValue::Group) => {
                // グループは値を持たないので何もしない
            }
            (FilterConfigItem::Button(_), FilterConfigItemValue::Button) => {
                // ボタンは値を持たないので何もしない
            }
            _ => {
                panic!("Mismatched filter config item type");
            }
        }
    }
}

/// トラックバー。
#[derive(Debug, Clone)]
pub struct FilterConfigTrack {
    /// 設定名。
    pub name: String,

    /// 設定値。
    pub value: f64,

    /// 設定値の範囲。
    pub range: std::ops::RangeInclusive<f64>,

    /// 設定値の単位。
    pub step: FilterConfigTrackStep,
}

/// トラックバーの設定値の単位。
#[derive(Clone, Copy)]
pub enum FilterConfigTrackStep {
    /// 1.0
    One,
    /// 0.1
    PointOne,
    /// 0.01
    PointZeroOne,
    /// 0.001
    PointZeroZeroOne,
}
impl TryFrom<f64> for FilterConfigTrackStep {
    type Error = anyhow::Error;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value {
            1.0 => Ok(FilterConfigTrackStep::One),
            0.1 => Ok(FilterConfigTrackStep::PointOne),
            0.01 => Ok(FilterConfigTrackStep::PointZeroOne),
            0.001 => Ok(FilterConfigTrackStep::PointZeroZeroOne),
            _ => Err(anyhow::anyhow!("Invalid step value: {}", value)),
        }
    }
}
impl From<FilterConfigTrackStep> for f64 {
    fn from(value: FilterConfigTrackStep) -> Self {
        match value {
            FilterConfigTrackStep::One => 1.0,
            FilterConfigTrackStep::PointOne => 0.1,
            FilterConfigTrackStep::PointZeroOne => 0.01,
            FilterConfigTrackStep::PointZeroZeroOne => 0.001,
        }
    }
}
impl std::fmt::Debug for FilterConfigTrackStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: f64 = (*self).into();
        f.debug_tuple("FilterConfigTrackStep")
            .field(&value)
            .finish()
    }
}

/// チェックボックス。
#[derive(Debug, Clone)]
pub struct FilterConfigCheckbox {
    /// 設定名。
    pub name: String,

    /// 設定値。
    pub value: bool,
}

/// 色選択。
#[derive(Debug, Clone)]
pub struct FilterConfigColor {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: FilterConfigColorValue,
}

/// 色選択の設定値の色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FilterConfigColorValue(pub u32);
impl FilterConfigColorValue {
    /// 色をRGB形式の各成分に分解して取得します。
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let r = ((self.0 >> 16) & 0xFF) as u8;
        let g = ((self.0 >> 8) & 0xFF) as u8;
        let b = (self.0 & 0xFF) as u8;
        (r, g, b)
    }

    /// RGB形式の各成分から色を作成します。
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let value = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
        FilterConfigColorValue(value)
    }
}
impl From<u32> for FilterConfigColorValue {
    fn from(value: u32) -> Self {
        FilterConfigColorValue(value)
    }
}
impl From<FilterConfigColorValue> for u32 {
    fn from(value: FilterConfigColorValue) -> Self {
        value.0
    }
}
impl From<aviutl2_sys::filter2::FILTER_ITEM_COLOR_VALUE> for FilterConfigColorValue {
    fn from(value: aviutl2_sys::filter2::FILTER_ITEM_COLOR_VALUE) -> Self {
        unsafe { FilterConfigColorValue(value.code) }
    }
}
impl From<FilterConfigColorValue> for aviutl2_sys::filter2::FILTER_ITEM_COLOR_VALUE {
    fn from(value: FilterConfigColorValue) -> Self {
        aviutl2_sys::filter2::FILTER_ITEM_COLOR_VALUE { code: value.0 }
    }
}
impl std::fmt::Display for FilterConfigColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (r, g, b) = self.to_rgb();
        write!(f, "#{:02X}{:02X}{:02X}", r, g, b)
    }
}
impl std::fmt::LowerHex for FilterConfigColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06x}", self.0 & 0xFFFFFF)
    }
}
impl std::fmt::UpperHex for FilterConfigColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06X}", self.0 & 0xFFFFFF)
    }
}

/// 選択リスト。
#[derive(Debug, Clone)]
pub struct FilterConfigSelect {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: i32,
    /// 選択肢リスト。
    pub items: Vec<FilterConfigSelectItem>,
}

/// 選択リストの選択肢。
#[derive(Debug, Clone)]
pub struct FilterConfigSelectItem {
    /// 選択肢の名前。
    pub name: String,
    /// 選択肢の値。
    pub value: i32,
}

/// `[Vec<FilterConfigSelectItem>]`に変換したり、AviUtl2側の値から変換するためのトレイト。
///
/// 基本的にはこのトレイトを手動で実装する必要はありません。
/// [`derive@FilterConfigSelectItems`] マクロを使用してください。
///
/// <div class="warning">
///
/// [`FilterConfigSelect`]は[`Vec<FilterConfigSelectItems>`]との相互変換が可能であれば十分に機能します。
/// このトレイトを手動で実装する必要はありません。
///
/// </div>
///
/// # See Also
///
/// [derive@FilterConfigSelectItems]
pub trait FilterConfigSelectItems {
    /// [`Vec<FilterConfigSelectItem>`] に変換します。
    fn to_select_items() -> Vec<FilterConfigSelectItem>;

    /// [`i32`] から変換します。
    ///
    /// # Panics
    ///
    /// `item` の内容が不正な場合、パニックします。
    fn from_select_item_value(item: i32) -> Self;

    /// [`i32`] へ変換します。
    fn to_select_item_value(&self) -> i32;
}

#[doc(inline)]
pub use aviutl2_macros::FilterConfigSelectItems;

/// ファイル選択。
#[derive(Debug, Clone)]
pub struct FilterConfigFile {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: String,
    /// ファイルフィルタ。
    pub filters: Vec<crate::common::FileFilter>,
}

/// ボタン。
#[derive(Debug, Clone)]
pub struct FilterConfigButton {
    /// 設定名。
    pub name: String,
    /// コールバック関数。
    pub callback: extern "C" fn(*mut EDIT_SECTION),
}

/// 文字列。
#[derive(Debug, Clone)]
pub struct FilterConfigString {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: String,
}

/// テキスト。
#[derive(Debug, Clone)]
pub struct FilterConfigText {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: String,
}

/// フォルダ選択。
#[derive(Debug, Clone)]
pub struct FilterConfigFolder {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: String,
}

/// 型を消去した汎用データ。
///
/// # Warning
///
/// この型は型が全くついていません。
/// 基本的には[`FilterConfigData`]を使用してください。
#[derive(Debug, Clone)]
pub struct ErasedFilterConfigData {
    /// 設定名。
    pub name: String,
    /// データのサイズ。
    ///
    /// # Note
    ///
    /// 1024バイトを超えることはできません。
    pub size: usize,
    /// 現在の値を指すポインタ。
    pub value: Option<NonNull<std::ffi::c_void>>,
    default_value: [u8; 1024],
}

impl ErasedFilterConfigData {
    /// 新しく作成します。
    /// `value` は `None` になります。
    ///
    /// # Panics
    ///
    /// Tが1024バイトを超える場合、パニックします。
    pub fn new<T: Copy + Default + 'static>(name: String) -> Self {
        Self::with_default_value(name, T::default())
    }

    /// デフォルト値を指定して新しく作成します。
    /// `value` は `None` になります。
    ///
    /// # Panics
    ///
    /// Tが1024バイトを超える場合、パニックします。
    pub fn with_default_value<T: Copy + 'static>(name: String, default_value: T) -> Self {
        assert!(
            std::mem::size_of::<T>() <= 1024,
            "FilterConfigData<T> size must be <= 1024 bytes"
        );
        let size = std::mem::size_of::<T>();
        let mut default_value_bytes = [0u8; 1024];
        let default_value_ptr = (&raw const default_value).cast::<u8>();
        default_value_bytes[..size]
            .copy_from_slice(unsafe { std::slice::from_raw_parts(default_value_ptr, size) });

        ErasedFilterConfigData {
            name,
            size,
            value: None,
            default_value: default_value_bytes,
        }
    }

    /// デフォルト値のスライスを取得します。
    pub fn default_value(&self) -> &[u8] {
        &self.default_value[..self.size]
    }

    /// 型付きの汎用データに変換します。
    ///
    /// # Safety
    ///
    /// - `self` を消去する前の型Tと同じ型で呼び出す必要があります。
    /// - Tのサイズが`self.size`と一致している必要があります。
    /// - `self.value`が指すポインタが有効である必要があります。
    /// - `self.default_value`はTとして有効なデータである必要があります。
    pub unsafe fn into_typed<T: Copy + 'static>(self) -> FilterConfigData<T> {
        let expected_size = std::mem::size_of::<T>();
        assert_eq!(
            self.size, expected_size,
            "Size mismatch when converting ErasedFilterConfigData to FilterConfigData<T>"
        );
        let value = self
            .value
            .map(|v| NonNull::new(v.as_ptr().cast::<T>()).unwrap());
        let default_value_ptr = self.default_value.as_ptr().cast::<T>();
        let default_value = unsafe { *default_value_ptr };
        FilterConfigData {
            name: self.name,
            value,
            default_value,
        }
    }
}

/// 汎用データ。
///
/// # Note
///
/// Tのサイズが変わったとき、値はデフォルト値にリセットされます。
#[derive(Debug, Clone)]
pub struct FilterConfigData<T: Copy + 'static> {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: Option<NonNull<T>>,
    /// デフォルト値。
    pub default_value: T,
}

impl<T: Copy + 'static> FilterConfigData<T> {
    /// 型を消去した汎用データに変換します。
    ///
    /// # Panics
    ///
    /// Tが1024バイトを超える場合、パニックします。
    pub fn erase_type(&self) -> ErasedFilterConfigData {
        assert!(
            std::mem::size_of::<T>() <= 1024,
            "FilterConfigData<T> size must be <= 1024 bytes"
        );
        let size = std::mem::size_of::<T>();
        let mut default_value = [0u8; 1024];
        let default_value_ptr = (&raw const self.default_value).cast::<u8>();
        default_value[..size]
            .copy_from_slice(unsafe { std::slice::from_raw_parts(default_value_ptr, size) });

        ErasedFilterConfigData {
            name: self.name.clone(),
            size,
            value: self
                .value
                .map(|v| NonNull::new(v.as_ptr().cast::<c_void>()).unwrap()),
            default_value,
        }
    }
}

impl<T: Copy + 'static> From<FilterConfigData<T>> for ErasedFilterConfigData {
    fn from(value: FilterConfigData<T>) -> Self {
        value.erase_type()
    }
}

/// グループ。
#[derive(Debug, Clone)]
pub struct FilterConfigGroup {
    /// 設定名。
    /// Noneの場合、グループの終端として扱われます。
    pub name: Option<String>,

    /// デフォルトで開いているかどうか。
    pub opened: bool,
}

impl FilterConfigGroup {
    /// グループの開始を表す設定を作成します。
    pub fn start(name: String) -> Self {
        Self::start_with_opened(name, true)
    }

    /// `opened` を指定してグループの開始を表す設定を作成します。
    pub fn start_with_opened(name: String, opened: bool) -> Self {
        FilterConfigGroup {
            name: Some(name),
            opened,
        }
    }

    /// グループの終了を表す設定を作成します。
    pub fn end() -> Self {
        FilterConfigGroup {
            name: None,
            opened: false,
        }
    }
}
