mod transpose;
use aviutl2::{
    AnyResult, AviUtl2Info,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcVideo, RgbaPixel,
    },
};
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[track(name = "しきい値", range = 0.0..=1.0, step = 0.001, default = 0.5)]
    threshold: f64,
    #[select(
        name = "基準",
        items = ["しきい値以上", "しきい値以下"],
        default = 0
    )]
    threshold_type: usize,
    #[select(
        name = "ソート方向",
        items = ["右", "下", "左", "上"],
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
        let mut pixels = image.to_vec();
        let (width, height) = if config.direction == 1 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::Ninety);
            (height, width)
        } else if config.direction == 2 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::OneEighty);
            (width, height)
        } else if config.direction == 3 {
            transpose::transpose_image(
                &mut pixels,
                width,
                height,
                transpose::Transpose::TwoSeventy,
            );
            (height, width)
        } else {
            (width, height)
        };

        let threshold = (config.threshold * 65535.0) as u16;
        let luminance = calc_luminances(&pixels);
        let mask = if config.threshold_type == 0 {
            over_threshold(&luminance, threshold)
        } else {
            under_threshold(&luminance, threshold)
        };
        pixels
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(height, row)| {
                let mut start = 0;
                for x in 0..row.len() {
                    if !mask[x + height * width] {
                        continue;
                    }
                    if start < x {
                        let mut indices = (start..x).collect::<Vec<_>>();
                        indices.sort_by_key(|&i| luminance[i + height * width]);
                        let original = row[start..x].to_vec();
                        for (i, &idx) in indices.iter().enumerate() {
                            row[start + i] = original[idx - start];
                        }
                    }
                    start = x + 1;
                }
                if start < row.len() {
                    let mut indices = (start..row.len()).collect::<Vec<_>>();
                    indices.sort_by_key(|&i| luminance[i + height * width]);
                    let original = row[start..].to_vec();
                    for (i, &idx) in indices.iter().enumerate() {
                        row[start + i] = original[idx - start];
                    }
                }
            });

        let (pixels, width, height) = if config.direction == 1 {
            transpose::transpose_image(
                &mut pixels,
                width,
                height,
                transpose::Transpose::TwoSeventy,
            );
            (pixels, height, width)
        } else if config.direction == 2 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::OneEighty);
            (pixels, width, height)
        } else if config.direction == 3 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::Ninety);
            (pixels, height, width)
        } else {
            (pixels, width, height)
        };

        video.set_image_data(&pixels, width as u32, height as u32);
        Ok(())
    }
}

macro_rules! repeat_32 {
    ($e:expr) => {
        [
            $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e, $e,
            $e, $e, $e, $e, $e, $e, $e, $e, $e, $e,
        ]
    };
}

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
            let luminance: wide::u16x32 = luminance;
            luminance.to_array()
        })
        .collect()
}

#[duplicate::duplicate_item(
    method_name       compare;
    [over_threshold]  [simd_gt];
    [under_threshold] [simd_lt];
)]
fn method_name(luminances: &[u16], threshold: u16) -> Vec<bool> {
    let threshold = wide::u16x32::splat(threshold);
    luminances
        .chunks(32)
        .flat_map(|p| {
            let mut p = p.iter().copied();
            let chunk = wide::u16x32::new(repeat_32!(p.next().unwrap_or(0)));
            let mask = chunk.compare(threshold);
            mask.to_array().map(|b| b != 0)
        })
        .collect()
}

aviutl2::register_filter_plugin!(PixelSortFilter);
