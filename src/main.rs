use std::error::Error;
use std::fs::File;
use std::time::Instant;

use glam::{Vec2, vec2};
use onitama::ai::{AIOpponent, RandomMover};
use onitama::game::{Board, GameMove};
use onitama::graphics::{GFXState, Rect};
use onitama::gui::GameGraphics;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source, source::Buffered};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
const ANIM_TIME: f32 = 0.25;

#[derive(Clone)]
pub struct Inputs {
    pub mouse_pressed: bool,
    pub mouse_pos: Vec2,
}

struct Application<'a> {
    gfx_state: Option<GFXState<'a>>,
    game: Option<OnitamaGame>,
    inputs: Inputs,
    timer: Instant,
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

        let mut gfx_state = pollster::block_on(GFXState::new(window));
        let disciple_tex = gfx_state.load_texture("assets/disciple.png");
        let sensei_tex = gfx_state.load_texture("assets/sensei.png");
        
        let game_board = Board::random_cards();
        let game_graphics = GameGraphics::new(
            Rect::new(Vec2::ZERO, vec2(WIDTH as f32, HEIGHT as f32)),
            &game_board,
            disciple_tex,
            sensei_tex,
        );
        let game = OnitamaGame::new(game_graphics, game_board);

        self.gfx_state = Some(gfx_state);
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
                self.redraw_window();
            }
            winit::event::WindowEvent::MouseInput {
                button,
                device_id: _,
                state,
            } => {
                self.inputs.mouse_pressed = button == winit::event::MouseButton::Left
                    && state == winit::event::ElementState::Pressed;
                self.redraw_window();
            }
            winit::event::WindowEvent::RedrawRequested => {
                // Run update
                let delta_time = self.timer.elapsed();
                let needs_update = self
                    .game
                    .as_mut()
                    .unwrap()
                    .update(delta_time.as_secs_f32(), self.inputs.clone());
                self.game_end(event_loop);
                if needs_update {
                    self.redraw_window()
                }

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
                    Err(wgpu::SurfaceError::Other) => {
                        log::error!("Some surface error");
                        event_loop.exit();
                    }
                }
                self.timer = Instant::now();
            }
            _ => (),
        }
    }
}
impl Application<'_> {
    fn redraw_window(&self) {
        self.gfx_state.as_ref().unwrap().window.request_redraw();
    }
    fn game_end(&mut self, event_loop: &ActiveEventLoop) {
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
        timer: Instant::now(),
    };

    event_loop.run_app(&mut app).unwrap();
}

