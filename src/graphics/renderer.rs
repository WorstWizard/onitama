use std::sync::Arc;

use glam::{vec2, vec3, Vec2, Vec3};
pub type Color = Vec3;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: Vec3,
    col: Color,
    tex: Vec2,
}

struct Texture {
    texture: wgpu::Texture,
    extent: wgpu::Extent3d,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}
#[derive(Clone, Copy)]
pub struct TexHandle(usize);

const VERT_BUFFER_SIZE: u64 = 1 << 20; // 1MiB, hardcoded, should be complete overkill for this program
pub struct SimpleRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    vertex_buffer: Arc<wgpu::Buffer>,
    surface_config: Arc<wgpu::SurfaceConfiguration>,
    colored_pipeline: wgpu::RenderPipeline,
    colored_vert_queue: Vec<Vertex>,
    textured_pipeline: wgpu::RenderPipeline,
    textured_vert_queues: Vec<Vec<Vertex>>,
    textures: Vec<Texture>,
    texture_sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    last_z_level: f32,
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
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::offset_of!(Vertex, col) as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::offset_of!(Vertex, tex) as u64,
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
            textured_pipeline,
            textures: vec![],
            textured_vert_queues: vec![],
            texture_sampler,
            texture_bind_group_layout,
            colored_pipeline,
            colored_vert_queue: vec![],
            last_z_level: 1.0, // WebGPU NDC goes from 0 to 1, start at 1 and move primitives back to front
        }
    }

    /// Creates texture on GPU from provided RGBA8 image, returns 'handle' to be used for drawing later
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
            texture,
            extent: texture_extent,
            view,
            bind_group,
        });
        self.textured_vert_queues.push(vec![]);
        TexHandle(self.textures.len() - 1)
    }

    /// Rectangle specified in window coordinates.
    /// Origin is taken as the top-left corner of the rectangle.
    pub fn draw_filled_rect(&mut self, origin: Vec2, width: f32, height: f32, color: Color) {
        let z = self.last_z_level - f32::EPSILON;
        self.last_z_level = z;
        let pos_clip = self.window_to_clip_pos(origin);
        let (width, height) = self.window_to_clip_scale(vec2(width, height)).into();

        let pos = vec3(pos_clip.x, pos_clip.y, z);
        let vertices = [
            Vertex {
                pos,
                col: color,
                tex: Vec2::default(),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: width,
                        y: 0.0,
                        z: 0.0,
                    },
                col: color,
                tex: Vec2::default(),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: 0.0,
                        y: -height,
                        z: 0.0,
                    },
                col: color,
                tex: Vec2::default(),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: 0.0,
                        y: -height,
                        z: 0.0,
                    },
                col: color,
                tex: Vec2::default(),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: width,
                        y: 0.0,
                        z: 0.0,
                    },
                col: color,
                tex: Vec2::default(),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: width,
                        y: -height,
                        z: 0.0,
                    },
                col: color,
                tex: Vec2::default(),
            },
        ];
        self.colored_vert_queue.extend(vertices);
    }

    /// Rectangle specified in window coordinates.
    /// Origin is taken as the top-left corner of the rectangle.
    pub fn draw_textured_rect(&mut self, origin: Vec2, width: f32, height: f32, modulate_color: Color, texture_handle: TexHandle) {
        let z = self.last_z_level - f32::EPSILON;
        self.last_z_level = z;
        let pos_clip = self.window_to_clip_pos(origin);
        let (width, height) = self.window_to_clip_scale(vec2(width, height)).into();

        let pos = vec3(pos_clip.x, pos_clip.y, z);
        let vertices = [
            Vertex {
                pos,
                col: modulate_color,
                tex: vec2(0.0, 0.0),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: width,
                        y: 0.0,
                        z: 0.0,
                    },
                col: modulate_color,
                tex: vec2(1.0, 0.0),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: 0.0,
                        y: -height,
                        z: 0.0,
                    },
                col: modulate_color,
                tex: vec2(0.0, 1.0),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: 0.0,
                        y: -height,
                        z: 0.0,
                    },
                col: modulate_color,
                tex: vec2(0.0, 1.0),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: width,
                        y: 0.0,
                        z: 0.0,
                    },
                col: modulate_color,
                tex: vec2(1.0, 0.0),
            },
            Vertex {
                pos: pos
                    + Vec3 {
                        x: width,
                        y: -height,
                        z: 0.0,
                    },
                col: modulate_color,
                tex: vec2(1.0, 1.0),
            },
        ];
        self.textured_vert_queues[texture_handle.0].extend(vertices);
    }

    pub fn render(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass) {
        let mut buffer_offset = 0;
        let mut vert_offset = 0;

        // Uniform color
        let mut colored_range = 0..0;
        if !self.colored_vert_queue.is_empty() {
            let vertex_bytes = bytemuck::cast_slice(&self.colored_vert_queue);
            let n_verts = self.colored_vert_queue.len() as u32;
            let n_bytes = vertex_bytes.len() as u64;
            queue.write_buffer(&self.vertex_buffer, 0, vertex_bytes);
            buffer_offset += n_bytes;
            vert_offset += n_verts;
            colored_range = 0..n_verts;
        }
        // Textures
        let mut textured_ranges = vec![];
        if self.textured_vert_queues.iter().any(|vec| !vec.is_empty()) {
            // Submit all texture verts in one go by flattening, then render slice of the vertex buffer as neccesary
            let mut flattened_verts: Vec<Vertex> = vec![];
            for vert_queue in &self.textured_vert_queues {
                let n_verts = vert_queue.len() as u32;
                flattened_verts.extend(vert_queue);
                textured_ranges.push(vert_offset..vert_offset + n_verts);
                vert_offset += n_verts;
            }

            // Write all vertices once
            let vertex_bytes = bytemuck::cast_slice(&flattened_verts);
            let n_bytes = vertex_bytes.len() as u64;
            queue.write_buffer(&self.vertex_buffer, buffer_offset, vertex_bytes);
            buffer_offset += n_bytes;
        }
        // Set buffer
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..buffer_offset));

        // Colored draw
        if !colored_range.is_empty() {
            render_pass.set_pipeline(&self.colored_pipeline);
            render_pass.draw(colored_range, 0..1);
        }
        // Textured draw
        // One draw call per texture
        for (texture, vert_range) in self.textures.iter().zip(textured_ranges.iter()) {
            if !vert_range.is_empty() {
                render_pass.set_pipeline(&self.textured_pipeline);
                render_pass.set_bind_group(0, &texture.bind_group, &[]);
                render_pass.draw(vert_range.clone(), 0..1);
            }
        }

        // Reset
        self.colored_vert_queue.clear();
        self.textured_vert_queues
            .iter_mut()
            .for_each(|vec| vec.clear());
        self.last_z_level = 1.0;
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