use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;

mod game;
use game::*;
mod graphics;
use graphics::*;
mod cards;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const FRAMERATE: u32 = 60;

fn main() {
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();

    let window = video_subsystem
        .window("Onitama", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let tex_creator = canvas.texture_creator();
    let piece_textures = PieceTextures::init(&tex_creator);

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut game_board = Board::new();
    let graphic_board = GraphicBoard::new(&canvas);

    let mut inputs = Inputs {
        mouse_pressed: false,
        mouse_just_pressed: false,
        mouse_just_released: false,
        mouse_pos: (0, 0),
    };

    let mut piece_graphics =
        PieceGraphicsManager::new(&graphic_board, &game_board, &piece_textures);

    let mut fps_manager = FPSManager::new(60);
    // Start event loop
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

        if inputs.mouse_just_released && piece_graphics.selected_piece().is_some() {
            let new_pos = graphic_board.window_to_board_pos(inputs.mouse_pos);
            let old_pos = piece_graphics.selected_piece().unwrap().board_pos;
            if new_pos.is_some() && old_pos != new_pos.unwrap() {
                if let Some(pos) = new_pos {
                    // If the move is legal, make the move
                    let captured_piece = game_board.make_move(old_pos, pos);
                    if let Some(piece) = captured_piece {
                        println!("Captured a {:?}", piece);
                    }
                    piece_graphics.make_move(&graphic_board, new_pos.unwrap());
                }
            } else {
                // If the move is illegal, put the piece back
                let prev_index = old_pos.to_index();
                let corner = graphic_board.tile_corners()[prev_index];
                let piece_mut = piece_graphics.selected_piece_mut().unwrap();
                piece_mut.x = corner.0;
                piece_mut.y = corner.1;
            }
            piece_graphics.unselect();
        } else if inputs.mouse_just_pressed {
            if let Some(pos) = graphic_board.window_to_board_pos(inputs.mouse_pos) {
                piece_graphics.select_at_pos(pos);
            }
        }

        if let Some(piece) = piece_graphics.selected_piece_mut() {
            piece.x = inputs.mouse_pos.0 - (piece.width / 2) as i32;
            piece.y = inputs.mouse_pos.1 - (piece.height / 2) as i32;
        }

        graphic_board.draw_board(&mut canvas);
        piece_graphics.draw(&mut canvas);

        canvas.present();
        fps_manager.delay_frame();
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
            target_duration_per_frame: std::time::Duration::from_millis(1000/target_framerate)
        }
    }
    pub fn delay_frame(&mut self) {
        let since_last_frame = self.timer.elapsed();
        self.timer = std::time::Instant::now();
        let sleep_time = self.target_duration_per_frame.saturating_sub(since_last_frame);
        std::thread::sleep(sleep_time)
    }
}