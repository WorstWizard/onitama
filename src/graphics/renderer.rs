use std::sync::Arc;

use glam::{vec2, vec3, Vec2, Vec3};
pub type Color = Vec3;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct VertexColor {
    pos: Vec3,
    col: Color
}

const VERT_BUFFER_SIZE: u64 = 1<<20; // 1MiB, hardcoded, should be complete overkill for this program
pub struct SimpleRenderer {
    vertex_buffer: Arc<wgpu::Buffer>,
    surface_config: Arc<wgpu::SurfaceConfiguration>,
    colored_pipeline: wgpu::RenderPipeline,
    colored_vert_queue: Vec<VertexColor>,
    last_z_level: f32,
}
impl SimpleRenderer {
    pub fn new(device: &wgpu::Device, surface_config: Arc<wgpu::SurfaceConfiguration>) -> Self {
        let colored_shader = device.create_shader_module(wgpu::include_wgsl!("filled_color.wgsl"));
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            size: VERT_BUFFER_SIZE,
        });
        let vert_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexColor>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::offset_of!(VertexColor, col) as u64,
                    shader_location: 1,
                },
            ],
        };
        let colored_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &colored_shader,
                entry_point: Some("vs_main"),
                buffers: &[vert_buffer_layout],
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
                module: &colored_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::REPLACE),
                    format: surface_config.format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        SimpleRenderer {
            surface_config,
            vertex_buffer: Arc::new(vertex_buffer),
            colored_pipeline,
            colored_vert_queue: vec![],
            last_z_level: 1.0  // WebGPU NDC goes from 0 to 1, start at 1 and move primitives back to front
        }
    }

    /// Rectangle specified in window coordinates.
    /// Origin is taken as the top-left corner of the rectangle.
    pub fn draw_filled_rect(&mut self, origin: Vec2, width: f32, height: f32, color: Color) {
        let z =  self.last_z_level - f32::EPSILON;
        self.last_z_level = z;
        let pos_clip = self.window_to_clip_pos(origin);
        let (width, height) = self.window_to_clip_scale(vec2(width, height)).into();

        let pos = vec3(pos_clip.x, pos_clip.y, z);
        let vertices = [
            VertexColor { pos, col: color },
            VertexColor { pos: pos + Vec3 { x: width, y: 0.0, z: 0.0 }, col: color },
            VertexColor { pos: pos + Vec3 { x: 0.0, y: -height, z: 0.0 }, col: color },
            VertexColor { pos: pos + Vec3 { x: 0.0, y: -height, z: 0.0 }, col: color },
            VertexColor { pos: pos + Vec3 { x: width, y: 0.0, z: 0.0 }, col: color },
            VertexColor { pos: pos + Vec3 { x: width, y: -height, z: 0.0 }, col: color },
        ];
        self.colored_vert_queue.extend(vertices);
    }

    pub fn render(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass) {
        // Uniform color
        {
            let vertex_bytes = bytemuck::cast_slice(&self.colored_vert_queue);
            let n_verts = self.colored_vert_queue.len() as u64;
            let n_bytes = vertex_bytes.len() as u64;

            // Write vertices to vert buffer
            queue.write_buffer(&self.vertex_buffer, 0, vertex_bytes);

            // Draw
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..n_bytes));
            render_pass.set_pipeline(&self.colored_pipeline);
            render_pass.draw(0..n_verts as u32, 0..1);

            // Reset
            self.colored_vert_queue.clear();
            self.last_z_level = 1.0;
        }
    }

    pub fn output_size(&self) -> (u32,u32) {
        (
            self.surface_config.width,
            self.surface_config.height,
        )
    }

    fn window_to_clip_pos(&self, pos: Vec2) -> Vec2 {
        let (width_px, height_px) = self.output_size();
        vec2(
            (pos.x / width_px as f32) * 2.0 - 1.0,
            ((height_px as f32 - pos.y) / height_px as f32) * 2.0 - 1.0
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