// Main game
pub struct OnitamaGame {
    pub graphics: GameGraphics,
    pub board: Board,
    audio_player: Option<AudioPlayer>,
    ai_opponent: RandomMover,
    last_ai_move: Option<GameMove>,
    animator: Option<MoveAnimator>,
}
impl OnitamaGame {
    pub fn new(graphics: GameGraphics, board: Board) -> Self {
        let audio_player = AudioPlayer::new().ok();
        OnitamaGame {
            graphics,
            board,
            audio_player,
            ai_opponent: RandomMover::default(),
            last_ai_move: None,
            animator: None,
        }
    }
    /// Updates game, outputs true if another update loop should be run
    pub fn update(&mut self, delta_time: f32, inputs: Inputs) -> bool {
        // Player is red, AI is blue
        if self.board.red_to_move() {
            self.handle_mouse_input(inputs.mouse_pressed, inputs.mouse_pos)
        } else if let Some(animator) = &mut self.animator
            && animator.animating()
        {
            // Currently animating a move
            let graphic_piece = self.graphics.pieces.selected_piece_mut().unwrap();
            let finished = animator.animate(&mut graphic_piece.rect.origin, delta_time);
            if finished {
                self.graphics.pieces.make_move(
                    &self.graphics.board,
                    self.last_ai_move.as_ref().unwrap().start_pos,
                    self.last_ai_move.as_ref().unwrap().end_pos,
                );
                self.graphics.cards.swap_cards();
                self.board
                    .make_move_unchecked(self.last_ai_move.take().unwrap());
                if let Some(player) = &mut self.audio_player {
                    player.play_sound()
                }
            }
            true
        } else {
            let ai_move = self.ai_opponent.suggest_move(self.board.clone());
            self.last_ai_move = Some(ai_move.clone());

            // Start animation
            self.graphics
                .pieces
                .select_by_index(ai_move.start_pos.to_index());
            self.graphics.cards.select_card(ai_move.used_card);
            let start_pos = self.graphics.board.tile_corners()[ai_move.start_pos.to_index()];
            let end_pos = self.graphics.board.tile_corners()[ai_move.end_pos.to_index()];
            self.animator = Some(MoveAnimator::new(start_pos, end_pos));
            true
        }
    }
    /// Handles player input, returns true if another update loop should be run
    pub fn handle_mouse_input(&mut self, pressed: bool, mouse_pos: Vec2) -> bool {
        // If a piece is held
        if let Some(piece) = self.graphics.pieces.selected_piece_mut() {
            // Piece released
            if !pressed {
                self.graphics.board.highlight_tiles(&[]);

                // Try to make move
                if let Some(to_pos) = self.graphics.board.window_to_board_pos(mouse_pos)
                    && let Some(card) = self.graphics.cards.selected_card()
                {
                    let from_pos = piece.board_pos;
                    if self
                        .board
                        .make_move(card.card(), from_pos, to_pos)
                        .is_some()
                    {
                        // Move was legal
                        self.graphics
                            .pieces
                            .make_move(&self.graphics.board, from_pos, to_pos);
                        self.graphics.cards.swap_cards();
                        if let Some(audio_player) = self.audio_player.as_mut() {
                            audio_player.play_sound();
                        }
                    }
                }
                self.graphics.pieces.unselect();
            } else {
                // Piece is held
                piece.rect.origin = mouse_pos - piece.rect.size * 0.5;
            }
            return true;
        } else if pressed {
            // Piece clicked
            self.graphics
                .pieces
                .select_by_click(mouse_pos, self.board.red_to_move());
            // Card clicked
            self.graphics
                .cards
                .select_by_click(mouse_pos, self.board.red_to_move());
            // Update highlights
            if let Some(piece) = self.graphics.pieces.selected_piece()
                && let Some(graphic_card) = self.graphics.cards.selected_card()
            {
                let start_pos = piece.board_pos;
                let highlights: Vec<_> = self
                    .board
                    .legal_moves_from_pos(start_pos)
                    .iter()
                    .filter_map(|game_move| {
                        if game_move.used_card == graphic_card.card() {
                            Some(game_move.end_pos)
                        } else {
                            None
                        }
                    })
                    .collect();
                self.graphics.board.highlight_tiles(&highlights);
            }
            return true;
        }
        false
    }
    pub fn winner(&self) -> Option<bool> {
        self.board.winner()
    }
}

struct AudioPlayer {
    _out_stream: OutputStream,
    sink: Sink,
    tap_sound: Buffered<Decoder<File>>,
}
impl AudioPlayer {
    fn new() -> Result<Self, Box<dyn Error>> {
        let out_stream = OutputStreamBuilder::open_default_stream()?;
        let tap_file = File::open("assets/tap_sound.wav")?;
        let tap_sound = Decoder::new(tap_file)?.buffered();
        let sink = Sink::connect_new(out_stream.mixer());

        Ok(AudioPlayer {
            _out_stream: out_stream,
            sink,
            tap_sound,
        })
    }
    fn play_sound(&mut self) {
        self.sink.append(self.tap_sound.clone());
    }
}

pub struct MoveAnimator {
    animating: bool,
    time: f32,
    start_point: Vec2,
    end_point: Vec2,
}
impl MoveAnimator {
    pub fn new(start_point: Vec2, end_point: Vec2) -> Self {
        Self {
            animating: true,
            time: 0.0,
            start_point,
            end_point,
        }
    }
    pub fn animating(&self) -> bool {
        self.animating
    }
    /// Animates the position, returns true when animation is over
    pub fn animate(&mut self, position: &mut Vec2, delta_time: f32) -> bool {
        self.time += delta_time;
        if self.time >= ANIM_TIME {
            self.time = ANIM_TIME;
            self.animating = false
        }
        let new_pos = self.start_point.lerp(self.end_point, self.time / ANIM_TIME);
        *position = new_pos;
        !self.animating
    }
}
