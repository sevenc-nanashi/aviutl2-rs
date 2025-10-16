use rayon::prelude::*;

#[inline(always)]
pub fn pixelsort(
    config: &crate::FilterConfig,
    image: Vec<aviutl2::filter::RgbaPixel>,
    width: usize,
    height: usize,
) -> Vec<aviutl2::filter::RgbaPixel> {
    let (mut pixels, width, height) = match config.direction {
        crate::SortDirection::Horizontal => (image, width, height),
        crate::SortDirection::HorizontalInverted => (
            crate::rotate::rotate_image(&image, width, height, crate::rotate::Rotate::OneEighty),
            width,
            height,
        ),
        crate::SortDirection::Vertical => (
            crate::rotate::rotate_image(&image, width, height, crate::rotate::Rotate::Ninety),
            height,
            width,
        ),
        crate::SortDirection::VerticalInverted => (
            crate::rotate::rotate_image(&image, width, height, crate::rotate::Rotate::TwoSeventy),
            height,
            width,
        ),
    };

    let luminance = calc_luminances(&pixels);
    let threshold = (config.threshold * 65535.0) as u16;
    let mask = over_threshold(&luminance, threshold);
    let sort_if_brighter = config.threshold_type == crate::ThresholdType::Above;
    cfg_elif::expr_feature!(if ("rayon-sort-rows") {
        pixels.par_chunks_mut(width)
    } else {
        pixels.chunks_mut(width)
    })
    .enumerate()
    .for_each(|(y, row)| {
        let mut start = 0;
        let mut indices = (0..row.len()).collect::<Vec<_>>();
        for x in 0..row.len() {
            if mask[x + y * width] != sort_if_brighter {
                // ソートしないときの処理（区間の確定とソート）
                if start < x {
                    if cfg!(feature = "rayon-sort-inner") {
                        indices[start..x].par_sort_by_key(|&i| luminance[i + y * width]);
                    } else {
                        indices[start..x].sort_by_key(|&i| luminance[i + y * width]);
                    }
                }
                start = x + 1;
            }
        }
        if start < row.len() {
            if cfg!(feature = "rayon-sort-inner") {
                indices[start..row.len()].par_sort_by_key(|&i| luminance[i + y * width]);
            } else {
                indices[start..row.len()].sort_by_key(|&i| luminance[i + y * width]);
            }
        }

        permute_in_place(row, indices);
    });

    let pixels = match config.direction {
        crate::SortDirection::Horizontal => pixels,
        crate::SortDirection::HorizontalInverted => {
            crate::rotate::rotate_image(&pixels, width, height, crate::rotate::Rotate::OneEighty)
        }
        crate::SortDirection::Vertical => {
            crate::rotate::rotate_image(&pixels, width, height, crate::rotate::Rotate::TwoSeventy)
        }
        crate::SortDirection::VerticalInverted => {
            crate::rotate::rotate_image(&pixels, width, height, crate::rotate::Rotate::Ninety)
        }
    };

    pixels
}

fn permute_in_place<T>(data: &mut [T], mut perm: Vec<usize>) {
    let n = data.len();
    debug_assert_eq!(n, perm.len());

    let p = perm.to_vec();
    for i in 0..n {
        perm[p[i]] = i;
    }

    for i in 0..n {
        let current = i;
        while perm[current] != current {
            let next = perm[current];
            data.swap(current, next);
            perm.swap(current, next);
        }
    }
}

#[allow(unused_macros)]
macro_rules! repeat_32 {
    ($e:expr) => {
        [
            $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e,
            $e, $e, $e, $e, $e, $e, $e, $e, $e, $e,
        ]
    };
}

#[cfg(feature = "simd-luminance")]
fn calc_luminances(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
    pixels
        .chunks(32)
        .flat_map(|p| {
            let mut red = p.iter().map(|px| px.r as u16);
            let mut green = p.iter().map(|px| px.g as u16);
            let mut blue = p.iter().map(|px| px.b as u16);
            let red = wide::u16x32::new(repeat_32!(red.next().unwrap_or(0)));
            let green = wide::u16x32::new(repeat_32!(green.next().unwrap_or(0)));
            let blue = wide::u16x32::new(repeat_32!(blue.next().unwrap_or(0)));
            let luminance = red * wide::u16x32::splat(76)
                + green * wide::u16x32::splat(150)
                + blue * wide::u16x32::splat(29);
            luminance.to_array()
        })
        .collect()
}

#[cfg(feature = "rayon-luminance")]
fn calc_luminances(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
    let mut luminances = Vec::with_capacity(pixels.len());
    pixels
        .par_iter()
        .map(|px| {
            let r = px.r as u16;
            let g = px.g as u16;
            let b = px.b as u16;
            r * 76 + g * 150 + b * 29
        })
        .collect_into_vec(&mut luminances);
    luminances
}

#[cfg(not(any(feature = "simd-luminance", feature = "rayon-luminance")))]
fn calc_luminances(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
    pixels
        .iter()
        .map(|px| {
            let r = px.r as u16;
            let g = px.g as u16;
            let b = px.b as u16;
            r * 76 + g * 150 + b * 29
        })
        .collect()
}

#[cfg(feature = "simd-threshold")]
fn over_threshold(luminances: &[u16], threshold: u16) -> Vec<bool> {
    let threshold = wide::u16x32::splat(threshold);
    luminances
        .chunks(32)
        .flat_map(|p| {
            let mut p = p.iter().copied();
            let chunk = wide::u16x32::new(repeat_32!(p.next().unwrap_or(0)));
            let mask = chunk.simd_gt(threshold);
            mask.to_array().map(|b| b != 0)
        })
        .collect()
}
#[cfg(feature = "rayon-threshold")]
fn over_threshold(luminances: &[u16], threshold: u16) -> Vec<bool> {
    let mut mask = Vec::with_capacity(luminances.len());
    luminances
        .par_iter()
        .map(|&l| l > threshold)
        .collect_into_vec(&mut mask);
    mask
}

#[cfg(not(any(feature = "simd-threshold", feature = "rayon-threshold")))]
fn over_threshold(luminances: &[u16], threshold: u16) -> Vec<bool> {
    luminances.iter().map(|&l| l > threshold).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permute_in_place() {
        let mut data = vec!['a', 'b', 'c', 'd'];
        let perm = vec![2, 0, 1, 3];
        permute_in_place(&mut data, perm);
        assert_eq!(data, vec!['c', 'a', 'b', 'd']);
    }
}
