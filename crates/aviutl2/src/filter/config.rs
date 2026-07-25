use crate::common::LeakManager;
use aviutl2_sys::plugin2::EDIT_SECTION;
use parking_lot::lock_api::RawRwLock;
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
    fn to_config_items() -> Vec<crate::filter::FilterConfigItem>;

    /// [`Vec<FilterConfigItem>`] から変換します。
    ///
    /// # Panics
    ///
    /// `items` の内容が不正な場合、パニックします。
    fn from_config_items(items: &[crate::filter::FilterConfigItem]) -> Self;
}
#[doc(inline)]
pub use aviutl2_macros::filter_config_items;

/// `&[FilterConfigItem]` に対する拡張トレイト。
pub trait FilterConfigItemSliceExt {
    /// `&[FilterConfigItem]` から指定した構造体に変換します。
    fn to_struct<T: crate::filter::FilterConfigItems>(&self) -> T;
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
    /// セクションごとのチェックボックス。
    CheckSection(FilterConfigCheckSection),
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
    /// セパレーター。
    Separator(FilterConfigSeparator),
    /// ボタン。
    Button(FilterConfigButton),
    /// トラックバーグループ。
    TrackGroup(FilterConfigTrackGroup),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterConfigItemValue {
    Track(f64),
    Checkbox(bool),
    CheckSection(bool),
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
    TrackGroup,
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
            FilterConfigItem::Separator(item) => &item.name,
            FilterConfigItem::Button(item) => &item.name,
            FilterConfigItem::CheckSection(item) => &item.name,
            FilterConfigItem::TrackGroup(item) => &item.name,
        }
    }

    pub(crate) fn to_raw(&self, leak_manager: &LeakManager) -> aviutl2_sys::filter2::FILTER_ITEM {
        match self {
            FilterConfigItem::Track(item) => aviutl2_sys::filter2::FILTER_ITEM {
                track: aviutl2_sys::filter2::FILTER_ITEM_TRACK {
                    r#type: leak_manager.leak_as_wide_string("track2"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    value: item.value,
                    s: *item.range.start(),
                    e: *item.range.end(),
                    step: item.step,
                    zero_display: item
                        .zero_display
                        .as_ref()
                        .map_or(std::ptr::null(), |s| leak_manager.leak_as_wide_string(s)),
                    slider_ratio: item.slider_ratio,
                },
            },
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
                    default_value: [MaybeUninit::new(0); 16 * 1024],
                };
                assert!(
                    item.size <= 16 * 1024,
                    "FilterConfigData size must be <= 16 KiB"
                );
                unsafe {
                    // SAFETY:
                    // - item.size <= 16KiBかつ、
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
            FilterConfigItem::Separator(item) => aviutl2_sys::filter2::FILTER_ITEM {
                separator: aviutl2_sys::filter2::FILTER_ITEM_SEPARATOR {
                    r#type: leak_manager.leak_as_wide_string("separator"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                },
            },
            FilterConfigItem::Button(item) => aviutl2_sys::filter2::FILTER_ITEM {
                button: aviutl2_sys::filter2::FILTER_ITEM_BUTTON {
                    r#type: leak_manager.leak_as_wide_string("button"),
                    name: leak_manager.leak_as_wide_string(&item.name),
                    callback: item.callback,
                },
            },
            FilterConfigItem::CheckSection(filter_config_check_section) => {
                aviutl2_sys::filter2::FILTER_ITEM {
                    check_section: aviutl2_sys::filter2::FILTER_ITEM_CHECK_SECTION {
                        r#type: leak_manager.leak_as_wide_string("checksection2"),
                        name: leak_manager.leak_as_wide_string(&filter_config_check_section.name),
                        value: filter_config_check_section.value,
                        multi_section: filter_config_check_section.multi_section,
                    },
                }
            }
            FilterConfigItem::TrackGroup(filter_config_track_group) => {
                let mut tracks = Vec::new();
                for item in &filter_config_track_group.tracks {
                    let raw = FilterConfigItem::to_raw(
                        &FilterConfigItem::Track(item.clone()),
                        leak_manager,
                    );
                    tracks.push(leak_manager.leak(raw));
                }
                tracks.push(std::ptr::null_mut());

                let pointers = leak_manager.leak_value_vec(tracks);

                aviutl2_sys::filter2::FILTER_ITEM {
                    track_group: aviutl2_sys::filter2::FILTER_ITEM_TRACK_GROUP {
                        r#type: leak_manager.leak_as_wide_string("trackgroup"),
                        name: leak_manager.leak_as_wide_string(&filter_config_track_group.name),
                        tracks: pointers as _,
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
                *(raw.cast::<aviutl2_sys::common::LPCWSTR>()),
            )
        };
        match item_type.as_str() {
            "track" | "track2" => {
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
                    size <= 16 * 1024,
                    "FILTER_ITEM_DATA size must be 16 KiB or less"
                );
                FilterConfigItemValue::Data {
                    value: raw_data,
                    size,
                }
            }
            "group" => FilterConfigItemValue::Group,
            "button" => FilterConfigItemValue::Button,
            "checksection" | "checksection2" => {
                let raw_check_section = unsafe { &(*raw).check_section };
                FilterConfigItemValue::CheckSection(raw_check_section.value)
            }
            "trackgroup" => FilterConfigItemValue::TrackGroup,
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

                let binding_changed = item.variable_state.as_ref().is_some_and(|state| {
                    let state = state.read();
                    !matches!(
                        *state,
                        VariableLengthDataBacking::Host(pointer)
                            if std::ptr::eq(
                                pointer.as_ptr(),
                                raw.cast_mut().cast::<aviutl2_sys::filter2::FILTER_ITEM_DATA>()
                            )
                    )
                });

                size_changed || ptr_changed || binding_changed
            }
            (FilterConfigItem::Group(_), FilterConfigItemValue::Group) => false,
            (FilterConfigItem::Button(_), FilterConfigItemValue::Button) => false,
            (FilterConfigItem::CheckSection(item), FilterConfigItemValue::CheckSection(v)) => {
                item.value != v
            }
            (FilterConfigItem::TrackGroup(_), FilterConfigItemValue::TrackGroup) => false,
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
                if let Some(state) = &item.variable_state {
                    let raw = NonNull::new(
                        raw.cast_mut()
                            .cast::<aviutl2_sys::filter2::FILTER_ITEM_DATA>(),
                    )
                    .expect("FILTER_ITEM_DATA pointer must not be null");
                    *state.write() = VariableLengthDataBacking::Host(raw);
                }
            }
            (FilterConfigItem::Group(_), FilterConfigItemValue::Group) => {
                // グループは値を持たないので何もしない
            }
            (FilterConfigItem::Button(_), FilterConfigItemValue::Button) => {
                // ボタンは値を持たないので何もしない
            }
            (FilterConfigItem::CheckSection(item), FilterConfigItemValue::CheckSection(v)) => {
                item.value = v;
            }
            (FilterConfigItem::TrackGroup(_), FilterConfigItemValue::TrackGroup) => {
                // トラックバーグループは値を持たないので何もしない
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
    pub step: f64,

    /// 0のときに表示する文字列。
    pub zero_display: Option<String>,

    /// スライダーの値の範囲に対するつまみの操作範囲の割合。
    pub slider_ratio: f64,
}

/// チェックボックス。
#[derive(Debug, Clone)]
pub struct FilterConfigCheckbox {
    /// 設定名。
    pub name: String,

    /// 設定値。
    pub value: bool,
}

/// セクションごとのチェックボックス。
#[derive(Debug, Clone)]
pub struct FilterConfigCheckSection {
    /// 設定名。
    pub name: String,

    /// 設定値。
    pub value: bool,

    /// デフォルトでセクションごとのチェックボックスにするかどうか。
    pub multi_section: bool,
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
    fn to_select_items() -> Vec<crate::filter::FilterConfigSelectItem>;

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
    /// 16KiBを超えるデータはサポートされません。
    pub size: usize,
    /// 現在の値を指すポインタ。
    pub value: Option<NonNull<std::ffi::c_void>>,
    default_value: [u8; 16 * 1024],
    variable_state: Option<std::sync::Arc<parking_lot::RwLock<VariableLengthDataBacking>>>,
}

impl ErasedFilterConfigData {
    /// 新しく作成します。
    /// `value` は `None` になります。
    ///
    /// # Panics
    ///
    /// Tが16KiBを超える場合、パニックします。
    pub fn new<T: Copy + Default + 'static>(name: String) -> Self {
        Self::with_default_value(name, T::default())
    }

    /// デフォルト値を指定して新しく作成します。
    /// `value` は `None` になります。
    ///
    /// # Panics
    ///
    /// Tが16KiBを超える場合、パニックします。
    pub fn with_default_value<T: Copy + 'static>(name: String, default_value: T) -> Self {
        assert!(
            std::mem::size_of::<T>() <= 16 * 1024,
            "FilterConfigData<T> size must be <= 16 KiB"
        );
        let size = std::mem::size_of::<T>();
        let mut default_value_bytes = [0u8; 16 * 1024];
        let default_value_ptr = (&raw const default_value).cast::<u8>();
        default_value_bytes[..size]
            .copy_from_slice(unsafe { std::slice::from_raw_parts(default_value_ptr, size) });

        ErasedFilterConfigData {
            name,
            size,
            value: None,
            default_value: default_value_bytes,
            variable_state: None,
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
/// [`VariableLengthFilterConfigData`]と違い、サイズが固定されています。
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
            std::mem::size_of::<T>() <= 16 * 1024,
            "FilterConfigData<T> size must be <= 16 KiB"
        );
        let size = std::mem::size_of::<T>();
        let mut default_value = [0u8; 16 * 1024];
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
            variable_state: None,
        }
    }
}

