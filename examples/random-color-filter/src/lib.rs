use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcVideo,
    },
};

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[color(name = "Color", default = "#48b0d5")]
    color: aviutl2::filter::FilterConfigColorValue,

    #[track(name = "Width", range = 1..=4096, step = 1.0, default = 640)]
    width: u32,
    #[track(name = "Height", range = 1..=4096, step = 1.0, default = 640)]
    height: u32,
}

struct Color {}

#[aviutl2::plugin(FilterPlugin)]
struct RandomColorFilter {}

impl FilterPlugin for RandomColorFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(Self {})
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "Rusty Random Color Filter".to_string(),
            label: None,
            information: format!(
                "Example render filter plugin, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/wgsl-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            filter_type: aviutl2::filter::FilterType::Video,
            as_object: true,
            support_filter_object: false,
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_video(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        video: &mut FilterProcVideo,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();
        let width = config.width;
        let height = config.height;

        todo!()
    }
}

aviutl2::register_filter_plugin!(RandomColorFilter);
