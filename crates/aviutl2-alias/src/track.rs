/// トラックバーのフラグ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TrackFlags {
    /// 「加速」相当。
    ease_in: bool,
    /// 「減速」相当。
    ease_out: bool,
    /// 「中間点無視」相当。
    twopoints: bool,
}

/// トラックバーの移動単位。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

/// トラックバーの移動単位のパースエラー。
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum TrackStepParseError {
    #[error("invalid track step precision")]
    InvalidPrecision,
    #[error("failed to parse track step value")]
    ParseError(#[from] std::num::ParseFloatError),
}

impl TrackStep {
    /// パースして精度と値を取得します。
    pub fn parse_and_get(value: &str) -> Result<(TrackStep, f64), TrackStepParseError> {
        let maybe_negative = value.starts_with('-');
        let abs_value_str = if maybe_negative { &value[1..] } else { value };
        let v: f64 = abs_value_str.parse()?;
        let dot_index = abs_value_str.find('.');
        let step = match dot_index {
            None => TrackStep::One,
            Some(idx) => match abs_value_str.len() - idx - 1 {
                1 => TrackStep::PointOne,
                2 => TrackStep::PointZeroOne,
                3 => TrackStep::PointZeroZeroOne,
                _ => return Err(TrackStepParseError::InvalidPrecision),
            },
        };
        let final_value = if maybe_negative { -v } else { v };
        Ok((step, final_value))
    }

    /// 値を特定の精度に丸めて文字列化します。
    pub fn round_to_string(&self, value: f64) -> String {
        match self {
            TrackStep::One => format!("{}", value.round() as i64),
            TrackStep::PointOne => format!("{:.1}", value),
            TrackStep::PointZeroOne => format!("{:.2}", value),
            TrackStep::PointZeroZeroOne => format!("{:.3}", value),
        }
    }
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
        write!(f, "TrackStep({})", f64::from(*self))
    }
}

/// 時間制御のカーブ。
#[derive(Debug, Clone, PartialEq)]
pub struct TimeCurve {
    /// カーブの制御点。
    pub control_points: Vec<TimeCurvePoint>,
}

impl std::fmt::Display for TimeCurve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.control_points.is_empty() {
            return write!(f, "");
        }
        if self.control_points.len() == 2
            && self.control_points[0].position == 0.0
            && self.control_points[0].value == 0.0
            && self.control_points[1].position == 1.0
            && self.control_points[1].value == 1.0
        {
            return write!(
                f,
                "{},{},{},{}",
                self.control_points[0].right_handle.0,
                self.control_points[0].right_handle.1,
                self.control_points[1].right_handle.0,
                self.control_points[1].right_handle.1
            );
        }
        let parts: Vec<String> = self
            .control_points
            .iter()
            .map(|pt| {
                format!(
                    "{},{},{},{}",
                    pt.position, pt.value, pt.right_handle.0, pt.right_handle.1
                )
            })
            .collect();
        write!(f, "{}", parts.join(","))
    }
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

/// 時間制御カーブのパースエラー。
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum TimeCurveParseError {
    #[error("invalid format")]
    InvalidFormat,

    #[error("invalid number of components ({0})")]
    InvalidNumComponents(usize),

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
            n if n % 4 != 0 => Err(TimeCurveParseError::InvalidNumComponents(n)),
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

/// `.aup2`に保存されるトラックバー項目を表します。
#[derive(Debug, Clone, PartialEq)]
pub enum TrackItem {
    /// スクリプトを伴わない単一値トラック。
    Static(StaticTrackItem),
    /// スクリプトや時間制御を含んだトラックバー項目。
    Animated(AnimatedTrackItem),
}

impl std::fmt::Display for TrackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackItem::Static(item) => write!(f, "{}", item),
            TrackItem::Animated(item) => write!(f, "{}", item),
        }
    }
}

impl std::str::FromStr for TrackItem {
    type Err = TrackItemParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(',') {
            let animated: AnimatedTrackItem = s.parse()?;
            Ok(TrackItem::Animated(animated))
        } else {
            let static_item: StaticTrackItem = s.parse()?;
            Ok(TrackItem::Static(static_item))
        }
    }
}

/// スクリプトを伴わない単一値トラックを表します。
#[derive(Debug, Clone, PartialEq)]
pub struct StaticTrackItem {
    /// 移動単位。
    pub step: TrackStep,

    /// 値。
    pub value: f64,
}

