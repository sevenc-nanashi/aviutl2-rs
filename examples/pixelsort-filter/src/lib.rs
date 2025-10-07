mod transpose;
use aviutl2::{
    AnyResult, AviUtl2Info,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcVideo, RgbaPixel,
    },
};

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[track(name = "しきい値", range = 0.0..=1.0, step = 0.01, default = 0.5)]
    threshold: f64,
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
        Ok(Self)
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "Pixel Sort Filter".to_string(),
            label: None,
            information: format!(
                "Pixel sort filter, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/equalizer-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            filter_type: aviutl2::filter::FilterType::Video,
            wants_initial_input: false,
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

        use rayon::prelude::*;
        let threshold = (config.threshold * 255.0) as u8;
        pixels.par_chunks_mut(width).for_each(|row| {
            let mut start = 0;
            for x in 0..row.len() {
                let pixel = row[x];
                let brightness =
                    ((pixel.r as u16 * 76 + pixel.g as u16 * 150 + pixel.b as u16 * 29) >> 8) as u8;

                if brightness < threshold {
                    if start < x {
                        row[start..x].sort_by_key(|p| {
                            ((p.r as u16 * 76 + p.g as u16 * 150 + p.b as u16 * 29) >> 8) as u8
                        });
                    }
                    start = x + 1;
                }
            }
            if start < row.len() {
                row[start..].sort_by_key(|p| {
                    ((p.r as u16 * 76 + p.g as u16 * 150 + p.b as u16 * 29) >> 8) as u8
                });
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

aviutl2::register_filter_plugin!(PixelSortFilter);
