use onitama::{game::Board, graphics::GFXState};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = Application {
        gfx_state: None,
        egui_renderer: None,
        egui_state: None,
        game: None,
        disciple_tex: None,
        sensei_tex: None
    };

    event_loop.run_app(&mut app).unwrap();
}

struct Application<'a> {
    egui_renderer: Option<egui_wgpu::Renderer>,
    egui_state: Option<egui_winit::State>,
    gfx_state: Option<GFXState<'a>>,
    game: Option<Board>,
    disciple_tex: Option<onitama::graphics::renderer::TexHandle>,
    sensei_tex: Option<onitama::graphics::renderer::TexHandle>,
}
impl ApplicationHandler for Application<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
                    .with_resizable(false)
                    .with_title("Onitama"),
            )
            .unwrap();

        let mut gfx_state = pollster::block_on(GFXState::new(window));

        let egui_renderer = egui_wgpu::Renderer::new(
            &gfx_state.device,
            gfx_state.surface_format(),
            None,
            1,
            false,
        );
        let egui_ctx = egui::Context::default();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        self.game = Some(Board::random_cards());
        self.disciple_tex = Some(gfx_state.load_texture("assets/disciple.png"));
        self.sensei_tex = Some(gfx_state.load_texture("assets/sensei.png"));
        self.egui_state = Some(egui_winit::State::new(egui_ctx.clone(), egui::viewport::ViewportId::ROOT, &gfx_state.window, None, None, None));
        self.egui_renderer = Some(egui_renderer);
        self.gfx_state = Some(gfx_state);
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let gfx_state = self.gfx_state.as_mut().unwrap();                
        let state = self.egui_state.as_mut().unwrap();
        let game = self.game.as_ref().unwrap();

        let _ = state.on_window_event(&gfx_state.window, &event); // Process event with egui
        // if event_response.consumed { return }

        match event {
            WindowEvent::RedrawRequested => {
                const PPP: f32 = 1.0;
                
                let raw_input = state.take_egui_input(&gfx_state.window);
                let mut leftover_rect = egui::Rect::ZERO;
                let full_output = state.egui_ctx().run(raw_input, |ctx| {
                    egui_ui(ctx, game);
                    leftover_rect = ctx.available_rect();
                });
                state.handle_platform_output(&gfx_state.window, full_output.platform_output);

                // Begin render pass
                let (encoder, output_texture) = {
                    let onitama::graphics::RenderingObjects {
                        mut encoder,
                        mut render_pass,
                        output_texture,
                    } = gfx_state.begin_render_pass().expect("surface error");

                    // Render game
                    let game_rect = from_egui_rect(leftover_rect);
                    let game_graphics = onitama::gui::GameGraphics::new(
                        game_rect,
                        game,
                        self.disciple_tex.unwrap(),
                        self.sensei_tex.unwrap()
                    );
                    game_graphics.draw(&mut gfx_state.renderer, game.red_to_move()); // Draw game
                    gfx_state.renderer.render(&gfx_state.queue, &mut render_pass);

                    // Update egui
                    let clipped_prims = state.egui_ctx().tessellate(full_output.shapes, PPP);
                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        pixels_per_point: PPP,
                        size_in_pixels: [WIDTH, HEIGHT],
                    };
                    self.egui_renderer.as_mut().unwrap().update_buffers(&gfx_state.device, &gfx_state.queue, &mut encoder, &clipped_prims, &screen_descriptor);
                    for (id, delta) in full_output.textures_delta.set {
                        self.egui_renderer.as_mut().unwrap()
                            .update_texture(&gfx_state.device, &gfx_state.queue, id, &delta);
                    }
                    // Render egui
                    self.egui_renderer.as_ref().unwrap().render(
                        &mut render_pass,
                        &clipped_prims,
                        &screen_descriptor,
                    );
                    for tex in full_output.textures_delta.free {
                        self.egui_renderer.as_mut().unwrap().free_texture(&tex);
                    }
                    (encoder, output_texture)
                };

                // Present
                gfx_state.queue.submit(std::iter::once(encoder.finish()));
                gfx_state.window.pre_present_notify();
                output_texture.present();

                gfx_state.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn egui_ui(ctx: &egui::Context, game: &Board) {
    egui::SidePanel::left("left panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Cards in use:");
                let mut sorted_cards = game.cards();
                sorted_cards.sort_by(|a, b| {
                    onitama::cards::card_identifier(a).cmp(&onitama::cards::card_identifier(b))
                });
                for card in sorted_cards {
                    ui.label((onitama::cards::card_identifier(&card) as char).to_string());
                }
            });
    });
}

fn from_egui_rect(rect: egui::Rect) -> onitama::graphics::Rect {
    let (min_x, min_y) = (rect.left_top().x, rect.left_top().y);
    let (max_x, max_y) = (rect.right_bottom().x, rect.right_bottom().y);
    onitama::graphics::Rect {
        origin: glam::vec2(min_x, min_y),
        size: glam::vec2(max_x - min_x, max_y - min_y)
    }
}