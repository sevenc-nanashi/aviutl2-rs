mod rotate;
mod sort;
use aviutl2::{
    AnyResult, AviUtl2Info,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterConfigSelectItems, FilterPlugin,
        FilterPluginTable, FilterProcVideo, RgbaPixel,
    },
};

pub use sort::{pixelsort, calc_luminances, over_threshold};
pub use rotate::{rotate_image, Rotate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FilterConfigSelectItems)]
pub enum ThresholdType {
    #[item(name = "しきい値以上")]
    Above,
    #[item(name = "しきい値以下")]
    Below,
}

#[derive(Debug, Clone, PartialEq, Eq, FilterConfigSelectItems)]
pub enum SortDirection {
    #[item(name = "左右")]
    Horizontal,
    #[item(name = "左右（反転）")]
    HorizontalInverted,
    #[item(name = "上下")]
    Vertical,
    #[item(name = "上下（反転）")]
    VerticalInverted,
}

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
pub struct FilterConfig {
    #[track(name = "しきい値", range = 0.0..=1.0, step = 0.001, default = 0.5)]
    pub threshold: f64,
    #[select(
        name = "ソート対象",
        items = ThresholdType,
        default = ThresholdType::Above
    )]
    pub threshold_type: ThresholdType,
    #[select(
        name = "ソート方向",
        items = SortDirection,
        default = SortDirection::Horizontal
    )]
    pub direction: SortDirection,
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
            name: "Rusty Pixel Sort Filter".to_string(),
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
        video: &mut FilterProcVideo,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();
        let (width, height) = (
            video.video_object.width as usize,
            video.video_object.height as usize,
        );
        let mut image: Vec<RgbaPixel> = vec![RgbaPixel::default(); width * height];
        video.get_image_data(&mut image);
        let pixels = sort::pixelsort(&config, image, width, height);
        video.set_image_data(&pixels, video.video_object.width, video.video_object.height);
        Ok(())
    }
}

aviutl2::register_filter_plugin!(PixelSortFilter);
