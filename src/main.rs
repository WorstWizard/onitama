use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

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

        if inputs.mouse_just_released && piece_graphics.selected_piece.is_some() {
            let new_pos = graphic_board.window_to_board_pos(inputs.mouse_pos);
            let old_pos = piece_graphics.selected_piece().unwrap().board_pos;
            if new_pos.is_some() && old_pos != new_pos.unwrap() {
                if let Some(pos) = new_pos {
                    // If the move is legal, make the move
                    let captured_piece = game_board.make_move(old_pos, pos);
                    if let Some(piece) = captured_piece {
                        println!("Captured a {:?}", piece);
                        piece_graphics.remove_at_pos(pos);
                    }
    
                    // Place piece graphic at new location
                    let new_index = pos.to_index();
                    let corner = graphic_board.tile_corners()[new_index];
    
                    let piece_mut = piece_graphics.selected_piece_mut().unwrap();
                    piece_mut.x = corner.0;
                    piece_mut.y = corner.1;
                    piece_mut.board_pos = pos;
                }
            } else {
                // If the move is illegal, put the piece back
                let prev_index = old_pos.to_index();
                let corner = graphic_board.tile_corners()[prev_index];
                let piece_mut = piece_graphics.selected_piece_mut().unwrap();
                piece_mut.x = corner.0;
                piece_mut.y = corner.1;
            }
            piece_graphics.selected_piece = None;
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
        fps_manager.delay();
    }
}

struct Inputs {
    pub mouse_pressed: bool,
    pub mouse_just_pressed: bool,
    pub mouse_just_released: bool,
    pub mouse_pos: (i32, i32),
}

struct PieceGraphicsManager<'tex> {
    piece_graphics: Vec<GraphicPiece<'tex>>,
    selected_piece: Option<usize>,
}
impl<'tex> PieceGraphicsManager<'tex> {
    pub fn new(
        graphic_board: &GraphicBoard,
        game_board: &Board,
        textures: &'tex PieceTextures<'tex>,
    ) -> Self {
        // Create a separate graphics object for each piece
        let mut piece_graphics = Vec::with_capacity(10);
        for (i, (corner, piece)) in graphic_board
            .tile_corners()
            .iter()
            .zip(game_board.squares().iter())
            .enumerate()
        {
            if let Some(piece) = piece {
                let texture = match *piece {
                    Piece::RedDisciple => &textures.red_disciple,
                    Piece::RedSensei => &textures.red_sensei,
                    Piece::BlueDisciple => &textures.blue_disciple,
                    Piece::BlueSensei => &textures.blue_sensei,
                };
                let board_pos = Pos::from_index(i);
                let mut new_piece = GraphicPiece::new(texture, board_pos);
                new_piece.x = corner.0;
                new_piece.y = corner.1;
                new_piece.width = graphic_board.tile_width as u32;
                new_piece.height = graphic_board.tile_width as u32;
                piece_graphics.push(new_piece);
            }
        }
        PieceGraphicsManager {
            piece_graphics,
            selected_piece: None,
        }
    }
    pub fn remove_at_pos(&mut self, pos: Pos) {
        match self
            .piece_graphics
            .iter()
            .enumerate()
            .find(|(_, piece)| piece.board_pos == pos)
        {
            Some((i, _)) => {
                self.piece_graphics.swap_remove(i);
            }
            None => (),
        }
    }
    pub fn select_at_pos(&mut self, pos: Pos) {
        for (i, piece) in self.piece_graphics.iter().enumerate() {
            if piece.board_pos == pos {
                self.selected_piece = Some(i);
                return;
            }
        }
        self.selected_piece = None;
    }
    pub fn selected_piece(&mut self) -> Option<&GraphicPiece<'tex>> {
        if let Some(i) = self.selected_piece {
            return self.piece_graphics.get(i);
        }
        None
    }
    pub fn selected_piece_mut(&mut self) -> Option<&mut GraphicPiece<'tex>> {
        if let Some(i) = self.selected_piece {
            return self.piece_graphics.get_mut(i);
        }
        None
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        for piece in &self.piece_graphics {
            piece.draw(canvas)
        }
    }
}

