use crate::gui::content::{ContentConstructor, Renderable};
use crate::media::video::player::VideoPlayer;
use crate::media::video::renderer::VideoRenderer;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

pub struct VideoContent {
    pub player: Arc<VideoPlayer>,
}

impl VideoContent {
    pub fn new(player: Arc<VideoPlayer>) -> Self {
        Self { player }
    }
}

impl ContentConstructor for VideoContent {
    fn create_renderer(
        &self,
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
    ) -> Result<Box<dyn Renderable>> {
        let renderer = VideoRenderable::new(device, queue, config, self.player.clone())?;
        Ok(Box::new(renderer))
    }
}

struct VideoRenderable {
    #[allow(dead_code)]
    player: Arc<VideoPlayer>,
    renderer: VideoRenderer,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    opacity_buffer: wgpu::Buffer,
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl VideoRenderable {
    pub fn new(
        device: &Device,
        _queue: &Queue,
        config: &SurfaceConfiguration,
        player: Arc<VideoPlayer>,
    ) -> Result<Self> {
        let renderer = VideoRenderer::new(player.get_mpv())?;

        let width = config.width;
        let height = config.height;

        let (texture, texture_view, sampler) =
            Self::create_texture_resources(device, width, height);

        let opacity_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Opacity Buffer"),
            contents: bytemuck::cast_slice(&[1.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: opacity_buffer.as_entire_binding(),
                },
            ],
            label: Some("texture_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../image/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let buffer = vec![0u8; (width * height * 4) as usize];

        Ok(Self {
            player,
            renderer,
            texture,
            bind_group,
            pipeline,
            opacity_buffer,
            buffer,
            width,
            height,
            bind_group_layout,
            sampler,
        })
    }

    fn create_texture_resources(
        device: &Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Video Texture"),
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        (texture, texture_view, sampler)
    }
}

impl Renderable for VideoRenderable {
    fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        queue: &Queue,
        opacity: f32,
    ) {
        queue.write_buffer(&self.opacity_buffer, 0, bytemuck::cast_slice(&[opacity]));

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn update(&mut self, queue: &Queue) -> Option<Instant> {
        if self.renderer.update() {
            // New frame available
            let stride = self.width as i32 * 4;
            if self
                .renderer
                .render_sw(
                    self.width as i32,
                    self.height as i32,
                    stride,
                    "rgba",
                    &mut self.buffer,
                )
                .is_ok()
            {
                let texture_size = wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                };

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &self.buffer,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * self.width),
                        rows_per_image: Some(self.height),
                    },
                    texture_size,
                );

                return Some(Instant::now());
            }
        }

        // Poll every 16ms
        Some(Instant::now() + std::time::Duration::from_millis(16))
    }

    fn resize(&mut self, device: &Device, _queue: &Queue, config: &SurfaceConfiguration) {
        if config.width > 0
            && config.height > 0
            && (config.width != self.width || config.height != self.height)
        {
            self.width = config.width;
            self.height = config.height;
            self.buffer
                .resize((self.width * self.height * 4) as usize, 0);

            let (texture, texture_view, _) =
                Self::create_texture_resources(device, self.width, self.height);
            self.texture = texture;

            // Recreate bind group
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.opacity_buffer.as_entire_binding(),
                    },
                ],
                label: Some("texture_bind_group"),
            });
        }
    }
}
