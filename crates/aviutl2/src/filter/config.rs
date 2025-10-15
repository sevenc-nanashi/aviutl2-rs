use crate::common::LeakManager;

/// [`Vec<FilterConfigItem>`] と相互変換するためのトレイト。
/// 基本的にはこのトレイトを手動で実装する必要はありません。
/// [`derive@FilterConfigItems`] マクロを使用してください。
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
/// [derive@FilterConfigItems]
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
pub use aviutl2_macros::FilterConfigItems;

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
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterConfigItemValue {
    Track(f64),
    Checkbox(bool),
    Color(FilterConfigColorValue),
    Select(i32),
    File(String),
}

impl FilterConfigItem {
    /// 設定名を取得します。
    pub fn name(&self) -> &str {
        match self {
            FilterConfigItem::Track(item) => &item.name,
            FilterConfigItem::Checkbox(item) => &item.name,
            FilterConfigItem::Color(item) => &item.name,
            FilterConfigItem::Select(item) => &item.name,
            FilterConfigItem::File(item) => &item.name,
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
                *(raw as *const aviutl2_sys::common::LPCWSTR),
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
