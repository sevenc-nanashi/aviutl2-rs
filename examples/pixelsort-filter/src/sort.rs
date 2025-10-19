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

    match config.direction {
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
    }
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

fn calc_luminances_rayon_simd(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
    let mut luminances = Vec::<u16>::with_capacity(pixels.len());
    pixels
        .par_chunks(32)
        .zip(luminances.spare_capacity_mut().par_chunks_mut(32))
        .for_each(|(p, luminances_part)| {
            let mut red = [0u16; 32];
            let mut green = [0u16; 32];
            let mut blue = [0u16; 32];
            for (i, px) in p.iter().enumerate() {
                red[i] = px.r as u16;
                green[i] = px.g as u16;
                blue[i] = px.b as u16;
            }
            let red = wide::u16x32::new(red);
            let green = wide::u16x32::new(green);
            let blue = wide::u16x32::new(blue);
            let luminance = red * wide::u16x32::splat(76)
                + green * wide::u16x32::splat(150)
                + blue * wide::u16x32::splat(29);
            for (i, &l) in luminance.to_array().iter().take(p.len()).enumerate() {
                luminances_part[i].write(l);
            }
        });
    unsafe {
        luminances.set_len(pixels.len());
    }
    luminances
}

fn calc_luminances_simd(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
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
fn calc_luminances_rayon(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
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
fn calc_luminances_default(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
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

#[inline(always)]
pub fn calc_luminances(pixels: &[aviutl2::filter::RgbaPixel]) -> Vec<u16> {
    if cfg!(all(feature = "rayon-luminance", feature = "simd-luminance")) {
        calc_luminances_rayon_simd(pixels)
    } else if cfg!(feature = "simd-luminance") {
        calc_luminances_simd(pixels)
    } else if cfg!(feature = "rayon-luminance") {
        calc_luminances_rayon(pixels)
    } else {
        calc_luminances_default(pixels)
    }
}

fn over_threshold_simd_rayon(luminances: &[u16], threshold: u16) -> Vec<bool> {
    let threshold = wide::u16x32::splat(threshold);
    let mut mask = Vec::<bool>::with_capacity(luminances.len());
    luminances
        .par_chunks(32)
        .zip(mask.spare_capacity_mut().par_chunks_mut(32))
        .for_each(|(p, mask_part)| {
            let p_len = p.len();
            let mut p = p.iter().copied();
            let chunk = wide::u16x32::new(repeat_32!(p.next().unwrap_or(0)));
            let mask = chunk.simd_gt(threshold);
            for (i, &b) in mask.to_array().iter().take(p_len).enumerate() {
                mask_part[i].write(b != 0);
            }
        });
    unsafe {
        mask.set_len(luminances.len());
    }
    mask
}

fn over_threshold_simd(luminances: &[u16], threshold: u16) -> Vec<bool> {
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

fn over_threshold_rayon(luminances: &[u16], threshold: u16) -> Vec<bool> {
    let mut mask = Vec::with_capacity(luminances.len());
    luminances
        .par_iter()
        .map(|&l| l > threshold)
        .collect_into_vec(&mut mask);
    mask
}

fn over_threshold_default(luminances: &[u16], threshold: u16) -> Vec<bool> {
    luminances.iter().map(|&l| l > threshold).collect()
}

#[inline(always)]
pub fn over_threshold(luminances: &[u16], threshold: u16) -> Vec<bool> {
    if cfg!(all(feature = "rayon-threshold", feature = "simd-threshold")) {
        over_threshold_simd_rayon(luminances, threshold)
    } else if cfg!(feature = "simd-threshold") {
        over_threshold_simd(luminances, threshold)
    } else if cfg!(feature = "rayon-threshold") {
        over_threshold_rayon(luminances, threshold)
    } else {
        over_threshold_default(luminances, threshold)
    }
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

    #[test]
    fn test_over_threshold() {
        let luminances = vec![100u16, 200, 300, 400, 500];
        let threshold = 300u16;
        let mask = over_threshold(&luminances, threshold);
        assert_eq!(mask, vec![false, false, false, true, true]);
    }
}
