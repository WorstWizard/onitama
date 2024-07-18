use ai::AIOpponent;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer::{self, LoaderRWops};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;

mod game;
use game::*;
mod graphics;
use graphics::*;
use sdl2::rect::Rect;
mod ai;
mod cards;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
const FRAMERATE: u64 = 60;
const AI_OPPONENT: bool = true;
const AI_VERSUS_AI: bool = false;
const RED_AI_MAX_DEPTH: u32 = 4;
const BLUE_AI_MAX_DEPTH: u32 = 5;

fn main() {
    // Set up SDL, window, most graphics
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();

    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        1024,
    )
    .unwrap();
    let audio_loader = sdl2::rwops::RWops::from_file("assets/tap_sound.wav", "r").unwrap();
    let tap_sound = audio_loader.load_wav().unwrap();

    let play_tap = || {
        mixer::Channel::all().play(&tap_sound, 0).unwrap();
    };

    let window = video_subsystem
        .window("Onitama", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    // Load textures for the pieces
    let tex_creator = canvas.texture_creator();
    let piece_textures = PieceTextures::init(&tex_creator);

    // Make game board, set up graphics
    let mut game_board = Board::new();
    let graphic_board = GraphicBoard::new(&canvas);
    let mut piece_graphics =
        PieceGraphicsManager::new(&graphic_board, &game_board, &piece_textures);
    let mut position_highlights = Vec::new();
    let mut card_graphics = CardGraphicManager::new(
        &game_board,
        Rect::new(
            graphic_board.board_width() as i32,
            0,
            WIDTH - graphic_board.board_width(),
            HEIGHT,
        ),
    );

    // Animator for sliding pieces
    let mut move_animator: Option<MoveAnimator> = None;

    // AI's
    let red_ai = ai::MinMax::new(RED_AI_MAX_DEPTH);
    let blue_ai = ai::MinMax::new(BLUE_AI_MAX_DEPTH);

    // Inputs
    let mut inputs = Inputs {
        mouse_pressed: false,
        mouse_just_pressed: false,
        mouse_just_released: false,
        mouse_pos: (0, 0),
    };

    // Start event loop
    let mut fps_manager = FPSManager::new(FRAMERATE);
    let mut event_pump = sdl_ctx.event_pump().unwrap();
    'main: loop {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        // Manage inputs
        inputs.mouse_just_pressed = false;
        inputs.mouse_just_released = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    println!("{}", game_board.save_game());
                },
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    inputs.mouse_pressed = true;
                    inputs.mouse_just_pressed = true;
                    inputs.mouse_pos = (x, y);
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    inputs.mouse_pressed = false;
                    inputs.mouse_just_released = true;
                }
                Event::MouseMotion { x, y, .. } => {
                    inputs.mouse_pos = (x, y);
                }
                _ => (),
            }
        }

        if move_animator
            .as_ref()
            .is_some_and(|animator| animator.animating())
        {
            let delta_time = fps_manager.time_per_frame();
            let finished_animation = move_animator.as_mut().unwrap().animate(
                piece_graphics.selected_piece_mut().unwrap(),
                delta_time.as_secs_f32(),
            );
            if finished_animation {
                piece_graphics.unselect();
                card_graphics.swap_cards();
                play_tap();
            }
        } else if !AI_VERSUS_AI && (!AI_OPPONENT || game_board.red_to_move()) {
            fn return_piece(
                graphic_board: &GraphicBoard,
                piece_graphics: &mut PieceGraphicsManager,
                old_pos: Pos,
            ) {
                let prev_index = old_pos.to_index();
                let corner = graphic_board.tile_corners()[prev_index];
                let piece_mut = piece_graphics.selected_piece_mut().unwrap();
                piece_mut.x = corner.0;
                piece_mut.y = corner.1;
                piece_graphics.unselect();
            }

            if inputs.mouse_just_released
                && piece_graphics.selected_piece().is_some()
                && graphic_board
                    .window_to_board_pos(inputs.mouse_pos)
                    .is_some()
            {
                position_highlights.clear();
                let new_pos = graphic_board.window_to_board_pos(inputs.mouse_pos).unwrap();
                let old_pos = piece_graphics.selected_piece().unwrap().board_pos;
                // Shouldn't be possible to have no selected card if there's a selected piece, but checking anyway for good measure
                if old_pos != new_pos && card_graphics.selected_card().is_some() {
                    // Attempt to make move
                    let move_result = game_board.make_move(
                        card_graphics.selected_card().unwrap().card(),
                        old_pos,
                        new_pos,
                    );
                    if move_result.is_some() {
                        // If the move was legal, the move was made, update graphics
                        piece_graphics.make_move(&graphic_board, old_pos, new_pos);
                        piece_graphics.unselect();
                        card_graphics.swap_cards();
                        card_graphics.unselect();
                        play_tap();
                    } else {
                        // If the move is illegal, put the piece back
                        return_piece(&graphic_board, &mut piece_graphics, old_pos)
                    }
                } else {
                    return_piece(&graphic_board, &mut piece_graphics, old_pos)
                }

            // Mouse just clicked, pick up piece to move or select card
            } else if inputs.mouse_just_pressed {
                if let Some(pos) = graphic_board.window_to_board_pos(inputs.mouse_pos) {
                    let piece = game_board.squares()[pos.to_index()];
                    if piece.is_some_and(|piece| piece.is_red() == game_board.red_to_move())
                        && card_graphics.selected_card().is_some()
                    {
                        piece_graphics.select_at_pos(pos);
                        let selected_card = card_graphics.selected_card().unwrap().card();
                        let legal_moves = game_board.legal_moves_from_pos(pos);
                        let end_positions = legal_moves.iter().filter_map(|mov| {
                            (mov.used_card == selected_card).then_some(mov.end_pos)
                        });
                        position_highlights.extend(end_positions);
                    }
                } else {
                    card_graphics.select_by_click(inputs.mouse_pos, game_board.red_to_move())
                }
            }

            // If piece is held, move it under cursor
            if let Some(piece) = piece_graphics.selected_piece_mut() {
                piece.x = inputs.mouse_pos.0 - (piece.width / 2) as i32;
                piece.y = inputs.mouse_pos.1 - (piece.height / 2) as i32;
            }
        } else if AI_OPPONENT || AI_VERSUS_AI {
            // AI Takes turn
            let ai_move = if game_board.red_to_move() {
                red_ai.suggest_move(game_board.clone(), true)
            } else {
                blue_ai.suggest_move(game_board.clone(), false)
            };

            game_board.make_move(ai_move.used_card, ai_move.start_pos, ai_move.end_pos);
            card_graphics.select_card(ai_move.used_card); // Select card now, swap after animation finishes
            piece_graphics.make_move(&graphic_board, ai_move.start_pos, ai_move.end_pos);

            // Piece animation
            let from_corner = graphic_board.tile_corners()[ai_move.start_pos.to_index()];
            let to_corner = graphic_board.tile_corners()[ai_move.end_pos.to_index()];
            let mut animator = MoveAnimator::new(from_corner, to_corner);
            piece_graphics.select_at_pos(ai_move.end_pos); // Select the piece so it can be referenced by the animator
            animator.animate(piece_graphics.selected_piece_mut().unwrap(), 0.001);
            move_animator = Some(animator);
        }

        // Draw screen
        graphic_board.draw_board(&mut canvas);
        graphic_board.highlight_tiles(&mut canvas, &position_highlights);
        piece_graphics.draw(&mut canvas);
        card_graphics.draw(&mut canvas, game_board.red_to_move());

        canvas.present();
        fps_manager.delay_frame();

        if !move_animator
            .as_ref()
            .is_some_and(|animator| animator.animating())
        {
            match game_board.winner() {
                Some(true) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    println!("Red wins!");
                    break;
                }
                Some(false) => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    println!("Blue wins!");
                    break;
                }
                None => (),
            }
        }
    }
}

struct Inputs {
    pub mouse_pressed: bool,
    pub mouse_just_pressed: bool,
    pub mouse_just_released: bool,
    pub mouse_pos: (i32, i32),
}

struct FPSManager {
    timer: std::time::Instant,
    target_duration_per_frame: std::time::Duration,
}
impl FPSManager {
    pub fn new(target_framerate: u64) -> Self {
        FPSManager {
            timer: std::time::Instant::now(),
            target_duration_per_frame: std::time::Duration::from_millis(1000 / target_framerate),
        }
    }
    pub fn delay_frame(&mut self) {
        let since_last_frame = self.timer.elapsed();
        self.timer = std::time::Instant::now();
        let sleep_time = self
            .target_duration_per_frame
            .saturating_sub(since_last_frame);
        std::thread::sleep(sleep_time)
    }
    pub fn time_per_frame(&self) -> std::time::Duration {
        self.target_duration_per_frame
    }
}
