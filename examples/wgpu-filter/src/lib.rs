use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcVideo,
    },
};
use wgpu::util::DeviceExt;
use zerocopy::IntoBytes;

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[color(name = "Color", default = "#48b0d5")]
    color: aviutl2::filter::FilterConfigColorValue,

    #[track(name = "Width", range = 1..=4096, step = 1.0, default = 640)]
    width: u32,
    #[track(name = "Height", range = 1..=4096, step = 1.0, default = 640)]
    height: u32,
}

struct WgpuFilter {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
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

        device.on_uncaptured_error(std::sync::Arc::new(|error| {
            log::error!("WGPU Error: {:?}", error);
        }));

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
            device,
            queue,
            pipeline,
            bind_group_layout,
        })
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "Rusty wgpu Filter".to_string(),
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
        video: &mut FilterProcVideo,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();
        let width = config.width;
        let height = config.height;
        let (red, green, blue) = config.color.to_rgb();
        let color_data = [
            red as f32 / 255.0f32,
            green as f32 / 255.0f32,
            blue as f32 / 255.0f32,
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
                width,
                height,
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
        let padded_width = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer_size = (padded_width * height) as wgpu::BufferAddress;
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
                    bytes_per_row: Some(padded_width),
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

        let data = buffer_slice.get_mapped_range();
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        if padded_width == width * 4 {
            pixels.copy_from_slice(&data);
        } else {
            for y in 0..height as usize {
                let src_offset = y * padded_width as usize;
                let dst_offset = y * width as usize * 4;
                pixels[dst_offset..(dst_offset + (width as usize * 4))]
                    .copy_from_slice(&data[src_offset..(src_offset + (width as usize * 4))]);
            }
        }
        drop(data);
        output_buffer.unmap();

        video.set_image_data(&pixels, width, height);
        Ok(())
    }
}

impl Drop for WgpuFilter {
    fn drop(&mut self) {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
    }
}

aviutl2::register_filter_plugin!(WgpuFilter);