struct PieceTextures<'a> {
    red_disciple: Texture<'a>,
    red_sensei: Texture<'a>,
    blue_disciple: Texture<'a>,
    blue_sensei: Texture<'a>,
}
impl<'a> PieceTextures<'a> {
    pub fn init(tex_creator: &'a TextureCreator<WindowContext>) -> Self {
        // Load piece textures and tint them
        let mut red_disciple = tex_creator.load_texture("assets/disciple.png").unwrap();
        let mut red_sensei = tex_creator.load_texture("assets/sensei.png").unwrap();
        let mut blue_disciple = tex_creator.load_texture("assets/disciple.png").unwrap();
        let mut blue_sensei = tex_creator.load_texture("assets/sensei.png").unwrap();
        red_disciple.set_color_mod(255, 128, 128);
        red_sensei.set_color_mod(255, 128, 128);
        blue_disciple.set_color_mod(128, 128, 255);
        blue_sensei.set_color_mod(128, 128, 255);
        PieceTextures {
            red_disciple,
            red_sensei,
            blue_disciple,
            blue_sensei,
        }
    }
}

const LINEWIDTH: u32 = 10; //px
/// Draws board and provides offsets for individual tiles
struct GraphicBoard {
    x: i32,
    y: i32,
    board_width: u32,
    tile_width: u32,
    tiles: [Rect; 25],
    tile_corners: [(i32, i32); 25],
}
impl GraphicBoard {
    pub fn new(canvas: &Canvas<Window>) -> Self {
        let screen_size = canvas.output_size().unwrap();
        let board_width = u32::min(screen_size.0, screen_size.1);
        let tile_width = (board_width - LINEWIDTH * 6) / 5;
        let x = ((screen_size.0 - board_width) / 2) as i32;
        let y = ((screen_size.1 - board_width) / 2) as i32;
        let mut tiles = [Rect::new(0, 0, 0, 0); 25];
        let mut tile_corners = [(0, 0); 25];
        let mut i = 0;
        for row in 0..5 {
            for col in 0..5 {
                let tile_x = (LINEWIDTH + x as u32 + col * (tile_width + LINEWIDTH)) as i32;
                let tile_y = (LINEWIDTH + y as u32 + row * (tile_width + LINEWIDTH)) as i32;
                tiles[i] = Rect::new(tile_x, tile_y, tile_width, tile_width);
                tile_corners[i] = (tile_x, tile_y);
                i += 1;
            }
        }
        Self {
            x,
            y,
            board_width,
            tile_width,
            tiles,
            tile_corners,
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
                .fill_rect(Rect::new(*x, *y, self.tile_width, self.tile_width))
                .unwrap();
        }
    }
    pub fn tiles(&self) -> &[Rect; 25] {
        &self.tiles
    }
    pub fn tile_corners(&self) -> &[(i32, i32); 25] {
        &self.tile_corners
    }
    /// If the given position in window coords is on a tile on the board, returns that position.
    pub fn window_to_board_pos(&self, pos: (i32, i32)) -> Option<Pos> {
        for (i, tile) in self.tiles.iter().enumerate() {
            if tile.contains_point(pos) {
                return Some(Pos::from_index(i));
            }
        }
        None
    }
}

/// Tracks and draws an individual piece
struct GraphicPiece<'tex> {
    pub x: i32,
    pub y: i32,
    pub board_pos: Pos,
    pub width: u32,
    pub height: u32,
    texture: &'tex Texture<'tex>,
}
impl<'tex> GraphicPiece<'tex> {
    pub fn new(texture: &'tex Texture, board_pos: Pos) -> Self {
        let width = texture.query().width;
        let height = texture.query().height;
        GraphicPiece {
            x: 0,
            y: 0,
            board_pos,
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
