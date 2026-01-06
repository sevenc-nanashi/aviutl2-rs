/// `Table::parse_value`から呼び出される変換トレイト。
///
/// 任意の型に実装することで、`Table`から直接その型として値を取得できます。
pub trait FromTableValue: Sized {
    type Err;
    fn from_table_value(value: &str) -> Result<Self, Self::Err>;
}

/// バイナリ。
/// フィルタ効果のdata、汎用プラグインのデータなどで使われています。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryItem(Vec<u8>);

impl std::ops::Deref for BinaryItem {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<Vec<u8>> for BinaryItem {
    fn from(value: Vec<u8>) -> Self {
        BinaryItem(value)
    }
}
impl From<&[u8]> for BinaryItem {
    fn from(value: &[u8]) -> Self {
        BinaryItem(value.to_vec())
    }
}
impl<const N: usize> From<&[u8; N]> for BinaryItem {
    fn from(value: &[u8; N]) -> Self {
        BinaryItem(value.to_vec())
    }
}
impl std::fmt::Display for BinaryItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
impl std::str::FromStr for BinaryItem {
    type Err = BinaryParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.len().is_multiple_of(2) {
            return Err(BinaryParseError::InvalidLength);
        }
        let mut result = Vec::with_capacity(s.len() / 2);
        for i in (0..s.len()).step_by(2) {
            let byte = u8::from_str_radix(&s[i..i + 2], 16)?;
            result.push(byte);
        }
        Ok(result.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BinaryParseError {
    #[error("invalid length")]
    InvalidLength,
    #[error("invalid hex value")]
    InvalidHex(#[from] std::num::ParseIntError),
}

impl FromTableValue for BinaryItem {
    type Err = BinaryParseError;
    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        if !value.len().is_multiple_of(2) {
            return Err(BinaryParseError::InvalidLength);
        }
        let mut result = Vec::with_capacity(value.len() / 2);
        for i in (0..value.len()).step_by(2) {
            let byte = u8::from_str_radix(&value[i..i + 2], 16)?;
            result.push(byte);
        }
        Ok(result.into())
    }
}

/// 色項目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorItem {
    Transparent,
    Color(u8, u8, u8),
}

impl std::fmt::Display for ColorItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorItem::Transparent => write!(f, ""),
            ColorItem::Color(r, g, b) => write!(f, "{:02x}{:02x}{:02x}", r, g, b),
        }
    }
}

/// 色のパースエラー。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ColorParseError {
    #[error("invalid length")]
    InvalidLength,
    #[error("invalid hex value")]
    InvalidHex(#[from] std::num::ParseIntError),
}

impl std::str::FromStr for ColorItem {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(ColorItem::Transparent);
        }
        if s.len() != 6 {
            return Err(ColorParseError::InvalidLength);
        }
        let r = u8::from_str_radix(&s[0..2], 16)?;
        let g = u8::from_str_radix(&s[2..4], 16)?;
        let b = u8::from_str_radix(&s[4..6], 16)?;
        Ok(ColorItem::Color(r, g, b))
    }
}

impl FromTableValue for ColorItem {
    type Err = ColorParseError;

    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        value.parse()
    }
}

impl FromTableValue for std::path::PathBuf {
    type Err = std::convert::Infallible;

    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        Ok(std::path::PathBuf::from(value))
    }
}

impl FromTableValue for String {
    type Err = std::convert::Infallible;

    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        let mut str = String::with_capacity(value.len());
        let mut iter = value.chars();
        while let Some(c) = iter.next() {
            match c {
                '\\' => match iter.next() {
                    Some('n') => str.push('\n'),
                    Some('\\') => str.push('\\'),
                    Some(other) => {
                        str.push('\\');
                        str.push(other);
                    }
                    None => str.push('\\'),
                },
                _ => str.push(c),
            }
        }
        Ok(str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BoolParseError {
    #[error("invalid boolean value")]
    InvalidValue,
}

impl FromTableValue for bool {
    type Err = BoolParseError;

    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "1" => true,
            "0" => false,
            _ => return Err(BoolParseError::InvalidValue),
        })
    }
}

#[duplicate::duplicate_item(
    Int;
    [i8];
    [i16];
    [i32];
    [i64];
    [i128];
    [isize];
    [u8];
    [u16];
    [u32];
    [u64];
    [u128];
    [usize];
)]
const _: () = {
    use std::str::FromStr;

    impl FromTableValue for Int {
        type Err = std::num::ParseIntError;

        fn from_table_value(value: &str) -> Result<Self, Self::Err> {
            value.parse()
        }
    }
    impl FromTableValue for Vec<Int> {
        type Err = std::num::ParseIntError;

        fn from_table_value(value: &str) -> Result<Self, Self::Err> {
            value
                .split(',')
                .map(Int::from_str)
                .collect::<Result<Vec<_>, _>>()
        }
    }
};

impl FromTableValue for crate::TrackItem {
    type Err = crate::TrackItemParseError;

    fn from_table_value(value: &str) -> Result<Self, Self::Err> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Table;

    #[test]
    fn test_parse_table_value() {
        let input = include_str!("../test_assets/everything.aup2");
        let table: Table = input.parse().unwrap();

        let project_table = table.get_table("project").unwrap();
        assert_eq!(
            project_table
                .parse_value::<std::path::PathBuf>("file")
                .unwrap()
                .unwrap(),
            std::path::PathBuf::from("Z:\\test.aup2")
        );

        let obj0 = table.get_table("0").unwrap();
        assert_eq!(
            obj0.parse_value::<Vec<u8>>("frame").unwrap().unwrap(),
            &[0, 80]
        );
        let effect = obj0.get_table("0").unwrap();
        assert_eq!(
            effect.parse_value::<ColorItem>("主色").unwrap().unwrap(),
            ColorItem::Color(255, 255, 255)
        );
        assert_eq!(
            effect.parse_value::<String>("テキスト").unwrap().unwrap(),
            "Hello\\\nWorld"
        );
    }

    #[test]
    fn test_parse_binaries() {
        let input = include_str!("../test_assets/binary.aup2");
        let table: Table = input.parse().unwrap();

        let obj0 = table.get_table("0").unwrap();
        let effect = obj0.get_table("0").unwrap();
        assert_eq!(
            effect.parse_value::<BinaryItem>("color").unwrap().unwrap(),
            BinaryItem::from(&[0x01, 0x93, 0x4d, 0x5e])
        );

        let plugin = table.get_table("plugin").unwrap();
        let plugin0 = plugin.get_table("0").unwrap();

        assert_eq!(
            plugin0
                .parse_value::<BinaryItem>("--aviutl2-rs:serde-zstd-v1:chunk:alias_entries:0")
                .unwrap()
                .unwrap(),
            BinaryItem::from(&[0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x58, 0x09, 0x00, 0x00, 0x90])
        );
    }
}
