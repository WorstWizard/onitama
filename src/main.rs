use std::sync::Arc;

use onitama::graphics::renderer::SimpleRenderer;
use onitama::graphics::renderer::TexHandle;
// use onitama::ai::AIOpponent;
// use onitama::game::*;
// use onitama::graphics::*;
// use wgpu::Color;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::ElementState;
use winit::event::KeyEvent;
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::window::Window;

use wgpu::Color;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
// const FRAMERATE: u64 = 60;
// const AI_OPPONENT: bool = true;
// const AI_MAX_DEPTH: u32 = 4;

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
                std::fs::File::open("assets/disciple.png").expect("did not find 'assets/disciple.png'")
            ),
            image::ImageFormat::Png
        ).expect("failed to decode asset").into_rgba8();
        let sensei_img = image::load(
            std::io::BufReader::new(
                std::fs::File::open("assets/sensei.png").expect("did not find 'assets/sensei.png'")
            ),
            image::ImageFormat::Png
        ).expect("failed to decode asset").into_rgba8();

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
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            let test_board = onitama::game::Board::random_cards();
            let test_gfx_board = onitama::graphics::board::GraphicBoard::new(&self.renderer);
            test_gfx_board.draw_board(&mut self.renderer);
            // test_gfx_board.highlight_tiles(
            //     &mut self.renderer,
            //     &[
            //         onitama::game::Pos::from_index(0),
            //         onitama::game::Pos::from_index(1),
            //         onitama::game::Pos::from_index(5),
            //     ],
            // );
            let test_piece_manager = onitama::graphics::piece::PieceGraphicsManager::new(&test_gfx_board, &test_board, self.disciple_tex, self.sensei_tex);
            test_piece_manager.draw(&mut self.renderer);
            // self.renderer.draw_textured_rect(vec2(70.0, 30.0), 100.0, 100.0, vec3(1.0, 0.0, 0.0), self.sensei_tex);
            // self.renderer.draw_textured_rect(vec2(10.0, 10.0), 100.0, 100.0, vec3(1.0, 1.0, 1.0), self.disciple_tex);
            self.renderer.render(&self.queue, &mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        output.present();
        Ok(())
    }
}