impl std::fmt::Display for StaticTrackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value_str = self.step.round_to_string(self.value);
        write!(f, "{}", value_str)
    }
}

impl std::str::FromStr for StaticTrackItem {
    type Err = TrackStepParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (step, value) = TrackStep::parse_and_get(s)?;
        Ok(StaticTrackItem { step, value })
    }
}

/// スクリプトや時間制御を含んだトラックバー項目です。
#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedTrackItem {
    /// 移動単位。
    pub step: TrackStep,
    /// 中間点ごとの値。
    pub values: Vec<f64>,

    /// フラグ。
    pub flags: TrackFlags,
    /// トラックバースクリプトの名前。
    pub script_name: String,
    /// 設定値。
    pub parameter: Option<f64>,
    /// 時間制御のカーブ。
    pub time_curve: Option<TimeCurve>,
}

/// トラックバー項目のパースエラー。
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum TrackItemParseError {
    #[error("invalid segments count")]
    InvalidNumSegments(usize),

    #[error("invalid elements count")]
    InvalidNumElements(usize),

    #[error("failed to parse element")]
    ElementParseError(#[from] std::num::ParseFloatError),

    #[error("failed to parse curve")]
    TimeCurveParseError(#[from] TimeCurveParseError),

    #[error("invalid flag value")]
    InvalidFlagValue,

    #[error("inconsistent step")]
    InconsistentStep,

    #[error("invalid step value")]
    InvalidStepValue(#[from] TrackStepParseError),
}

impl std::fmt::Display for AnimatedTrackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut elements = Vec::new();
        for value in &self.values {
            elements.push(self.step.round_to_string(*value));
        }
        elements.push(self.script_name.clone());
        let flags_value = (if self.flags.ease_in { 0b0001 } else { 0 })
            | (if self.flags.ease_out { 0b0010 } else { 0 })
            | (if self.flags.twopoints { 0b0100 } else { 0 });
        elements.push(flags_value.to_string());
        let mut result = elements.join(",");
        if let Some(param) = self.parameter {
            result.push('|');
            result.push_str(&param.to_string());
        }
        if let Some(curve) = &self.time_curve {
            result.push('|');
            result.push_str(&curve.to_string());
        }
        write!(f, "{}", result)
    }
}

impl std::str::FromStr for AnimatedTrackItem {
    type Err = TrackItemParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = s.split('|').collect::<Vec<&str>>();
        let items = segments[0].split(",").collect::<Vec<&str>>();
        if items.len() < 4 {
            return Err(TrackItemParseError::InvalidNumElements(items.len()));
        }
        let flags_value: u8 = items[items.len() - 1]
            .parse()
            .map_err(|_| TrackItemParseError::InvalidFlagValue)?;
        let flags = TrackFlags {
            ease_in: (flags_value & 0b0001) != 0,
            ease_out: (flags_value & 0b0010) != 0,
            twopoints: (flags_value & 0b0100) != 0,
        };
        let (parameter, time_curve) = match segments.len() {
            1 => (None, None),
            2 if segments[1].contains(',') => {
                let time_curve: TimeCurve = segments[1]
                    .parse()
                    .map_err(TrackItemParseError::TimeCurveParseError)?;
                (None, Some(time_curve))
            }
            2 => {
                let parameter: f64 = segments[1]
                    .parse()
                    .map_err(TrackItemParseError::ElementParseError)?;
                (Some(parameter), None)
            }
            3 => {
                let parameter: f64 = segments[1]
                    .parse()
                    .map_err(TrackItemParseError::ElementParseError)?;
                let time_curve: TimeCurve = segments[2]
                    .parse()
                    .map_err(TrackItemParseError::TimeCurveParseError)?;
                (Some(parameter), Some(time_curve))
            }
            n => {
                return Err(TrackItemParseError::InvalidNumSegments(n));
            }
        };

        let script_name = items[items.len() - 2].to_string();
        let (step, _) = TrackStep::parse_and_get(items[0])?;
        let mut values = Vec::new();
        for item in &items[0..items.len() - 2] {
            let (item_step, value) = TrackStep::parse_and_get(item)?;
            if item_step != step {
                return Err(TrackItemParseError::InconsistentStep);
            }
            values.push(value);
        }
        Ok(AnimatedTrackItem {
            step,
            values,
            flags,
            script_name,
            parameter,
            time_curve,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_time_curve_from_str() {
        let tc: TimeCurve = "0.25,0.25,0.25,0.25".parse().unwrap();
        assert_eq!(
            tc.control_points,
            vec![
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
            ]
        );
        assert_eq!(tc.to_string(), "0.25,0.25,0.25,0.25");
    }

    #[test]
    fn test_time_curve_from_str_multiple_points() {
        let tc: TimeCurve = "0,0,0.25,0,0.5,0.5,0.25,0,1,1,0.25,0".parse().unwrap();

        assert_eq!(
            tc.control_points,
            vec![
                TimeCurvePoint {
                    position: 0.0,
                    value: 0.0,
                    right_handle: (0.25, 0.0),
                },
                TimeCurvePoint {
                    position: 0.5,
                    value: 0.5,
                    right_handle: (0.25, 0.0),
                },
                TimeCurvePoint {
                    position: 1.0,
                    value: 1.0,
                    right_handle: (0.25, 0.0),
                },
            ]
        );

        assert_eq!(tc.to_string(), "0,0,0.25,0,0.5,0.5,0.25,0,1,1,0.25,0");
    }

    #[test]
    fn test_track_item_parse() {
        let item_str = "0.1,0.2,MyScript,3|1.5|0.25,0.25,0.25,0.25";
        let animated_item: AnimatedTrackItem = item_str.parse().unwrap();
        assert_eq!(
            animated_item,
            AnimatedTrackItem {
                step: TrackStep::PointOne,
                values: vec![0.1, 0.2],
                flags: TrackFlags {
                    ease_in: true,
                    ease_out: true,
                    twopoints: false,
                },
                script_name: "MyScript".to_string(),
                parameter: Some(1.5),
                time_curve: Some(TimeCurve {
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
                }),
            }
        );
        assert_eq!(animated_item.to_string(), item_str);
    }
    #[test]
    fn test_track_item_parse_segments() {
        let item_str = "0.1,0.2,MyScript,3|1.5|0.25,0.25,0.25,0.25";
        let animated_item: AnimatedTrackItem = item_str.parse().unwrap();
        assert_eq!(animated_item.parameter, Some(1.5));
        assert!(animated_item.time_curve.is_some());

        let item_str_no_curve = "0.1,0.2,MyScript,3|1.5";
        let animated_item_no_curve: AnimatedTrackItem = item_str_no_curve.parse().unwrap();
        assert_eq!(animated_item_no_curve.parameter, Some(1.5));
        assert!(animated_item_no_curve.time_curve.is_none());

        let item_str_no_param = "0.1,0.2,MyScript,3|0.25,0.25,0.25,0.25";
        let animated_item_no_param: AnimatedTrackItem = item_str_no_param.parse().unwrap();
        assert!(animated_item_no_param.parameter.is_none());
        assert!(animated_item_no_param.time_curve.is_some());

        let item_str_only_values = "0.1,0.2,MyScript,3";
        let animated_item_only_values: AnimatedTrackItem = item_str_only_values.parse().unwrap();
        assert!(animated_item_only_values.parameter.is_none());
        assert!(animated_item_only_values.time_curve.is_none());
    }

    #[rstest]
    #[case("1", TrackStep::One, 1.0)]
    #[case("0.1", TrackStep::PointOne, 0.1)]
    #[case("0.01", TrackStep::PointZeroOne, 0.01)]
    #[case("0.001", TrackStep::PointZeroZeroOne, 0.001)]
    #[case("-2.34", TrackStep::PointZeroOne, -2.34)]
    fn test_track_step_parse_and_get(
        #[case] input: &str,
        #[case] expected_step: TrackStep,
        #[case] expected_value: f64,
    ) {
        let (step, value) = TrackStep::parse_and_get(input).unwrap();
        assert_eq!(step, expected_step);
        assert_eq!(value, expected_value);
    }

    #[rstest]
    #[case(TrackStep::One, 2.34, "2")]
    #[case(TrackStep::PointOne, 2.34, "2.3")]
    #[case(TrackStep::PointZeroOne, 2.345, "2.35")]
    #[case(TrackStep::PointZeroZeroOne, 2.3456, "2.346")]
    fn test_track_step_round_to_string(
        #[case] step: TrackStep,
        #[case] value: f64,
        #[case] expected_str: &str,
    ) {
        let result_str = step.round_to_string(value);
        assert_eq!(result_str, expected_str);
    }
}
