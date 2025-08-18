use onitama::graphics::GFXState;
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
    };

    event_loop.run_app(&mut app).unwrap();
}

struct Application<'a> {
    gfx_state: Option<GFXState<'a>>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    egui_state: Option<egui_winit::State>,
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

        let gfx_state = pollster::block_on(GFXState::new(window));

        let egui_renderer = egui_wgpu::Renderer::new(
            &gfx_state.device,
            gfx_state.surface_format(),
            None,
            1,
            false,
        );

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let egui_ctx = egui::Context::default();
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
        let gfx_state = self.gfx_state.as_ref().unwrap();                
        let state = self.egui_state.as_mut().unwrap();

        let _ = state.on_window_event(&gfx_state.window, &event); // Process event with egui
        // if event_response.consumed { return }

        match event {
            WindowEvent::RedrawRequested => {
                const PPP: f32 = 1.0;
                
                let raw_input = state.take_egui_input(&gfx_state.window);
                let full_output = state.egui_ctx().run(raw_input, egui_ui);
                state.handle_platform_output(&gfx_state.window, full_output.platform_output);

                let clipped_prims = state.egui_ctx().tessellate(full_output.shapes, PPP);
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    pixels_per_point: PPP,
                    size_in_pixels: [WIDTH, HEIGHT],
                };

                let mut r_objs =
                    gfx_state.begin_render_pass().expect("surface error");
                self.egui_renderer.as_mut().unwrap().update_buffers(&gfx_state.device, &gfx_state.queue, &mut r_objs.encoder, &clipped_prims, &screen_descriptor);

                for (id, delta) in full_output.textures_delta.set {
                    self.egui_renderer.as_mut().unwrap()
                        .update_texture(&gfx_state.device, &gfx_state.queue, id, &delta);
                }
                self.egui_renderer.as_ref().unwrap().render(
                    &mut r_objs.render_pass,
                    &clipped_prims,
                    &screen_descriptor,
                );

                for tex in full_output.textures_delta.free {
                    self.egui_renderer.as_mut().unwrap().free_texture(&tex);
                }

                std::mem::drop(r_objs.render_pass);
                gfx_state.queue.submit(std::iter::once(r_objs.encoder.finish()));
                gfx_state.window.pre_present_notify();
                r_objs.output_texture.present();

                gfx_state.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn egui_ui(ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let frame_nr = ctx.cumulative_frame_nr();
        ui.label("Hello world from the arena!");
        if ui.button("Click here!").clicked() {
            println!("Button clicked!");
        }
        ui.label(format!("Frame {frame_nr}"));
    });
}