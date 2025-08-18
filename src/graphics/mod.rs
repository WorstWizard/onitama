use std::sync::Arc;

use glam::{Vec2, Vec3};
use wgpu::TextureFormat;
use winit::window::Window;

use crate::{
    graphics::renderer::{SimpleRenderer, TexHandle},
    gui::GameGraphics,
};

pub mod board;
pub mod card;
pub mod piece;
pub mod renderer;

// const ANIM_TIME: f32 = 0.25;
pub type Color = Vec3;
#[derive(Clone, Copy)]
pub struct Rect {
    pub origin: Vec2,
    pub size: Vec2,
}
impl Rect {
    pub fn new(origin: Vec2, size: Vec2) -> Self {
        Rect { origin, size }
    }
    pub fn contains_point(&self, pos: Vec2) -> bool {
        pos.x >= self.origin.x
            && pos.x < self.origin.x + self.size.x
            && pos.y >= self.origin.y
            && pos.y < self.origin.y + self.size.y
    }
}
#[test]
fn contains_point() {
    use glam::vec2;
    let rect = Rect::new(vec2(1.0, 2.0), vec2(10.0, 20.0));
    assert!(rect.contains_point(vec2(5.0, 5.0)));
    assert!(!rect.contains_point(vec2(11.0, 5.0)));
}

mod colors {
    use super::Color;
    pub const BOARD_TILE: Color = Color::new(1.0, 1.0, 1.0);
    pub const BOARD_BG: Color = Color::new(0.5, 0.5, 0.5);
    pub const BOARD_HIGHLIGHT: Color = Color::new(1.0, 1.0, 0.0);
    pub const BOARD_RED_TEMPLE: Color = Color::new(200.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0);
    pub const BOARD_BLUE_TEMPLE: Color = Color::new(50.0 / 255.0, 50.0 / 255.0, 200.0 / 255.0);
    pub const CARD_BG: Color = Color::new(200.0 / 255.0, 200.0 / 255.0, 170.0 / 255.0);
    pub const CARD_TILE_BG: Color = Color::new(230.0 / 255.0, 230.0 / 255.0, 200.0 / 255.0);
    pub const CARD_TILE: Color = Color::new(130.0 / 255.0, 130.0 / 255.0, 100.0 / 255.0);
    pub const CARD_SELECTED: Color = Color::new(250.0 / 255.0, 250.0 / 255.0, 220.0 / 255.0);
    pub const CARD_CENTER: Color = Color::new(80.0 / 255.0, 80.0 / 255.0, 40.0 / 255.0);
    pub const PIECE_RED: Color = Color::new(1.0, 0.2, 0.2);
    pub const PIECE_BLUE: Color = Color::new(0.2, 0.2, 1.0);
}

// Based on
// https://sotrh.github.io/learn-wgpu/
pub struct GFXState<'a> {
    pub surface: wgpu::Surface<'a>,
    surface_format: TextureFormat,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    _config: Arc<wgpu::SurfaceConfiguration>,
    _size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub renderer: SimpleRenderer,
}
impl<'a> GFXState<'a> {
    pub async fn new(window: Window) -> Self {
        let window_arc = Arc::new(window);
        let size = window_arc.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window_arc.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
                label: None,
            })
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let device_arc = Arc::new(device);
        let queue_arc = Arc::new(queue);
        let config_arc = Arc::new(config);
        let renderer =
            SimpleRenderer::new(device_arc.clone(), queue_arc.clone(), config_arc.clone());

        Self {
            surface,
            surface_format,
            device: device_arc,
            queue: queue_arc,
            _config: config_arc,
            _size: size,
            window: window_arc,
            renderer,
        }
    }
    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }
    /// Loads a texture from a **PNG** and returns handle to it
    pub fn load_texture(&mut self, path: &str) -> TexHandle {
        let img = image::load(
            std::io::BufReader::new(
                std::fs::File::open(path).unwrap_or_else(|_| panic!("did not find {path}")),
            ),
            image::ImageFormat::Png,
        )
        .expect("failed to decode asset")
        .into_rgba8();
        self.renderer.make_texture(img.into())
    }
    pub fn render(
        &mut self,
        game_graphics: &GameGraphics,
        red_to_move: bool,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            game_graphics.draw(&mut self.renderer, red_to_move);

            self.renderer.render(&self.queue, &mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        output.present();
        Ok(())
    }
    /// Returns command encoder, associated render pass (static lifetime) and output surface texture
    /// Mutating command encoder before render pass is finished recording is a runtime error
    pub fn begin_render_pass(
        &self,
    ) -> Result<RenderingObjects, wgpu::SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let render_pass = {
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime()
        };
        Ok(RenderingObjects { encoder, render_pass, output_texture })
    }
}

pub struct RenderingObjects {
    pub encoder: wgpu::CommandEncoder,
    pub render_pass: wgpu::RenderPass<'static>,
    pub output_texture: wgpu::SurfaceTexture
}