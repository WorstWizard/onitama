use glam::Vec2;
use glam::vec2;
use onitama::game::Board;
use onitama::graphics::Rect;
use onitama::graphics::renderer::SimpleRenderer;
use onitama::graphics::renderer::TexHandle;
use onitama::gui::GameGraphics;
use onitama::gui::OnitamaGame;
use std::sync::Arc;
use wgpu::Color;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::Window;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;

// Based on
// https://sotrh.github.io/learn-wgpu/
struct GFXState<'a> {
    surface: wgpu::Surface<'a>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    _config: Arc<wgpu::SurfaceConfiguration>,
    _size: winit::dpi::PhysicalSize<u32>,
    disciple_tex: TexHandle,
    sensei_tex: TexHandle,
    window: Arc<Window>,
    renderer: SimpleRenderer,
}
impl<'a> GFXState<'a> {
    async fn new(window: Window) -> Self {
        let window_arc = Arc::new(window);
        let size = window_arc.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
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
        let mut renderer =
            SimpleRenderer::new(device_arc.clone(), queue_arc.clone(), config_arc.clone());

        // Load textures as RGBA8
        let disciple_img = image::load(
            std::io::BufReader::new(
                std::fs::File::open("assets/disciple.png")
                    .expect("did not find 'assets/disciple.png'"),
            ),
            image::ImageFormat::Png,
        )
        .expect("failed to decode asset")
        .into_rgba8();
        let sensei_img = image::load(
            std::io::BufReader::new(
                std::fs::File::open("assets/sensei.png").expect("did not find 'assets/sensei.png'"),
            ),
            image::ImageFormat::Png,
        )
        .expect("failed to decode asset")
        .into_rgba8();

        let disciple_tex = renderer.make_texture(disciple_img.into());
        let sensei_tex = renderer.make_texture(sensei_img.into());
        Self {
            surface,
            device: device_arc,
            queue: queue_arc,
            _config: config_arc,
            _size: size,
            disciple_tex,
            sensei_tex,
            window: window_arc,
            renderer,
        }
    }
    fn render(
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
                        load: wgpu::LoadOp::Clear(Color::BLACK),
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
}

struct Inputs {
    pub mouse_pressed: bool,
    pub mouse_pos: Vec2,
}

struct Application<'a> {
    gfx_state: Option<GFXState<'a>>,
    game: Option<OnitamaGame>,
    inputs: Inputs,
}
impl ApplicationHandler for Application<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
                    .with_resizable(false)
                    .with_title("Onitama"),
            )
            .unwrap();
        self.gfx_state = Some(pollster::block_on(GFXState::new(window)));
        let game_board = Board::random_cards();
        let game_graphics = GameGraphics::new(
            Rect::new(Vec2::ZERO, vec2(WIDTH as f32, HEIGHT as f32)),
            &game_board,
            self.gfx_state.as_ref().unwrap().disciple_tex,
            self.gfx_state.as_ref().unwrap().sensei_tex,
        );
        let game = OnitamaGame::new(game_graphics, game_board);
        self.game = Some(game);
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            // Close game on ESCAPE
            winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                is_synthetic: _,
            } => {
                event_loop.exit();
            }
            // Update mouse inputs and trigger update on game
            winit::event::WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.inputs.mouse_pos = vec2(position.x as f32, position.y as f32);
                self.game
                    .as_mut()
                    .unwrap()
                    .handle_mouse_input(self.inputs.mouse_pressed, self.inputs.mouse_pos);
                self.redraw_window();
                self.game_end(event_loop);
            }
            winit::event::WindowEvent::MouseInput {
                button,
                device_id: _,
                state,
            } => {
                self.inputs.mouse_pressed = button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Pressed;
                self.game
                    .as_mut()
                    .unwrap()
                    .handle_mouse_input(self.inputs.mouse_pressed, self.inputs.mouse_pos);
                self.redraw_window();
                self.game_end(event_loop);
            }
            winit::event::WindowEvent::RedrawRequested => {
                match self.gfx_state.as_mut().unwrap().render(
                    &self.game.as_ref().unwrap().graphics,
                    self.game.as_ref().unwrap().board.red_to_move(),
                ) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        log::error!("Surface lost or resized");
                        event_loop.exit();
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory");
                        event_loop.exit();
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout")
                    }
                }
            }
            _ => (),
        }
    }
}
impl Application<'_> {
    fn redraw_window(&self) {
        self.gfx_state.as_ref().unwrap().window.request_redraw();
    }
    fn game_end(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(red_won) = self.game.as_ref().unwrap().winner() {
            if red_won {
                println!("Red wins!")
            } else {
                println!("Blue wins!")
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            event_loop.exit();
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = Application {
        gfx_state: None,
        game: None,
        inputs: Inputs {
            mouse_pressed: false,
            mouse_pos: vec2(0.0, 0.0),
        },
    };

    event_loop.run_app(&mut app).unwrap();
}