impl<T: Copy + 'static> From<FilterConfigData<T>> for ErasedFilterConfigData {
    fn from(value: FilterConfigData<T>) -> Self {
        value.erase_type()
    }
}

/// 可変長の汎用データ。
///
/// [`FilterConfigData`]と違い、サイズを変更することができます。
/// 必要に応じて、`bytemuck`などのクレートを使用して型変換してください。
#[derive(Debug, Clone)]
pub struct VariableLengthFilterConfigData {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: Option<NonNull<[u8]>>,
    /// データのサイズ。
    pub size: usize,
    /// デフォルト値。
    pub default_value: Vec<u8>,
}

impl VariableLengthFilterConfigData {
    /// 新しく作成します。
    ///
    /// # Panics
    ///
    /// `size` が16KiBを超える場合、パニックします。
    pub fn new(name: String, size: usize) -> Self {
        Self::with_default_value(name, vec![0u8; size])
    }

    /// デフォルト値を指定して新しく作成します。
    ///
    /// # Panics
    ///
    /// `default_value.len()` が16KiBを超える場合、パニックします。
    pub fn with_default_value(name: String, default_value: Vec<u8>) -> Self {
        assert!(
            default_value.len() <= 16 * 1024,
            "VariableLengthFilterConfigData default_value size must be <= 16 KiB"
        );
        let size = default_value.len();
        VariableLengthFilterConfigData {
            name,
            value: None,
            size,
            default_value,
        }
    }
}
impl From<VariableLengthFilterConfigData> for ErasedFilterConfigData {
    fn from(value: VariableLengthFilterConfigData) -> Self {
        let size = value.size;
        let mut default_value = [0u8; 16 * 1024];
        assert!(
            size <= 16 * 1024,
            "VariableLengthFilterConfigData size must be <= 16 KiB"
        );
        assert_eq!(
            size,
            value.default_value.len(),
            "VariableLengthFilterConfigData size must match default_value length"
        );
        default_value[..size].copy_from_slice(&value.default_value);
        let backing = match value.value {
            Some(value) => VariableLengthDataBacking::Borrowed {
                pointer: value.as_ptr().cast(),
                size,
            },
            None => VariableLengthDataBacking::Owned(value.default_value),
        };

        ErasedFilterConfigData {
            name: value.name,
            size,
            value: None,
            default_value,
            variable_state: Some(std::sync::Arc::new(parking_lot::RwLock::new(backing))),
        }
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

/// セパレーター。
#[derive(Debug, Clone)]
pub struct FilterConfigSeparator {
    /// セパレーターに表示するテキスト。
    pub name: String,
}

static HANDLES: std::sync::LazyLock<dashmap::DashMap<usize, parking_lot::RawRwLock>> =
    std::sync::LazyLock::new(dashmap::DashMap::new);
static OWNED_REFERENCES: std::sync::LazyLock<
    std::sync::Arc<dashmap::DashMap<usize, std::sync::atomic::AtomicUsize>>,
> = std::sync::LazyLock::new(|| std::sync::Arc::new(dashmap::DashMap::new()));

#[derive(Debug)]
enum VariableLengthDataBacking {
    Borrowed { pointer: *mut u8, size: usize },
    Owned(Vec<u8>),
    Host(NonNull<aviutl2_sys::filter2::FILTER_ITEM_DATA>),
}

unsafe impl Send for VariableLengthDataBacking {}
unsafe impl Sync for VariableLengthDataBacking {}

impl VariableLengthDataBacking {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Borrowed { pointer, size } => unsafe {
                std::slice::from_raw_parts(non_null_slice_pointer(*pointer, *size), *size)
            },
            Self::Owned(value) => value,
            Self::Host(raw) => {
                let raw = unsafe { raw.as_ref() };
                let size =
                    usize::try_from(raw.size).expect("FILTER_ITEM_DATA size must not be negative");
                assert!(size <= 16 * 1024, "FILTER_ITEM_DATA size must be <= 16 KiB");
                unsafe {
                    std::slice::from_raw_parts(non_null_slice_pointer(raw.value.cast(), size), size)
                }
            }
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Self::Borrowed { pointer, size } => unsafe {
                std::slice::from_raw_parts_mut(non_null_slice_pointer(*pointer, *size), *size)
            },
            Self::Owned(value) => value,
            Self::Host(raw) => {
                let raw = unsafe { raw.as_mut() };
                let size =
                    usize::try_from(raw.size).expect("FILTER_ITEM_DATA size must not be negative");
                assert!(size <= 16 * 1024, "FILTER_ITEM_DATA size must be <= 16 KiB");
                unsafe {
                    std::slice::from_raw_parts_mut(
                        non_null_slice_pointer(raw.value.cast(), size),
                        size,
                    )
                }
            }
        }
    }
}

