use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigDataHandle, FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin,
        FilterPluginTable, FilterProcVideo, ImageResource,
    },
};
use rand::RngExt;

#[derive(aviutl2::filter::FilterConfigSelectItems, Debug, Clone, Copy)]
enum Shape {
    #[item(name = "Rectangle")]
    Rectangle,
    #[item(name = "Ellipse")]
    Ellipse,
    #[item(name = "Triangle")]
    Triangle,
}

#[aviutl2::filter::filter_config_items]
#[derive(Debug, Clone)]
struct FilterConfig {
    #[track(name = "Width", range = 1..=4096, step = 1.0, default = 640, group = "size")]
    width: u32,
    #[track(name = "Height", range = 1..=4096, step = 1.0, default = 640, group = "size")]
    height: u32,

    #[select(name = "Shape", default = Shape::Rectangle, items = Shape)]
    shape: Shape,

    #[data]
    color: FilterConfigDataHandle<Color>,

    #[data]
    triangle_tip_offset: FilterConfigDataHandle<[f32; 2]>,
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
    type Userdata = ();

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
                "Example render filter plugin, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/random-color-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            flag: aviutl2::bitflag!(aviutl2::filter::FilterPluginFlag {
                video: true,
                input: true,
            }),
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_video(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        video: &mut FilterProcVideo<Self::Userdata>,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();
        let width = config.width;
        let height = config.height;
        let tip_offset = *config.triangle_tip_offset.read();
        let triangle_tip = [
            tip_offset[0].clamp(-(width as f32) * 0.5, width as f32 * 0.5),
            (-(height as f32) * 0.5 + tip_offset[1])
                .clamp(-(height as f32) * 0.5, height as f32 * 0.5),
            0.0,
        ];
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

        let resource = ImageResource::Resource("random_color".to_string());
        let blank_image = vec![0u8; width as usize * height as usize * 4];
        video.create_image_resource(&resource, &blank_image, width, height)?;
        match config.shape {
            Shape::Rectangle => {
                video.clear_image_resource(&resource, (color.r, color.g, color.b, 255).into())?;
            }
            Shape::Triangle => {
                if let Some(mut anchor) = video.anchor() {
                    anchor.set_anchor_move_point(0, triangle_tip, |items, anchor| {
                        let config: FilterConfig = items.to_struct();
                        let half_width = config.width as f32 * 0.5;
                        let half_height = config.height as f32 * 0.5;
                        *config.triangle_tip_offset.write() = [
                            anchor.x.clamp(-half_width, half_width),
                            anchor.y.clamp(-half_height, half_height) + half_height,
                        ];
                        Ok(())
                    });
                    for x in [-(width as f32) * 0.5, width as f32 * 0.5] {
                        anchor.set_anchor_line(
                            triangle_tip,
                            [x, height as f32 * 0.5, 0.0],
                            (255, 255, 255, 255).into(),
                        );
                    }
                }
                video.draw_poly_to_resource(
                    &resource,
                    &aviutl2::filter::VertexList::TriangleColor(vec![[
                        aviutl2::filter::VertexColor {
                            x: triangle_tip[0],
                            y: triangle_tip[1],
                            z: 0.0,
                            r: color.r as f32 / 255.0,
                            g: color.g as f32 / 255.0,
                            b: color.b as f32 / 255.0,
                            a: 1.0,
                        },
                        aviutl2::filter::VertexColor {
                            x: (width as f32) * 0.5,
                            y: (height as f32) * 0.5,
                            z: 0.0,
                            r: color.r as f32 / 255.0,
                            g: color.g as f32 / 255.0,
                            b: color.b as f32 / 255.0,
                            a: 1.0,
                        },
                        aviutl2::filter::VertexColor {
                            x: (width as f32) * -0.5,
                            y: (height as f32) * 0.5,
                            z: 0.0,
                            r: color.r as f32 / 255.0,
                            g: color.g as f32 / 255.0,
                            b: color.b as f32 / 255.0,
                            a: 1.0,
                        },
                    ]]),
                    Some(&resource),
                )?;
            }
            Shape::Ellipse => {
                let mut vertices = Vec::new();
                let segments = 64;
                for i in 0..segments {
                    let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                    let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                    vertices.push([
                        aviutl2::filter::VertexColor {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            r: color.r as f32 / 255.0,
                            g: color.g as f32 / 255.0,
                            b: color.b as f32 / 255.0,
                            a: 1.0,
                        },
                        aviutl2::filter::VertexColor {
                            x: (width as f32) * 0.5 * angle.cos(),
                            y: (height as f32) * 0.5 * angle.sin(),
                            z: 0.0,
                            r: color.r as f32 / 255.0,
                            g: color.g as f32 / 255.0,
                            b: color.b as f32 / 255.0,
                            a: 1.0,
                        },
                        aviutl2::filter::VertexColor {
                            x: (width as f32) * 0.5 * angle2.cos(),
                            y: (height as f32) * 0.5 * angle2.sin(),
                            z: 0.0,
                            r: color.r as f32 / 255.0,
                            g: color.g as f32 / 255.0,
                            b: color.b as f32 / 255.0,
                            a: 1.0,
                        },
                    ]);
                }
                video.draw_poly_to_resource(
                    &resource,
                    &aviutl2::filter::VertexList::TriangleColor(vertices),
                    Some(&resource),
                )?;
            }
        }
        video.copy_image_resource(&resource, &ImageResource::Object)?;

        Ok(())
    }
}

aviutl2::register_filter_plugin!(RandomColorFilter);
