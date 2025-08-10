use super::{Color, Rect};
use glam::{Vec2, vec2};
use std::{ops::Range, sync::Arc};

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: Vec2,
    tex: Vec2,
    col: Color,
}

struct Texture {
    _texture: wgpu::Texture,
    _extent: wgpu::Extent3d,
    _view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

/// Handle (id) of a created texture; can be copied and passed around
#[derive(Clone, Copy)]
pub struct TexHandle(usize, u32, u32);
impl TexHandle {
    pub fn size(&self) -> (u32, u32) {
        (self.1, self.2)
    }
}

const VERT_BUFFER_SIZE: u64 = 1 << 20; // 1MiB, hardcoded, should be complete overkill for this program
pub struct SimpleRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface_config: Arc<wgpu::SurfaceConfiguration>,
    vertex_buffer: Arc<wgpu::Buffer>,
    vertex_queue: Vec<Vertex>,
    colored_pipeline: wgpu::RenderPipeline,
    textured_pipeline: wgpu::RenderPipeline,
    textures: Vec<Texture>,
    texture_sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    draw_commands: Vec<DrawCMD>,
}
impl SimpleRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_config: Arc<wgpu::SurfaceConfiguration>,
    ) -> Self {
        let colored_shader = device.create_shader_module(wgpu::include_wgsl!("filled_color.wgsl"));
        let textured_shader = device.create_shader_module(wgpu::include_wgsl!("textured.wgsl"));
        let vertex_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: VERT_BUFFER_SIZE,
        }));
        let vert_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::offset_of!(Vertex, tex) as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::offset_of!(Vertex, col) as u64,
                    shader_location: 2,
                },
            ],
        };
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        let make_pipeline = |shader_module: wgpu::ShaderModule,
                             layout: Option<&wgpu::PipelineLayout>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("pipeline"),
                layout,
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &[vert_buffer_layout.clone()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        format: surface_config.format,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            })
        };

        let colored_pipeline = make_pipeline(colored_shader, None);
        let textured_pipeline = make_pipeline(textured_shader, Some(&pipeline_layout));

        SimpleRenderer {
            device,
            queue,
            surface_config,
            vertex_buffer,
            vertex_queue: vec![],
            textured_pipeline,
            textures: vec![],
            texture_sampler,
            texture_bind_group_layout,
            colored_pipeline,
            draw_commands: vec![],
        }
    }

    /// Creates texture on GPU from provided RGBA8 image, returns handle to be used for drawing later
    pub fn make_texture(&mut self, image: image::DynamicImage) -> TexHandle {
        let texture_extent = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture descriptor"),
            size: texture_extent,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            image.as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width()),
                rows_per_image: Some(image.height()),
            },
            texture_extent,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                },
            ],
        });
        self.textures.push(Texture {
            _texture: texture,
            _extent: texture_extent,
            _view: view,
            bind_group,
        });

        TexHandle(
            self.textures.len() - 1,
            texture_extent.width,
            texture_extent.height,
        )
    }

    fn quad_vertices(&self, rect: Rect, modulate_color: Color) -> [Vertex; 6] {
        let pos = self.window_to_clip_pos(rect.origin);
        let (width, height) = self.window_to_clip_scale(rect.size).into();
        let col = modulate_color;
        [
            Vertex {
                pos,
                col,
                tex: vec2(0.0, 0.0),
            },
            Vertex {
                pos: pos + vec2(width, 0.0),
                col,
                tex: vec2(1.0, 0.0),
            },
            Vertex {
                pos: pos + vec2(0.0, -height),
                col,
                tex: vec2(0.0, 1.0),
            },
            Vertex {
                pos: pos + vec2(0.0, -height),
                col,
                tex: vec2(0.0, 1.0),
            },
            Vertex {
                pos: pos + vec2(width, 0.0),
                col,
                tex: vec2(1.0, 0.0),
            },
            Vertex {
                pos: pos + vec2(width, -height),
                col,
                tex: vec2(1.0, 1.0),
            },
        ]
    }

    /// Rectangle specified in window coordinates.
    /// Origin is taken as the top-left corner of the rectangle.
    pub fn draw_filled_rect(&mut self, rect: Rect, color: Color) {
        let a = self.vertex_queue.len() as u32;
        self.vertex_queue.extend(self.quad_vertices(rect, color));
        let b = self.vertex_queue.len() as u32;
        self.draw_commands.push(DrawCMD::Fill(a..b));
    }

    /// Rectangle specified in window coordinates.
    /// Origin is taken as the top-left corner of the rectangle.
    pub fn draw_textured_rect(
        &mut self,
        rect: Rect,
        modulate_color: Color,
        texture_handle: TexHandle,
    ) {
        let a = self.vertex_queue.len() as u32;
        self.vertex_queue
            .extend(self.quad_vertices(rect, modulate_color));
        let b = self.vertex_queue.len() as u32;
        self.draw_commands
            .push(DrawCMD::Textured(a..b, texture_handle));
    }

    pub fn render(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass) {
        let mut buffer_offset = 0;

        // Write vertices to buffer
        if !self.vertex_queue.is_empty() {
            let vertex_bytes = bytemuck::cast_slice(&self.vertex_queue);
            let n_bytes = vertex_bytes.len() as u64;
            queue.write_buffer(&self.vertex_buffer, 0, vertex_bytes);
            buffer_offset += n_bytes;
        }
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..buffer_offset));

        // Consume draw commands
        let mut n1 = 0;
        let mut n2 = 0;
        for cmd in self.draw_commands.drain(..) {
            match cmd {
                DrawCMD::Fill(range) => {
                    render_pass.set_pipeline(&self.colored_pipeline);
                    render_pass.draw(range, 0..1);
                    n1 += 1;
                }
                DrawCMD::Textured(range, texture_handle) => {
                    let tex = &self.textures[texture_handle.0];
                    render_pass.set_pipeline(&self.textured_pipeline);
                    render_pass.set_bind_group(0, &tex.bind_group, &[]);
                    render_pass.draw(range, 0..1);
                    n2 += 1;
                }
            }
        }
        println!("{n1} rect draw calls");
        println!("{n2} sprite draw calls");
        self.vertex_queue.clear();
    }

    pub fn output_size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    fn window_to_clip_pos(&self, pos: Vec2) -> Vec2 {
        let (width_px, height_px) = self.output_size();
        vec2(
            (pos.x / width_px as f32) * 2.0 - 1.0,
            ((height_px as f32 - pos.y) / height_px as f32) * 2.0 - 1.0,
        )
    }
    fn window_to_clip_scale(&self, vec: Vec2) -> Vec2 {
        let (width_px, height_px) = self.output_size();
        vec2(
            vec.x / width_px as f32 * 2.0,
            vec.y / height_px as f32 * 2.0,
        )
    }
}

#[derive(Clone)]
enum DrawCMD {
    /// Vertex range in buffer
    Fill(Range<u32>),
    /// Vertex range in buffer and texture handle
    Textured(Range<u32>, TexHandle),
}