fn non_null_slice_pointer(pointer: *mut u8, size: usize) -> *mut u8 {
    if size == 0 {
        NonNull::<u8>::dangling().as_ptr()
    } else {
        assert!(
            !pointer.is_null(),
            "filter config data pointer must not be null"
        );
        pointer
    }
}

/// フィルタプラグインで可変長の汎用データ項目を使うためのハンドル。
#[derive(Debug, Clone)]
pub struct VariableLengthFilterConfigDataHandle {
    inner: std::sync::Arc<parking_lot::RwLock<VariableLengthDataBacking>>,
}

impl VariableLengthFilterConfigDataHandle {
    #[doc(hidden)]
    pub fn __from_erased(erased: &ErasedFilterConfigData) -> Self {
        Self {
            inner: erased
                .variable_state
                .as_ref()
                .expect("expected variable-length filter config data")
                .clone(),
        }
    }

    #[doc(hidden)]
    pub fn __new_owned(value: Vec<u8>) -> Self {
        assert!(
            value.len() <= 16 * 1024,
            "VariableLengthFilterConfigData size must be <= 16 KiB"
        );
        Self {
            inner: std::sync::Arc::new(parking_lot::RwLock::new(VariableLengthDataBacking::Owned(
                value,
            ))),
        }
    }

