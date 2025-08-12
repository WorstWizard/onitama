use std::fs::File;

use glam::Vec2;
use glam::vec2;
use onitama::game::Board;
use onitama::graphics::{Rect, GFXState};
use onitama::gui::GameGraphics;
use rodio::source::Buffered;
use rodio::Decoder;
use rodio::OutputStream;
use rodio::Sink;
use rodio::Source;
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

struct Inputs {
    pub mouse_pressed: bool,
    pub mouse_pos: Vec2,
}

struct Application<'a> {
    gfx_state: Option<GFXState<'a>>,
    game: Option<OnitamaGame>,
    inputs: Inputs
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
                    Err(wgpu::SurfaceError::Other) => {
                        log::error!("Some surface error");
                        event_loop.exit();
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

// Main game
pub struct OnitamaGame {
    pub graphics: GameGraphics,
    pub board: Board,
    audio_player: Option<AudioPlayer>,
}
impl OnitamaGame {
    pub fn new(graphics: GameGraphics, board: Board) -> Self {
        let audio_player = AudioPlayer::new().ok();
        OnitamaGame { graphics, board, audio_player }
    }
    pub fn handle_mouse_input(&mut self, pressed: bool, mouse_pos: Vec2) {
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
                        if let Some(audio_player) = self.audio_player.as_mut() { audio_player.play_sound(); }
                    }
                }
                self.graphics.pieces.unselect();
            } else {
                // Piece is held
                piece.rect.origin = mouse_pos - piece.rect.size * 0.5;
            }
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
        }
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
    fn new() -> Result<Self,Box<dyn std::error::Error>> {
        let out_stream = rodio::OutputStreamBuilder::open_default_stream()?;
        let tap_file = std::fs::File::open("assets/tap_sound.wav")?;
        let tap_sound = rodio::Decoder::new(tap_file)?.buffered();
        let sink = Sink::connect_new(out_stream.mixer());

        Ok(AudioPlayer { _out_stream: out_stream, sink, tap_sound })
    }
    fn play_sound(&mut self) {
        self.sink.append(self.tap_sound.clone());
    }
}