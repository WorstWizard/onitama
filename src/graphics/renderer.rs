use std::sync::Arc;

use glam::{vec3, Vec2, Vec3};

type Color = Vec3;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct VertexColor {
    pos: Vec3,
    col: Color
}

const VERT_BUFFER_SIZE: u64 = 1024; // 1MiB, hardcoded, should be overkill for this program
pub struct SimpleRenderer {
    device: Arc<wgpu::Device>,
    vertex_buffer: Arc<wgpu::Buffer>,
    colored_pipeline: wgpu::RenderPipeline,
    colored_vert_queue: Vec<VertexColor>,
    last_z_level: f32,
}
impl SimpleRenderer {
    pub fn new(device: Arc<wgpu::Device>, out_format: wgpu::TextureFormat) -> Self {
        let colored_shader = device.create_shader_module(wgpu::include_wgsl!("filled_color.wgsl"));
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::MAP_WRITE,
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
                    format: out_format,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        SimpleRenderer { device, vertex_buffer: Arc::new(vertex_buffer), colored_pipeline, colored_vert_queue: vec![], last_z_level: 0.0 }
    }

    pub fn draw_filled_rect(&mut self, origin: Vec2, width: f32, height: f32, color: Color) {
        let z =  self.last_z_level - 1.0;
        let origin = vec3(origin.x, origin.y, z);

        let vertices = [
            VertexColor { pos: origin, col: color },
            VertexColor { pos: origin + Vec3 { x: width, y: 0.0, z: 0.0 }, col: color },
            VertexColor { pos: origin + Vec3 { x: 0.0, y: height, z: 0.0 }, col: color },
            VertexColor { pos: origin + Vec3 { x: 0.0, y: height, z: 0.0 }, col: color },
            VertexColor { pos: origin + Vec3 { x: width, y: 0.0, z: 0.0 }, col: color },
            VertexColor { pos: origin + Vec3 { x: width, y: height, z: 0.0 }, col: color },
        ];
        self.colored_vert_queue.extend(vertices);
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        // Uniform color
        let vertex_bytes = bytemuck::cast_slice(&self.colored_vert_queue);
        let n_bytes = vertex_bytes.len() as u64;
        let (sender, receiver) = std::sync::mpsc::channel();
        self.vertex_buffer.slice(..n_bytes).map_async(wgpu::MapMode::Write, move |result| {
            sender.send(result).expect("failed to send msg");
        });
        self.device.poll(wgpu::Maintain::Wait); // Wait for buffer to map
        if receiver.recv().is_ok() {
            let mut view = self.vertex_buffer.slice(..n_bytes).get_mapped_range_mut();
            view.clone_from_slice(vertex_bytes);
        } else {
            panic!("failed to map vertex buffer")
        }
        
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_pipeline(&self.colored_pipeline);
        render_pass.draw(0..self.colored_vert_queue.len() as u32, 0..1);
    }
}