    /// データを読み取るためのロックを取得する。
    pub fn read(&self) -> VariableLengthFilterConfigDataReadGuard<'_> {
        VariableLengthFilterConfigDataReadGuard {
            inner: self.inner.read(),
        }
    }

    /// データを読み取るためのロックの取得を試みる。
    pub fn try_read(&self) -> Option<VariableLengthFilterConfigDataReadGuard<'_>> {
        self.inner
            .try_read()
            .map(|inner| VariableLengthFilterConfigDataReadGuard { inner })
    }

    /// データを書き込むためのロックを取得する。
    pub fn write(&self) -> VariableLengthFilterConfigDataWriteGuard<'_> {
        VariableLengthFilterConfigDataWriteGuard {
            inner: self.inner.write(),
        }
    }

    /// データを書き込むためのロックの取得を試みる。
    pub fn try_write(&self) -> Option<VariableLengthFilterConfigDataWriteGuard<'_>> {
        self.inner
            .try_write()
            .map(|inner| VariableLengthFilterConfigDataWriteGuard { inner })
    }

    pub(crate) fn try_resize(
        &mut self,
        size: usize,
        resize: impl FnOnce(*mut c_void, i32),
    ) -> crate::filter::FilterProcResult<()> {
        if size > 16 * 1024 {
            return Err(crate::filter::FilterProcError::ValueOutOfRange);
        }
        let size =
            i32::try_from(size).map_err(|_| crate::filter::FilterProcError::ValueOutOfRange)?;
        let mut backing = self
            .inner
            .try_write()
            .ok_or(crate::filter::FilterProcError::FilterConfigDataLocked)?;
        let VariableLengthDataBacking::Host(raw) = &mut *backing else {
            return Err(crate::filter::FilterProcError::FilterConfigDataNotBound);
        };
        resize(raw.as_ptr().cast(), size);
        let backing = &mut *backing;
        match backing {
            VariableLengthDataBacking::Host(raw) => {
                unsafe { raw.as_mut() }.size = size;
            }
            VariableLengthDataBacking::Borrowed {
                pointer: _,
                size: s,
            } => {
                *s = size as usize;
            }
            VariableLengthDataBacking::Owned(vec) => {
                vec.resize(size as usize, 0);
            }
        }
        Ok(())
    }
}

