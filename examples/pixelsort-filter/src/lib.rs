mod transpose;
use aviutl2::{
    AnyResult, AviUtl2Info,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcVideo,
    },
};

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[track(name = "しきい値", range = 0.0..=1.0, step = 0.01, default = 0.5)]
    threshold: f64,
    #[select(
        name = "ソート方向",
        items = ["0°", "90°", "180°", "270°"],
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
        let (width, height) = (video.scene.width as usize, video.scene.height as usize);
        let image = video.get_image_data();
        let mut pixels = image.to_vec();
        if config.direction == 1 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::Ninety);
        } else if config.direction == 2 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::OneEighty);
        } else if config.direction == 3 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::TwoSeventy);
        }

        for y in 0..height {
            let mut start = 0;
            for x in 0..width {
                let i = y * width + x;
                let pixel = pixels[i];
                let brightness =
                    (pixel.r as f64 * 0.299 + pixel.g as f64 * 0.587 + pixel.b as f64 * 0.114)
                        / 255.0;

                if brightness < config.threshold {
                    if start < x {
                        pixels[start..i].sort_by_key(|p| {
                            (p.r as f64 * 0.299 + p.g as f64 * 0.587 + p.b as f64 * 0.114) as u8
                        });
                    }
                    start = x + 1;
                }
            }
            if start < width {
                pixels[start..(y * width + width)].sort_by_key(|p| {
                    (p.r as f64 * 0.299 + p.g as f64 * 0.587 + p.b as f64 * 0.114) as u8
                });
            }
        }
        if config.direction == 1 {
            transpose::transpose_image(&mut pixels, height, width, transpose::Transpose::TwoSeventy);
        } else if config.direction == 2 {
            transpose::transpose_image(&mut pixels, width, height, transpose::Transpose::OneEighty);
        } else if config.direction == 3 {
            transpose::transpose_image(&mut pixels, height, width, transpose::Transpose::Ninety);
        }

        video.set_image_data(&pixels, width as u32, height as u32);
        Ok(())
    }
}

aviutl2::register_filter_plugin!(PixelSortFilter);
