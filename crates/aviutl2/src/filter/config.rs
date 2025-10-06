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
#[derive(Debug, Clone, Copy)]
pub struct FilterConfigColorValue(pub u32);
impl FilterConfigColorValue {
    /// 色をBGR形式の各成分に分解して取得します。
    pub fn to_bgr(&self) -> (u8, u8, u8) {
        let b = (self.0 & 0x0000FF) as u8;
        let g = ((self.0 & 0x00FF00) >> 8) as u8;
        let r = ((self.0 & 0xFF0000) >> 16) as u8;
        (b, g, r)
    }

    /// BGR形式の各成分から色を作成します。
    pub fn from_bgr(b: u8, g: u8, r: u8) -> Self {
        let value = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
        FilterConfigColorValue(value)
    }

    /// 色をRGB形式の各成分に分解して取得します。
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let r = ((self.0 & 0xFF0000) >> 16) as u8;
        let g = ((self.0 & 0x00FF00) >> 8) as u8;
        let b = (self.0 & 0x0000FF) as u8;
        (r, g, b)
    }

    /// RGB形式の各成分から色を作成します。
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let value = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
        FilterConfigColorValue(value)
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

/// ファイル選択。
#[derive(Debug, Clone)]
pub struct FilterConfigFile {
    /// 設定名。
    pub name: String,
    /// 設定値。
    pub value: String,
    /// ファイルフィルタ。
    pub filefilter: crate::common::FileFilter,
}