/// 可変長の汎用データを読み取るためのガード。
pub struct VariableLengthFilterConfigDataReadGuard<'handle> {
    inner: parking_lot::RwLockReadGuard<'handle, VariableLengthDataBacking>,
}

impl std::ops::Deref for VariableLengthFilterConfigDataReadGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

/// 可変長の汎用データを書き込むためのガード。
pub struct VariableLengthFilterConfigDataWriteGuard<'handle> {
    inner: parking_lot::RwLockWriteGuard<'handle, VariableLengthDataBacking>,
}

impl std::ops::Deref for VariableLengthFilterConfigDataWriteGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

impl std::ops::DerefMut for VariableLengthFilterConfigDataWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut_slice()
    }
}

/// フィルタプラグインで固定長の汎用データ項目を使うためのハンドル。
/// RwLockのような仕組みで安全にデータを扱うことができます。
#[derive(Debug)]
pub struct FilterConfigDataHandle<T: Copy> {
    pub(crate) inner: *mut T,
}

unsafe impl<T: Send + Sync + Copy> Send for FilterConfigDataHandle<T> {}
unsafe impl<T: Send + Sync + Copy> Sync for FilterConfigDataHandle<T> {}

impl<T: Copy> Clone for FilterConfigDataHandle<T> {
    fn clone(&self) -> Self {
        if !self.inner.is_null() {
            let addr = self.inner as usize;
            if let Some(entry) = OWNED_REFERENCES.get(&addr) {
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

    #[inline]
    fn inner_ptr(&self) -> Option<*mut T> {
        if self.inner.is_null() {
            None
        } else {
            Some(self.inner)
        }
    }

    #[inline]
    fn inner_addr(&self) -> Option<usize> {
        self.inner_ptr().map(|inner| inner as *mut () as usize)
    }

    /// データを読み取るためのロックを取得する。
    pub fn read<'handle>(&'handle self) -> FilterConfigDataReadGuard<'handle, T> {
        let inner = self
            .inner_ptr()
            .expect("FilterConfigDataHandle::read called with a null inner pointer");
        let addr = self
            .inner_addr()
            .expect("FilterConfigDataHandle::read called with a null inner pointer");
        let lock = HANDLES
            .entry(addr)
            .or_insert_with(|| parking_lot::RawRwLock::INIT);
        let lock = lock.value();

        lock.lock_shared();
        FilterConfigDataReadGuard::new(inner)
    }

    /// データを読み取るためのロックの取得を試みる。
    /// ロックが取得できなかった場合は `None` を返します。
    pub fn try_read<'handle>(&'handle self) -> Option<FilterConfigDataReadGuard<'handle, T>> {
        let inner = self.inner_ptr()?;
        let addr = self.inner_addr()?;
        let lock = HANDLES
            .entry(addr)
            .or_insert_with(|| parking_lot::RawRwLock::INIT);
        let lock = lock.value();

        if lock.try_lock_shared() {
            Some(FilterConfigDataReadGuard::new(inner))
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

#[doc(hidden)]
pub trait FilterConfigDataHandleType: Sized {
    type Value;

    fn __default_value() -> Self::Value
    where
        Self::Value: Default,
    {
        Self::Value::default()
    }

    fn __to_erased(name: String, value: Self::Value) -> ErasedFilterConfigData;
    fn __from_erased_data(data: &ErasedFilterConfigData) -> Self;
    fn __new_owned_data(value: Self::Value) -> Self;
}

impl<T: Copy + 'static> FilterConfigDataHandleType for FilterConfigDataHandle<T> {
    type Value = T;

    fn __to_erased(name: String, value: Self::Value) -> ErasedFilterConfigData {
        ErasedFilterConfigData::with_default_value(name, value)
    }

    fn __from_erased_data(data: &ErasedFilterConfigData) -> Self {
        Self::__from_erased(data)
    }

    fn __new_owned_data(value: Self::Value) -> Self {
        Self::__new_owned(value)
    }
}

impl FilterConfigDataHandleType for VariableLengthFilterConfigDataHandle {
    type Value = Vec<u8>;

