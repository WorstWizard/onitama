use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use egui::Ui;
use onitama::{
    ai::{self, AIOpponent, AsyncAI, RandomMover},
    game::{Board, GameMove},
    graphics::{GFXState, renderer::TexHandle},
    gui::GameGraphics,
};
use strum::{Display, EnumIter, IntoEnumIterator};
use tinyrand::{RandRange, StdRand};
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
        arena: None,
    };

    event_loop.run_app(&mut app).unwrap();
}

struct Application<'a> {
    egui_renderer: Option<egui_wgpu::Renderer>,
    egui_state: Option<egui_winit::State>,
    gfx_state: Option<GFXState<'a>>,
    arena: Option<Arena>,
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

        self.arena = Some(Arena::new(
            gfx_state.load_texture("assets/disciple.png"),
            gfx_state.load_texture("assets/sensei.png"),
        ));
        self.egui_state = Some(egui_winit::State::new(
            egui_ctx.clone(),
            egui::viewport::ViewportId::ROOT,
            &gfx_state.window,
            None,
            None,
            None,
        ));
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
        let arena = self.arena.as_mut().unwrap();

        let _ = state.on_window_event(&gfx_state.window, &event); // Process event with egui
        // if event_response.consumed { return }

        match event {
            WindowEvent::RedrawRequested => {
                const PPP: f32 = 1.0;

                let raw_input = state.take_egui_input(&gfx_state.window);
                let mut leftover_rect = egui::Rect::ZERO;
                let full_output = state.egui_ctx().run(raw_input, |ctx| {
                    arena.make_ui(ctx);
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

                    // Update & render game
                    arena.update_match();
                    let game_rect = from_egui_rect(leftover_rect);
                    let game_graphics = arena.game_graphics(game_rect);
                    game_graphics.draw(&mut gfx_state.renderer, arena.red_to_move()); // Draw game
                    gfx_state
                        .renderer
                        .render(&gfx_state.queue, &mut render_pass);

                    // Update egui
                    let clipped_prims = state.egui_ctx().tessellate(full_output.shapes, PPP);
                    let screen_descriptor = egui_wgpu::ScreenDescriptor {
                        pixels_per_point: PPP,
                        size_in_pixels: [WIDTH, HEIGHT],
                    };
                    self.egui_renderer.as_mut().unwrap().update_buffers(
                        &gfx_state.device,
                        &gfx_state.queue,
                        &mut encoder,
                        &clipped_prims,
                        &screen_descriptor,
                    );
                    for (id, delta) in full_output.textures_delta.set {
                        self.egui_renderer.as_mut().unwrap().update_texture(
                            &gfx_state.device,
                            &gfx_state.queue,
                            id,
                            &delta,
                        );
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

struct Arena {
    game: Board,
    disciple_tex: TexHandle,
    sensei_tex: TexHandle,
    position_generation: PositionGeneration,
    stored_positions: Vec<String>,
    ai_selection: (AIVersion, AIVersion),
    ai_opps: (AsyncAI, AsyncAI), // red and blue
    ai_playing: bool,
    started_search: bool,
    last_move_time: Instant,
}
impl Arena {
    fn make_ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("AI match");
                ui.add_enabled_ui(!self.ai_playing, |ui| {
                    egui::ComboBox::from_label("Red AI")
                        .selected_text(self.ai_selection.0.to_string())
                        .show_ui(ui, |ui| {
                            for variant in AIVersion::iter() {
                                ui.selectable_value(
                                    &mut self.ai_selection.0,
                                    variant,
                                    variant.to_string(),
                                );
                            }
                        });
                    egui::ComboBox::from_label("Blue AI")
                        .selected_text(self.ai_selection.1.to_string())
                        .show_ui(ui, |ui| {
                            for variant in AIVersion::iter() {
                                ui.selectable_value(
                                    &mut self.ai_selection.1,
                                    variant,
                                    variant.to_string(),
                                );
                            }
                        });
                    if ui.button("Play").clicked() {
                        self.ai_opps.0 = self.ai_selection.0.make_ai();
                        self.ai_opps.1 = self.ai_selection.1.make_ai();
                        self.ai_playing = true;
                    }
                });
                ui.separator();
                self.position_generation
                    .make_ui(ui, &mut self.game, &mut self.stored_positions);
                ui.separator();
                ui.group(|ui| {
                    for (i, game_str) in self.stored_positions.iter().enumerate() {
                        ui.label(i.to_string() + ": " + game_str);
                    }
                })
            });
    }

    fn update_match(&mut self) {
        if !self.ai_playing {
            return;
        }
        let game = &mut self.game;
        let current_ai = if game.red_to_move() {
            &mut self.ai_opps.0
        } else {
            &mut self.ai_opps.1
        };
        const TIME_PER_MOVE: Duration = Duration::from_millis(2000);

        // Start a search for a move
        if !self.started_search {
            self.started_search = true;
            self.last_move_time = Instant::now();
            current_ai.start_search(game.clone(), None);
        // Stop search, get next move
        } else if !current_ai.is_thinking() || self.last_move_time.elapsed() > TIME_PER_MOVE {
            self.started_search = false;
            let game_move = current_ai.stop_search();
            game.make_move(game_move.used_card, game_move.start_pos, game_move.end_pos)
                .expect("Illegal move!");
        }

        if self.game.winner().is_some() {
            self.ai_playing = false
        }
    }

    fn game_graphics(&self, rect: onitama::graphics::Rect) -> GameGraphics {
        GameGraphics::new(rect, &self.game, self.disciple_tex, self.sensei_tex)
    }

    fn red_to_move(&self) -> bool {
        self.game.red_to_move()
    }

    fn new(disciple_tex: TexHandle, sensei_tex: TexHandle) -> Self {
        Self {
            game: Board::random_cards(),
            disciple_tex,
            sensei_tex,
            position_generation: PositionGeneration::new(),
            stored_positions: vec![],
            ai_selection: (AIVersion::Random, AIVersion::Random),
            ai_opps: (
                AsyncAI::new(Arc::new(RandomMover)),
                AsyncAI::new(Arc::new(RandomMover)),
            ),
            ai_playing: false,
            started_search: false,
            last_move_time: Instant::now(),
        }
    }
}

#[derive(Clone, Copy, EnumIter, Display, PartialEq)]
enum AIVersion {
    Random,
    MinMaxV0,
}
impl AIVersion {
    fn make_ai(&self) -> AsyncAI {
        let ai_opponent: Arc<dyn AIOpponent> = match self {
            Self::Random => Arc::new(ai::RandomMover),
            Self::MinMaxV0 => Arc::new(ai::MinMaxV0::new(4)),
        };
        AsyncAI::new(ai_opponent)
    }
}

struct PositionGeneration {
    bulk_number: u32,
    rng: StdRand,
}
impl PositionGeneration {
    fn new() -> Self {
        Self {
            bulk_number: 1,
            rng: StdRand::default(),
        }
    }

    pub fn make_ui(&mut self, ui: &mut Ui, game: &mut Board, board_positions: &mut Vec<String>) {
        ui.label("Starting position suite");
        if ui.button("Random position").clicked() {
            *game = self.generate_random_position();
            board_positions.push(game.save_game(false));
        }
        ui.horizontal(|ui| {
            if ui.button("Bulk generate").clicked() {
                for _ in 0..self.bulk_number {
                    *game = self.generate_random_position();
                    board_positions.push(game.save_game(false));
                }
            }
            ui.add(egui::DragValue::new(&mut self.bulk_number).range(1..=1000));
        });
    }

    fn generate_random_position(&mut self) -> Board {
        const MAX_MOVES: u32 = 10;
        let n = self.rng.next_range(0..MAX_MOVES);
        let mut board;
        loop {
            board = Board::random_cards();
            for _ in 0..n {
                let legal_moves: Vec<GameMove> = board
                    .piece_positions()
                    .into_iter()
                    .flat_map(|pos| board.legal_moves_from_pos(pos))
                    .collect();
                let game_move = legal_moves[self.rng.next_range(0..legal_moves.len())].clone();
                board.make_move_unchecked(game_move);
            }
            if !Self::one_move_from_winning(&mut board) {
                break;
            }
        }
        board
    }

    /// Doesn't modify board despite the mutable borrow
    fn one_move_from_winning(board: &mut Board) -> bool {
        let legal_moves: Vec<GameMove> = board
            .piece_positions()
            .into_iter()
            .flat_map(|pos| board.legal_moves_from_pos(pos))
            .collect();
        for game_move in legal_moves {
            board.make_move_unchecked(game_move);
            let winning = board.winner().is_some();
            board.undo_move();
            if winning {
                return true;
            }
        }
        false
    }
}

fn from_egui_rect(rect: egui::Rect) -> onitama::graphics::Rect {
    let (min_x, min_y) = (rect.left_top().x, rect.left_top().y);
    let (max_x, max_y) = (rect.right_bottom().x, rect.right_bottom().y);
    onitama::graphics::Rect {
        origin: glam::vec2(min_x, min_y),
        size: glam::vec2(max_x - min_x, max_y - min_y),
    }
}
