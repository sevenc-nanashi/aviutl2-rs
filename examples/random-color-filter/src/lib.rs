use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigDataHandle, FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin,
        FilterPluginTable, FilterProcVideo,
    },
};
use rand::RngExt;

#[aviutl2::filter::filter_config_items]
#[derive(Debug, Clone)]
struct FilterConfig {
    #[track(name = "Width", range = 1..=4096, step = 1.0, default = 640, group = "size")]
    width: u32,
    #[track(name = "Height", range = 1..=4096, step = 1.0, default = 640, group = "size")]
    height: u32,

    #[data]
    color: FilterConfigDataHandle<Color>,
}

#[derive(Debug, Clone, Copy, Default)]
struct Color {
    initialized: bool,
    r: u8,
    g: u8,
    b: u8,
}

#[aviutl2::plugin(FilterPlugin)]
struct RandomColorFilter {}

impl FilterPlugin for RandomColorFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        aviutl2::tracing_subscriber::fmt()
            .with_max_level(if cfg!(debug_assertions) {
                tracing::Level::DEBUG
            } else {
                tracing::Level::INFO
            })
            .event_format(aviutl2::logger::AviUtl2Formatter)
            .with_writer(aviutl2::logger::AviUtl2LogWriter)
            .init();
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
            flags: aviutl2::bitflag!(aviutl2::filter::FilterPluginFlags {
                video: true,
                input: true,
            }),
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
        let color_handle = config.color.read();

        let color = if !color_handle.initialized {
            let mut rng = rand::rng();
            let mut color = *color_handle;
            color.r = rng.random_range(0..=255);
            color.g = rng.random_range(0..=255);
            color.b = rng.random_range(0..=255);
            color.initialized = true;
            drop(color_handle);
            *config.color.write() = color;
            color
        } else {
            *color_handle
        };

        let cache_key = format!(
            "random_color_{}_{}_{}_{}_{}",
            color.r, color.g, color.b, width, height
        );
        let cache =
            aviutl2::cache::get_image_cache(&aviutl2::cache::GLOBAL_CACHE_HANDLE, &cache_key)?;
        if let Some(cache) = cache {
            tracing::debug!("Cache hit for key: {}", cache_key);
            video.set_image_data(
                cache.as_u8_slice(),
                cache.width() as u32,
                cache.height() as u32,
            );
            return Ok(());
        }

        let image_data = (0..(width * height))
            .map(|_| aviutl2::filter::RgbaPixel {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 255,
            })
            .collect::<Vec<_>>();
        video.set_image_data(&image_data, width, height);
        let mut cache = aviutl2::cache::create_image_cache(
            &aviutl2::cache::GLOBAL_CACHE_HANDLE,
            &cache_key,
            width as _,
            height as _,
        )?;
        cache.as_slice_mut().copy_from_slice(&image_data);
        tracing::debug!("Cache created for key: {}", cache_key);

        Ok(())
    }
}

aviutl2::register_filter_plugin!(RandomColorFilter);
