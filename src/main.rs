use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

mod game;
use game::*;

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

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let board = Board::new();
    let board_graphic = BoardGraphic::new(&canvas);

    // Load piece textures and tint them
    let mut red_disciple_tex = tex_creator.load_texture("assets/disciple.png").unwrap();
    let mut red_sensei_tex = tex_creator.load_texture("assets/sensei.png").unwrap();
    let mut blue_disciple_tex = tex_creator.load_texture("assets/disciple.png").unwrap();
    let mut blue_sensei_tex = tex_creator.load_texture("assets/sensei.png").unwrap();
    red_disciple_tex.set_color_mod(255, 128, 128);
    red_sensei_tex.set_color_mod(255, 128, 128);
    blue_disciple_tex.set_color_mod(128, 128, 255);
    blue_sensei_tex.set_color_mod(128, 128, 255);

    // Create a separate graphics object for each piece
    // Store a reference to each in the board graphic
    let mut piece_graphics = Vec::with_capacity(10);
    for (i, (corner, piece)) in board_graphic
        .tile_corners()
        .iter()
        .zip(board.squares().iter()).enumerate()
    {
        if let Some(piece) = piece {
            let texture = match *piece {
                Piece::RedDisciple => &red_disciple_tex,
                Piece::RedSensei => &red_sensei_tex,
                Piece::BlueDisciple => &blue_disciple_tex,
                Piece::BlueSensei => &blue_sensei_tex,
            };
            let board_pos = Pos::from_index(i);
            let mut new_piece = PieceGraphic::new(texture, board_pos.0, board_pos.1);
            new_piece.x = corner.0;
            new_piece.y = corner.1;
            new_piece.width = board_graphic.tile_width as u32;
            new_piece.height = board_graphic.tile_width as u32;
            piece_graphics.push(new_piece);
        }
    }

    let mut inputs = Inputs {
        mouse_pressed: false,
        mouse_just_pressed: false,
        mouse_just_released: false,
        mouse_pos: (0,0)
    };

    // Start event loop
    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(FRAMERATE).unwrap();
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
                    inputs.mouse_pos = (x,y);
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    inputs.mouse_pressed = false;
                    inputs.mouse_just_released = true;
                },
                Event::MouseMotion { x, y, .. } => {
                    inputs.mouse_pos = (x,y);
                }
                _ => (),
            }
        }

        if inputs.mouse_just_pressed {
            if let Some(pos) = board_graphic.window_to_board_pos(inputs.mouse_pos) {
                println!("{} {}", pos.0, pos.1)
            }
        }

        board_graphic.draw_board(&mut canvas);
        for piece in &piece_graphics {
            piece.draw(&mut canvas);
        }

        canvas.present();
        fps_manager.delay();
    }
}

struct Inputs {
    pub mouse_pressed: bool,
    pub mouse_just_pressed: bool,
    pub mouse_just_released: bool,
    pub mouse_pos: (i32, i32)
}

const LINEWIDTH: u32 = 10; //px
/// Draws board and provides offsets for individual tiles
struct BoardGraphic {
    x: i32,
    y: i32,
    board_width: u32,
    tile_width: u32,
}
impl BoardGraphic {
    pub fn new(canvas: &Canvas<Window>) -> Self {
        let screen_size = canvas.output_size().unwrap();
        let board_width = u32::min(screen_size.0, screen_size.1);
        let tile_width = (board_width - LINEWIDTH * 6) / 5;
        let x = ((screen_size.0 - board_width) / 2) as i32;
        let y = ((screen_size.1 - board_width) / 2) as i32;
        Self {
            x,
            y,
            board_width,
            tile_width,
        }
    }
    pub fn draw_board(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::GRAY);
        canvas
            .fill_rect(Rect::new(
                self.x,
                self.y,
                self.board_width,
                self.board_width,
            ))
            .unwrap();

        canvas.set_draw_color(Color::WHITE);
        for (x, y) in self.tile_corners() {
            canvas
                .fill_rect(Rect::new(
                    x,
                    y,
                    self.tile_width,
                    self.tile_width,
                ))
                .unwrap();
        }
    }
    pub fn tile_corners(&self) -> [(i32, i32); 25] {
        let mut corners = [(0, 0); 25];
        let mut i = 0;
        for row in 0..5 {
            for col in 0..5 {
                let x = LINEWIDTH + self.x as u32 + col * (self.tile_width + LINEWIDTH);
                let y = LINEWIDTH + self.y as u32 + row * (self.tile_width + LINEWIDTH);
                corners[i] = (x as i32, y as i32);
                i += 1;
            }
        }
        corners
    }
    pub fn window_to_board_pos(&self, pos: (i32,i32)) -> Option<Pos> {
        for (i, corner) in self.tile_corners().iter().enumerate() {
            let tile = Rect::new(corner.0, corner.1, self.tile_width, self.tile_width);
            if tile.contains_point(pos) {
                return Some(Pos::from_index(i))
            }
        }
        None
    }
}

/// Tracks and draws an individual piece
struct PieceGraphic<'tex> {
    pub x: i32,
    pub y: i32,
    pub row: u8,
    pub col: u8,
    pub width: u32,
    pub height: u32,
    texture: &'tex Texture<'tex>,
}
impl<'tex> PieceGraphic<'tex> {
    pub fn new(texture: &'tex Texture, row: u8, col: u8) -> Self {
        let width = texture.query().width;
        let height = texture.query().height;
        PieceGraphic {
            x: 0,
            y: 0,
            row,
            col,
            width,
            height,
            texture,
        }
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas
            .copy(
                self.texture,
                None,
                Rect::new(self.x, self.y, self.width, self.height),
            )
            .unwrap();
    }
}
