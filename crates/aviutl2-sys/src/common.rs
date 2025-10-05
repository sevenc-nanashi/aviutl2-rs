macro_rules! fourcc {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24))
    };
}

/// YUY2（YUV 4:2:2）フォーマット
pub const BI_YUY2: u32 = fourcc!('Y', 'U', 'Y', '2');
/// PA64（DXGI_FORMAT_R16G16B16A16_UNORM、乗算済みα）フォーマット
pub const BI_PA64: u32 = fourcc!('P', 'A', '6', '4');
/// YC48（互換対応のフォーマット）フォーマット
pub const BI_YC48: u32 = fourcc!('Y', 'C', '4', '8');
/// HF64（DXGI_FORMAT_R16G16B16A16_FLOAT、乗算済みα）フォーマット（内部フォーマット）
pub const BI_HF64: u32 = fourcc!('H', 'F', '6', '4');

pub type LPCWSTR = *const u16;