    fn __to_erased(name: String, value: Self::Value) -> ErasedFilterConfigData {
        VariableLengthFilterConfigData::with_default_value(name, value).into()
    }

    fn __from_erased_data(data: &ErasedFilterConfigData) -> Self {
        Self::__from_erased(data)
    }

    fn __new_owned_data(value: Self::Value) -> Self {
        Self::__new_owned(value)
    }
}

/// トラックバーグループ。
///
/// # Note
///
/// SDKではトラックグループのトラックは同じFILTER_ITEM_TRACKへのポインタを渡すことが想定されていますが、
/// 実装上の都合で同じ内容の別のFILTER_ITEM_TRACKへのポインタを渡すようになっています。
/// （これをちゃんとやろうとするとかなり複雑になるため...）
#[derive(Debug, Clone)]
pub struct FilterConfigTrackGroup {
    /// グループ名。
    pub name: String,
    /// トラックバーのリスト。
    pub tracks: Vec<FilterConfigTrack>,
}

#[doc(hidden)]
#[expect(private_bounds)]
pub fn __string_to_pathbuf_or_option_pathbuf<T: StringToPathBufOrOptionPathBuf>(s: &str) -> T {
    T::__string_to_pathbuf_or_option_pathbuf(s)
}

trait StringToPathBufOrOptionPathBuf: Sized {
    fn __string_to_pathbuf_or_option_pathbuf(s: &str) -> Self;
}
impl StringToPathBufOrOptionPathBuf for std::path::PathBuf {
    fn __string_to_pathbuf_or_option_pathbuf(s: &str) -> Self {
        std::path::PathBuf::from(s)
    }
}
impl StringToPathBufOrOptionPathBuf for Option<std::path::PathBuf> {
    fn __string_to_pathbuf_or_option_pathbuf(s: &str) -> Self {
        if s.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(s))
        }
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

    #[test]
    fn variable_length_filter_config_data_handle_reads_and_writes_owned_data() {
        let handle = VariableLengthFilterConfigDataHandle::__new_owned(vec![1, 2, 3]);
        let cloned = handle.clone();

        assert_eq!(&*handle.read(), &[1, 2, 3]);
        {
            let mut value = cloned.write();
            value[1] = 9;
        }
        assert_eq!(&*handle.read(), &[1, 9, 3]);
    }

    #[test]
    fn variable_length_filter_config_data_resize_updates_all_clones() {
        let mut old_value = vec![1u8, 2, 3];
        let mut new_value = vec![4u8, 5, 6, 7];
        let mut raw = aviutl2_sys::filter2::FILTER_ITEM_DATA {
            r#type: std::ptr::null(),
            name: std::ptr::null(),
            value: old_value.as_mut_ptr().cast(),
            size: old_value.len() as i32,
            default_value: [MaybeUninit::new(0); 16 * 1024],
        };
        let mut handle = VariableLengthFilterConfigDataHandle {
            inner: std::sync::Arc::new(parking_lot::RwLock::new(VariableLengthDataBacking::Host(
                NonNull::from(&mut raw),
            ))),
        };
        let cloned = handle.clone();

        handle
            .try_resize(new_value.len(), |raw, size| {
                let raw = unsafe { &mut *raw.cast::<aviutl2_sys::filter2::FILTER_ITEM_DATA>() };
                raw.value = new_value.as_mut_ptr().cast();
                raw.size = size;
            })
            .unwrap();

        assert_eq!(&*cloned.read(), &[4, 5, 6, 7]);
    }

    #[test]
    fn variable_length_filter_config_data_rejects_oversized_resize() {
        let mut handle = VariableLengthFilterConfigDataHandle::__new_owned(Vec::new());

        let result = handle.try_resize(16 * 1024 + 1, |_, _| panic!("resize must not be called"));

        assert!(matches!(
            result,
            Err(crate::filter::FilterProcError::ValueOutOfRange)
        ));
    }
}
