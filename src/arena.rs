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
    };

    event_loop.run_app(&mut app).unwrap();
}

struct Application<'a> {
    gfx_state: Option<GFXState<'a>>,
    egui_renderer: Option<egui_wgpu::Renderer>,
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

        self.gfx_state = Some(gfx_state);
        self.egui_renderer = Some(egui_renderer);
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                println!("drawing");
                let mut ctx = egui::Context::default();
                let gfx_state = self.gfx_state.as_ref().unwrap();
                let full_output = test_ui(&mut ctx);
                let clipped_prims = ctx.tessellate(full_output.shapes, 1.0);
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        pixels_per_point: 1.0,
                        size_in_pixels: [WIDTH, HEIGHT],
                    };
                let mut r_objs =
                    gfx_state.begin_render_pass().expect("surface error");
                self.egui_renderer.as_mut().unwrap().update_buffers(&gfx_state.device, &gfx_state.queue, &mut r_objs.encoder, &clipped_prims, &screen_descriptor);
                println!("buffers updated");
                for (id, delta) in full_output.textures_delta.set {
                    self.egui_renderer.as_mut().unwrap()
                        .update_texture(&gfx_state.device, &gfx_state.queue, id, &delta);
                }
                println!("textures updated");
                self.egui_renderer.as_ref().unwrap().render(
                    &mut r_objs.render_pass,
                    &clipped_prims,
                    &screen_descriptor,
                );
                println!("egui rendered");

                for tex in full_output.textures_delta.free {
                    self.egui_renderer.as_mut().unwrap().free_texture(&tex);
                }

                std::mem::drop(r_objs.render_pass);
                gfx_state.queue.submit(std::iter::once(r_objs.encoder.finish()));
                println!("encoder finished");
                gfx_state.window.pre_present_notify();
                r_objs.output_texture.present();
                println!("presented texture");
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn test_ui(ctx: &mut egui::Context) -> egui::FullOutput {
    let test_input = egui::RawInput {
        ..Default::default()
    };
    let full_output = ctx.run(test_input, |ctx| {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("Hello world from the arena!");
        });
    });
    full_output
}

fn handle_output(platform_output: egui::PlatformOutput) {
    // todo!()
}
