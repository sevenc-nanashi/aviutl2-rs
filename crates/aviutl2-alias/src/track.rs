bitflags::bitflags! {
    /// トラックバーのフラグ。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TrackFlags: u8 {
        /// 「加速」相当。
        const EASE_IN = 0b0000_0001;
        /// 「減速」相当。
        const EASE_OUT = 0b0000_0010;
        /// 「中間点無視」相当。
        const TWOPOINTS = 0b0000_0100;
        /// 設定値のあるトラック項目であることを示すフラグ。
        const PARAMETER = 0b0000_1000;
    }
}

/// トラックバーの移動単位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackStep {
    /// 1。
    One,
    /// 0.1。
    PointOne,
    /// 0.01。
    PointZeroOne,
    /// 0.001。
    PointZeroZeroOne,
}

impl TryFrom<f64> for TrackStep {
    type Error = ();

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value {
            1.0 => Ok(TrackStep::One),
            0.1 => Ok(TrackStep::PointOne),
            0.01 => Ok(TrackStep::PointZeroOne),
            0.001 => Ok(TrackStep::PointZeroZeroOne),
            _ => Err(()),
        }
    }
}
impl From<TrackStep> for f64 {
    fn from(step: TrackStep) -> Self {
        match step {
            TrackStep::One => 1.0,
            TrackStep::PointOne => 0.1,
            TrackStep::PointZeroOne => 0.01,
            TrackStep::PointZeroZeroOne => 0.001,
        }
    }
}
impl TrackStep {
    /// トラックバーのステップ値を取得します。
    pub fn value(&self) -> f64 {
        (*self).into()
    }
}

impl std::fmt::Display for TrackStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackStep::One => write!(f, "1"),
            TrackStep::PointOne => write!(f, "0.1"),
            TrackStep::PointZeroOne => write!(f, "0.01"),
            TrackStep::PointZeroZeroOne => write!(f, "0.001"),
        }
    }
}
impl std::fmt::Debug for TrackStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrackStep::{}", f64::from(*self))
    }
}

/// 時間制御のカーブ。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeCurve {
    /// カーブの制御点。
    pub control_points: Vec<TimeCurvePoint>,
}

/// 時間制御カーブの制御点。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeCurvePoint {
    /// 時間軸の位置（0.0〜1.0）。
    pub position: f64,
    /// 値（0.0〜1.0）。
    pub value: f64,
    /// 右ハンドルの相対位置。
    ///
    /// # Note
    ///
    /// 左ハンドルはこの点の数値を反転した位置にあります。
    pub right_handle: (f64, f64),
}

impl Default for TimeCurve {
    fn default() -> Self {
        TimeCurve {
            control_points: vec![
                TimeCurvePoint {
                    position: 0.0,
                    value: 0.0,
                    right_handle: (0.25, 0.25),
                },
                TimeCurvePoint {
                    position: 1.0,
                    value: 1.0,
                    right_handle: (0.25, 0.25),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum TimeCurveParseError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("invalid number of components ({0})")]
    InvalidComponentCount(usize),

    #[error("value out of range")]
    ValueOutOfRange,

    #[error("control points are not in increasing order")]
    BadPositionOrder,
}

impl std::str::FromStr for TimeCurve {
    type Err = TimeCurveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<f64> = s
            .split(',')
            .map(|part| {
                part.parse::<f64>()
                    .map_err(|_| TimeCurveParseError::InvalidFormat)
            })
            .collect::<Result<_, _>>()?;
        match parts.len() {
            n if n % 4 != 0 => Err(TimeCurveParseError::InvalidComponentCount(n)),
            0 => Ok(TimeCurve::default()),
            4 => Ok(TimeCurve {
                control_points: vec![
                    TimeCurvePoint {
                        position: 0.0,
                        value: 0.0,
                        right_handle: (parts[0], parts[1]),
                    },
                    TimeCurvePoint {
                        position: 1.0,
                        value: 1.0,
                        right_handle: (parts[2], parts[3]),
                    },
                ],
            }),
            _ => {
                let control_points = parts
                    .chunks(4)
                    .map(|chunk| {
                        if chunk[0] < 0.0
                            || chunk[0] > 1.0
                            || chunk[1] < 0.0
                            || chunk[1] > 1.0
                            || chunk[2] < 0.0
                        {
                            return Err(TimeCurveParseError::ValueOutOfRange);
                        }
                        Ok(TimeCurvePoint {
                            position: chunk[0],
                            value: chunk[1],
                            right_handle: (chunk[2], chunk[3]),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if !control_points
                    .windows(2)
                    .all(|w| w[0].position < w[1].position)
                {
                    return Err(TimeCurveParseError::BadPositionOrder);
                }
                Ok(TimeCurve { control_points })
            }
        }
    }
}

/// トラックバー項目。
pub struct TrackItem {
    /// フラグ。
    pub flags: TrackFlags,
    /// 移動単位。
    pub step: TrackStep,

    /// 中間点ごとの値。
    pub values: Vec<f64>,

    /// 設定値。
    pub parameter: Option<f64>,

    /// 時間制御のカーブ。
    pub time_curve: Option<TimeCurve>,
}

impl std::str::FromStr for TrackItem {
    type Err = TimeCurveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let time_curve = if s.is_empty() {
            None
        } else {
            Some(s.parse::<TimeCurve>()?)
        };
        Ok(TrackItem {
            flags: TrackFlags::empty(),
            step: TrackStep::One,
            values: Vec::new(),
            parameter: None,
            time_curve,
        })
    }
}
