mod rotate;
use aviutl2::{
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterConfigSelectItems, FilterPlugin, FilterPluginTable, FilterProcVideo, RgbaPixel
    }, AnyResult, AviUtl2Info
};
use rayon::prelude::*;

#[derive(Debug, Clone, FilterConfigSelectItems)]
enum ThresholdType {
    #[item(name = "しきい値以上")]
    Above,
    #[item(name = "しきい値以下")]
    Below,
}

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[track(name = "しきい値", range = 0.0..=1.0, step = 0.001, default = 0.5)]
    threshold: f64,
    #[select(
        name = "ソート対象",
        items = ThresholdType,
        default = ThresholdType::Above
    )]
    threshold_type: usize,
    #[select(
        name = "ソート方向",
        items = ["左右", "左右（反転）", "上下", "上下（反転）"],
        default = 0
    )]
    direction: usize,
}

struct PixelSortFilter;

impl FilterPlugin for PixelSortFilter {
    fn new(_info: AviUtl2Info) -> AnyResult<Self> {
        env_logger::Builder::new()
            .parse_filters("info")
            .target(aviutl2::utils::debug_logger_target())
            .init();
        Ok(Self)
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "Pixel Sort Filter".to_string(),
            label: None,
            information: format!(
                "Pixel sort filter, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/pixelsort-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            filter_type: aviutl2::filter::FilterType::Video,
            as_object: false,
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_video(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        video: &FilterProcVideo,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();
        let (width, height) = (
            video.video_object.width as usize,
            video.video_object.height as usize,
        );
        let image: Vec<RgbaPixel> = video.get_image_data();
        let (mut pixels, width, height) = if config.direction == 1 {
            (
                rotate::rotate_image(&image, width, height, rotate::Rotate::OneEighty),
                width,
                height,
            )
        } else if config.direction == 2 {
            (
                rotate::rotate_image(&image, width, height, rotate::Rotate::Ninety),
                height,
                width,
            )
        } else if config.direction == 3 {
            (
                rotate::rotate_image(&image, width, height, rotate::Rotate::TwoSeventy),
                height,
                width,
            )
        } else {
            (image, width, height)
        };

        let luminance = calc_luminances(&pixels);
        let threshold = (config.threshold * 65535.0) as u16;
        let mask = over_threshold(&luminance, threshold);
        let sort_if_brighter = config.threshold_type == 0;
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

        let pixels = if config.direction == 1 {
            rotate::rotate_image(&pixels, width, height, rotate::Rotate::OneEighty)
        } else if config.direction == 2 {
            rotate::rotate_image(&pixels, width, height, rotate::Rotate::TwoSeventy)
        } else if config.direction == 3 {
            rotate::rotate_image(&pixels, width, height, rotate::Rotate::Ninety)
        } else {
            pixels
        };
        video.set_image_data(&pixels, video.video_object.width, video.video_object.height);
        Ok(())
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

#[cfg(feature = "simd-luminance")]
fn calc_luminances(pixels: &[RgbaPixel]) -> Vec<u16> {
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

#[cfg(not(feature = "simd-luminance"))]
fn calc_luminances(pixels: &[RgbaPixel]) -> Vec<u16> {
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

#[cfg(not(feature = "simd-threshold"))]
fn over_threshold(luminances: &[u16], threshold: u16) -> Vec<bool> {
    luminances.iter().map(|&l| l > threshold).collect()
}

aviutl2::register_filter_plugin!(PixelSortFilter);

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