struct OnitamaApp<'a> {
    gfx_state: Option<GFXState<'a>>,
}
impl ApplicationHandler for OnitamaApp<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
                    // .with_active(true)
                    .with_resizable(false)
                    // .with_visible(true)
                    .with_title("Test"),
            )
            .unwrap();
        self.gfx_state = Some(pollster::block_on(GFXState::new(window)));
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
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
            winit::event::WindowEvent::RedrawRequested => {
                match self.gfx_state.as_mut().unwrap().render() {
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

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = OnitamaApp { gfx_state: None };

    event_loop.run_app(&mut app).unwrap();

    // mixer::open_audio(
    //     mixer::DEFAULT_FREQUENCY,
    //     mixer::DEFAULT_FORMAT,
    //     mixer::DEFAULT_CHANNELS,
    //     1024,
    // )
    // .unwrap();
    // let audio_loader = sdl2::rwops::RWops::from_file("assets/tap_sound.wav", "r").unwrap();
    // let tap_sound = audio_loader.load_wav().unwrap();

    // let play_tap = || {
    //     mixer::Channel::all().play(&tap_sound, 0).unwrap();
    // };

    // let window = video_subsystem
    //     .window("Onitama", WIDTH, HEIGHT)
    //     .position_centered()
    //     .build()
    //     .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();
    // canvas.set_draw_color(Color::BLACK);
    // canvas.clear();
    // canvas.present();

    // // Load textures for the pieces
    // let tex_creator = canvas.texture_creator();
    // let piece_textures = PieceTextures::init(&tex_creator);

    // // Make game board, set up graphics
    // let mut game_board = Board::random_cards();
    // let graphic_board = GraphicBoard::new(&canvas);
    // let mut piece_graphics =
    //     PieceGraphicsManager::new(&graphic_board, &game_board, &piece_textures);
    // let mut position_highlights = Vec::new();
    // let mut card_graphics = CardGraphicManager::new(
    //     &game_board,
    //     Rect::new(
    //         graphic_board.board_width() as i32,
    //         0,
    //         WIDTH - graphic_board.board_width(),
    //         HEIGHT,
    //     ),
    // );

    // // Animator for sliding pieces
    // let mut move_animator: Option<MoveAnimator> = None;

    // // AI
    // let blue_ai = onitama::ai::MinMax::new(AI_MAX_DEPTH);

    // // Inputs
    // let mut inputs = Inputs {
    //     mouse_pressed: false,
    //     mouse_just_pressed: false,
    //     mouse_just_released: false,
    //     mouse_pos: (0, 0),
    // };

    // // Start event loop
    // let mut fps_manager = FPSManager::new(FRAMERATE);
    // let mut event_pump = sdl_ctx.event_pump().unwrap();
    // 'main: loop {
    //     canvas.set_draw_color(Color::BLACK);
    //     canvas.clear();

    //     // Manage inputs
    //     inputs.mouse_just_pressed = false;
    //     inputs.mouse_just_released = false;
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => break 'main,
    //             Event::MouseButtonDown {
    //                 mouse_btn: MouseButton::Left,
    //                 x,
    //                 y,
    //                 ..
    //             } => {
    //                 inputs.mouse_pressed = true;
    //                 inputs.mouse_just_pressed = true;
    //                 inputs.mouse_pos = (x, y);
    //             }
    //             Event::MouseButtonUp {
    //                 mouse_btn: MouseButton::Left,
    //                 ..
    //             } => {
    //                 inputs.mouse_pressed = false;
    //                 inputs.mouse_just_released = true;
    //             }
    //             Event::MouseMotion { x, y, .. } => {
    //                 inputs.mouse_pos = (x, y);
    //             }
    //             _ => (),
    //         }
    //     }

    //     if move_animator
    //         .as_ref()
    //         .is_some_and(|animator| animator.animating())
    //     {
    //         let delta_time = fps_manager.time_per_frame();
    //         let finished_animation = move_animator.as_mut().unwrap().animate(
    //             piece_graphics.selected_piece_mut().unwrap(),
    //             delta_time.as_secs_f32(),
    //         );
    //         if finished_animation {
    //             piece_graphics.unselect();
    //             card_graphics.swap_cards();
    //             play_tap();
    //         }
    //     } else if !AI_OPPONENT || game_board.red_to_move() {
    //         fn return_piece(
    //             graphic_board: &GraphicBoard,
    //             piece_graphics: &mut PieceGraphicsManager,
    //             old_pos: Pos,
    //         ) {
    //             let prev_index = old_pos.to_index();
    //             let corner = graphic_board.tile_corners()[prev_index];
    //             let piece_mut = piece_graphics.selected_piece_mut().unwrap();
    //             piece_mut.x = corner.0;
    //             piece_mut.y = corner.1;
    //             piece_graphics.unselect();
    //         }

    //         if inputs.mouse_just_released
    //             && piece_graphics.selected_piece().is_some()
    //             && graphic_board
    //                 .window_to_board_pos(inputs.mouse_pos)
    //                 .is_some()
    //         {
    //             position_highlights.clear();
    //             let new_pos = graphic_board.window_to_board_pos(inputs.mouse_pos).unwrap();
    //             let old_pos = piece_graphics.selected_piece().unwrap().board_pos;
    //             // Shouldn't be possible to have no selected card if there's a selected piece, but checking anyway for good measure
    //             if old_pos != new_pos && card_graphics.selected_card().is_some() {
    //                 // Attempt to make move
    //                 let move_result = game_board.make_move(
    //                     card_graphics.selected_card().unwrap().card(),
    //                     old_pos,
    //                     new_pos,
    //                 );
    //                 if move_result.is_some() {
    //                     // If the move was legal, the move was made, update graphics
    //                     piece_graphics.make_move(&graphic_board, old_pos, new_pos);
    //                     piece_graphics.unselect();
    //                     card_graphics.swap_cards();
    //                     card_graphics.unselect();
    //                     play_tap();
    //                 } else {
    //                     // If the move is illegal, put the piece back
    //                     return_piece(&graphic_board, &mut piece_graphics, old_pos)
    //                 }
    //             } else {
    //                 return_piece(&graphic_board, &mut piece_graphics, old_pos)
    //             }

    //         // Mouse just clicked, pick up piece to move or select card
    //         } else if inputs.mouse_just_pressed {
    //             if let Some(pos) = graphic_board.window_to_board_pos(inputs.mouse_pos) {
    //                 let piece = game_board.squares()[pos.to_index()];
    //                 if piece.is_some_and(|piece| piece.is_red() == game_board.red_to_move())
    //                     && card_graphics.selected_card().is_some()
    //                 {
    //                     piece_graphics.select_at_pos(pos);
    //                     let selected_card = card_graphics.selected_card().unwrap().card();
    //                     let legal_moves = game_board.legal_moves_from_pos(pos);
    //                     let end_positions = legal_moves.iter().filter_map(|mov| {
    //                         (mov.used_card == selected_card).then_some(mov.end_pos)
    //                     });
    //                     position_highlights.extend(end_positions);
    //                 }
    //             } else {
    //                 card_graphics.select_by_click(inputs.mouse_pos, game_board.red_to_move())
    //             }
    //         }

    //         // If piece is held, move it under cursor
    //         if let Some(piece) = piece_graphics.selected_piece_mut() {
    //             piece.x = inputs.mouse_pos.0 - (piece.width / 2) as i32;
    //             piece.y = inputs.mouse_pos.1 - (piece.height / 2) as i32;
    //         }
    //     } else if AI_OPPONENT {
    //         // AI Takes turn
    //         let ai_move = blue_ai.suggest_move(game_board.clone(), false);

    //         game_board.make_move(ai_move.used_card, ai_move.start_pos, ai_move.end_pos);
    //         card_graphics.select_card(ai_move.used_card); // Select card now, swap after animation finishes
    //         piece_graphics.make_move(&graphic_board, ai_move.start_pos, ai_move.end_pos);

    //         // Piece animation
    //         let from_corner = graphic_board.tile_corners()[ai_move.start_pos.to_index()];
    //         let to_corner = graphic_board.tile_corners()[ai_move.end_pos.to_index()];
    //         let mut animator = MoveAnimator::new(from_corner, to_corner);
    //         piece_graphics.select_at_pos(ai_move.end_pos); // Select the piece so it can be referenced by the animator
    //         animator.animate(piece_graphics.selected_piece_mut().unwrap(), 0.001);
    //         move_animator = Some(animator);
    //     }

    //     // Draw screen
    //     graphic_board.draw_board(&mut canvas);
    //     graphic_board.highlight_tiles(&mut canvas, &position_highlights);
    //     piece_graphics.draw(&mut canvas);
    //     card_graphics.draw(&mut canvas, game_board.red_to_move());

    //     canvas.present();
    //     fps_manager.delay_frame();

    //     if !move_animator
    //         .as_ref()
    //         .is_some_and(|animator| animator.animating())
    //     {
    //         match game_board.winner() {
    //             Some(true) => {
    //                 std::thread::sleep(std::time::Duration::from_secs(1));
    //                 println!("Red wins!");
    //                 break;
    //             }
    //             Some(false) => {
    //                 std::thread::sleep(std::time::Duration::from_secs(1));
    //                 println!("Blue wins!");
    //                 break;
    //             }
    //             None => (),
    //         }
    //     }
    // }
}

// struct Inputs {
//     pub mouse_pressed: bool,
//     pub mouse_just_pressed: bool,
//     pub mouse_just_released: bool,
//     pub mouse_pos: (i32, i32),
// }

// struct FPSManager {
//     timer: std::time::Instant,
//     target_duration_per_frame: std::time::Duration,
// }
// impl FPSManager {
//     pub fn new(target_framerate: u64) -> Self {
//         FPSManager {
//             timer: std::time::Instant::now(),
//             target_duration_per_frame: std::time::Duration::from_millis(1000 / target_framerate),
//         }
//     }
//     pub fn delay_frame(&mut self) {
//         let since_last_frame = self.timer.elapsed();
//         self.timer = std::time::Instant::now();
//         let sleep_time = self
//             .target_duration_per_frame
//             .saturating_sub(since_last_frame);
//         std::thread::sleep(sleep_time)
//     }
//     pub fn time_per_frame(&self) -> std::time::Duration {
//         self.target_duration_per_frame
//     }
// }
