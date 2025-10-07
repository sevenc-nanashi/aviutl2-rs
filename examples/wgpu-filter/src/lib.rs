use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcVideo,
    },
};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use zerocopy::IntoBytes;

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[track(name = "Red", range = 0.0..=1.0, step = 0.01, default = 1.0)]
    red: f64,
    #[track(name = "Green", range = 0.0..=1.0, step = 0.01, default = 0.0)]
    green: f64,
    #[track(name = "Blue", range = 0.0..=1.0, step = 0.01, default = 0.0)]
    blue: f64,

    #[track(name = "Width", range = 1..=1920, step = 1.0, default = 640)]
    width: u32,
    #[track(name = "Height", range = 1..=1080, step = 1.0, default = 480)]
    height: u32,
}

struct WgpuFilter {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: Arc<wgpu::RenderPipeline>,
    bind_group_layout: Arc<wgpu::BindGroupLayout>,
}

impl FilterPlugin for WgpuFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        // --- Initialize WGPU ---
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|e| anyhow::anyhow!("No suitable GPU adapter found: {e}"))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("WGPU Filter Device"),
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            }))?;

        // --- Create uniform layout ---
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // --- Create pipeline ---
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            pipeline: Arc::new(pipeline),
            bind_group_layout: Arc::new(bind_group_layout),
        })
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "wgsl Filter".to_string(),
            label: None,
            information: format!(
                "Example render filter plugin, using wgsl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/wgsl-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            filter_type: aviutl2::filter::FilterType::Video,
            as_object: true,
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_video(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        video: &FilterProcVideo,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();
        let color_data = [
            config.red as f32,
            config.green as f32,
            config.blue as f32,
            1.0f32,
        ];
        let uniform_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Color Uniform Buffer"),
                contents: color_data.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };

        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Triangle Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        // --- Copy result to buffer ---
        let buffer_size = (config.width * config.height * 4) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(config.width * 4),
                    rows_per_image: None,
                },
            },
            texture_desc.size,
        );

        self.queue.submit(Some(encoder.finish()));

        // --- Read back result ---
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        self.device.poll(wgpu::PollType::wait_indefinitely())?;
        rx.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range().to_vec();
        output_buffer.unmap();
        video.set_image_data(&data, config.width, config.height);
        Ok(())
    }
}

aviutl2::register_filter_plugin!(WgpuFilter);
