use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

const ANIM_TIME: f32 = 0.25;

use crate::cards::Card;
use crate::game::*;

mod colors {
    use sdl2::pixels::Color;
    pub const BOARD_TILE: Color = Color::WHITE;
    pub const BOARD_BG: Color = Color::GRAY;
    pub const BOARD_HIGHLIGHT: Color = Color::YELLOW;
    pub const BOARD_RED_TEMPLE: Color = Color::RGB(200, 50, 50);
    pub const BOARD_BLUE_TEMPLE: Color = Color::RGB(50, 50, 200);
    pub const CARD_BG: Color = Color::RGB(200, 200, 170);
    pub const CARD_TILE_BG: Color = Color::RGB(230, 230, 200);
    pub const CARD_TILE: Color = Color::RGB(130, 130, 100);
    pub const CARD_SELECTED: Color = Color::RGB(250, 250, 220);
    pub const CARD_CENTER: Color = Color::RGB(80, 80, 40);
}

pub struct PieceGraphicsManager<'tex> {
    piece_graphics: [Option<GraphicPiece<'tex>>; 25],
    selected_index: Option<usize>,
}
impl<'tex> PieceGraphicsManager<'tex> {
    pub fn new(
        graphic_board: &GraphicBoard,
        game_board: &Board,
        textures: &'tex PieceTextures<'tex>,
    ) -> Self {
        // Create a separate graphics object for each piece
        const ARR_INIT: Option<GraphicPiece> = None;
        let mut piece_graphics = [ARR_INIT; 25];
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
                new_piece.width = graphic_board.tile_width;
                new_piece.height = graphic_board.tile_width;
                piece_graphics[i] = Some(new_piece);
            }
        }
        PieceGraphicsManager {
            piece_graphics,
            selected_index: None,
        }
    }
    pub fn remove_at_pos(&mut self, pos: Pos) {
        self.piece_graphics[pos.to_index()] = None;
    }
    pub fn select_at_pos(&mut self, pos: Pos) {
        if self.piece_graphics[pos.to_index()].is_some() {
            self.selected_index = Some(pos.to_index())
        } else {
            self.selected_index = None
        }
    }
    pub fn selected_piece(&mut self) -> Option<&GraphicPiece<'tex>> {
        if let Some(i) = self.selected_index {
            return self.piece_graphics[i].as_ref();
        }
        None
    }
    pub fn selected_piece_mut(&mut self) -> Option<&mut GraphicPiece<'tex>> {
        if let Some(i) = self.selected_index {
            return self.piece_graphics[i].as_mut();
        }
        None
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        for piece in self.piece_graphics.iter().flatten() {
            piece.draw(canvas)
        }
        if let Some(i) = self.selected_index {
            self.piece_graphics[i].as_ref().unwrap().draw(canvas)
        }
    }
    /// Moves a piece from one board position to another, deleting a piece if one is already present
    /// Unselects any held piece
    /// Does not check whether the move is legal, or the move is on top of itself
    pub fn make_move(&mut self, graphic_board: &GraphicBoard, from: Pos, to: Pos) {
        self.unselect();
        self.remove_at_pos(to);
        self.piece_graphics[to.to_index()] = self.piece_graphics[from.to_index()].take();
        self.piece_graphics[from.to_index()] = None;
        let to_corner = graphic_board.tile_corners()[to.to_index()];
        let piece_mut = self.piece_graphics[to.to_index()].as_mut().unwrap();
        piece_mut.board_pos = to;
        piece_mut.x = to_corner.0;
        piece_mut.y = to_corner.1;
    }

    pub fn unselect(&mut self) {
        self.selected_index = None;
    }
}

pub struct PieceTextures<'a> {
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

/// Draws board and provides offsets for individual tiles
pub struct GraphicBoard {
    x: i32,
    y: i32,
    board_width: u32,
    tile_width: u32,
    tiles: [Rect; 25],
    tile_corners: [(i32, i32); 25],
}
impl GraphicBoard {
    pub fn new(canvas: &Canvas<Window>) -> Self {
        const LINEWIDTH: u32 = 10; //px

        let screen_size = canvas.output_size().unwrap();
        let board_width = u32::min(screen_size.0, screen_size.1);
        let tile_width = (board_width - LINEWIDTH * 6) / 5;
        // let x = ((screen_size.0 - board_width) / 2) as i32;
        // let y = ((screen_size.1 - board_width) / 2) as i32;
        let x = 0;
        let y = 0;
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
        canvas.set_draw_color(colors::BOARD_BG);
        canvas
            .fill_rect(Rect::new(
                self.x,
                self.y,
                self.board_width,
                self.board_width,
            ))
            .unwrap();

        canvas.set_draw_color(colors::BOARD_TILE);
        for (x, y) in self.tile_corners() {
            canvas
                .fill_rect(Rect::new(*x, *y, self.tile_width, self.tile_width))
                .unwrap();
        }
        // Draw temples
        let w = self.tile_width / 3;
        let h = self.tile_width / 4;
        let red_start_corner = self.tile_corners[22];
        let blue_start_corner = self.tile_corners[2];
        canvas.set_draw_color(colors::BOARD_RED_TEMPLE);
        canvas
            .fill_rect(Rect::new(
                red_start_corner.0 + w as i32,
                red_start_corner.1 + 3 * h as i32,
                w,
                h,
            ))
            .unwrap();
        canvas.set_draw_color(colors::BOARD_BLUE_TEMPLE);
        canvas
            .fill_rect(Rect::new(
                blue_start_corner.0 + w as i32,
                blue_start_corner.1,
                w,
                h,
            ))
            .unwrap();
    }
    pub fn highlight_tiles(&self, canvas: &mut Canvas<Window>, positions: &[Pos]) {
        for pos in positions {
            let (x, y) = self.tile_corners[pos.to_index()];
            canvas.set_draw_color(colors::BOARD_HIGHLIGHT);
            canvas
                .fill_rect(Rect::new(x, y, self.tile_width, self.tile_width))
                .unwrap();
        }
    }
    pub fn tile_corners(&self) -> &[(i32, i32); 25] {
        &self.tile_corners
    }
    pub fn board_width(&self) -> u32 {
        self.board_width
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
#[derive(Clone)]
pub struct GraphicPiece<'tex> {
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

#[derive(Clone)]
pub struct GraphicCard {
    game_card: Card,
    pub rect: Rect,
}
impl GraphicCard {
    const WIDTH: u32 = 200;
    const HEIGHT: u32 = 200;

    pub fn new(game_card: Card, rect: Rect) -> Self {
        Self { game_card, rect }
    }
    pub fn card(&self) -> Card {
        self.game_card
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>, upwards: bool, selected: bool) {
        const LINEWIDTH: i32 = 5;

        if selected {
            canvas.set_draw_color(colors::CARD_SELECTED);
        } else {
            canvas.set_draw_color(colors::CARD_BG);
        }
        canvas.fill_rect(self.rect).unwrap();

        let (x, y) = (self.rect.x(), self.rect.y());
        let sub_rect_w = (self.rect.width() - 6 * LINEWIDTH as u32) / 5;
        let sub_rect_h = (self.rect.height() - 6 * LINEWIDTH as u32) / 5;
        // let sub_rect_w = self.rect.width() / 5;
        // let sub_rect_h = self.rect.height() / 5;
        let offsets = if upwards {
            self.game_card.offsets()
        } else {
            self.game_card.rev_offsets()
        };
        canvas.set_draw_color(colors::CARD_TILE_BG);
        for row in 0..5 {
            for col in 0..5 {
                canvas
                    .fill_rect(Rect::new(
                        x + LINEWIDTH + col * (sub_rect_w as i32 + LINEWIDTH),
                        y + LINEWIDTH + row * (sub_rect_h as i32 + LINEWIDTH),
                        sub_rect_w,
                        sub_rect_h,
                    ))
                    .unwrap();
            }
        }
        canvas.set_draw_color(colors::CARD_TILE);
        for pos in offsets {
            canvas
                .fill_rect(Rect::new(
                    x + LINEWIDTH + (pos.1 as i32 + 2) * (sub_rect_w as i32 + LINEWIDTH),
                    y + LINEWIDTH + (pos.0 as i32 + 2) * (sub_rect_h as i32 + LINEWIDTH),
                    sub_rect_w,
                    sub_rect_h,
                ))
                .unwrap();
        }
        canvas.set_draw_color(colors::CARD_CENTER);
        canvas
            .fill_rect(Rect::new(
                x + LINEWIDTH + 2 * (sub_rect_w as i32 + LINEWIDTH),
                y + LINEWIDTH + 2 * (sub_rect_h as i32 + LINEWIDTH),
                sub_rect_w,
                sub_rect_h,
            ))
            .unwrap();
    }
}
pub struct CardGraphicManager {
    pub red_cards: (GraphicCard, GraphicCard),
    pub blue_cards: (GraphicCard, GraphicCard),
    pub transfer_card: GraphicCard,
    selected_card: Option<GraphicCard>,
}
impl CardGraphicManager {
    pub fn new(game_board: &Board, container_rect: Rect) -> Self {
        let cards = game_board.cards();
        let card_w = (container_rect.width() / 2).min(GraphicCard::WIDTH) as i32;
        let card_h = (container_rect.height() / 3).min(GraphicCard::HEIGHT) as i32;
        let x = container_rect.x();
        let y = container_rect.y();
        let w = container_rect.width() as i32;
        let h = container_rect.height() as i32;
        let red_card_0 = GraphicCard::new(
            cards[0],
            Rect::new(x, y + h - card_h, card_w as u32, card_h as u32),
        );
        let red_card_1 = GraphicCard::new(
            cards[1],
            Rect::new(x + w - card_w, y + h - card_h, card_w as u32, card_h as u32),
        );
        let blue_card_0 = GraphicCard::new(cards[2], Rect::new(x, y, card_w as u32, card_h as u32));
        let blue_card_1 = GraphicCard::new(
            cards[3],
            Rect::new(x + w - card_w, y, card_w as u32, card_h as u32),
        );
        let transfer_card = GraphicCard::new(
            cards[4],
            Rect::new(
                x + (w - card_w) / 2,
                y + (h - card_h) / 2,
                card_w as u32,
                card_h as u32,
            ),
        );
        Self {
            red_cards: (red_card_0, red_card_1),
            blue_cards: (blue_card_0, blue_card_1),
            transfer_card,
            selected_card: None,
        }
    }
    pub fn select_by_click(&mut self, clicked_pos: (i32, i32), red_to_move: bool) {
        if red_to_move {
            if self.red_cards.0.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.red_cards.0.clone())
            } else if self.red_cards.1.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.red_cards.1.clone())
            }
        } else if self.blue_cards.0.rect.contains_point(clicked_pos) {
            self.selected_card = Some(self.blue_cards.0.clone())
        } else if self.blue_cards.1.rect.contains_point(clicked_pos) {
            self.selected_card = Some(self.blue_cards.1.clone())
        }
    }
    pub fn select_card(&mut self, card: Card) {
        let mut idx = None;
        for (i, graphic_card) in self.cards().into_iter().enumerate() {
            if graphic_card.game_card == card {
                idx = Some(i)
            }
        }
        if let Some(i) = idx {
            self.selected_card = Some(self.cards()[i].clone())
        }
    }
    fn cards(&self) -> [&GraphicCard; 5] {
        [
            &self.red_cards.0,
            &self.red_cards.1,
            &self.blue_cards.0,
            &self.blue_cards.1,
            &self.transfer_card,
        ]
    }
    pub fn unselect(&mut self) {
        self.selected_card = None
    }
    pub fn selected_card(&self) -> Option<&GraphicCard> {
        self.selected_card.as_ref()
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>, red_to_move: bool) {
        self.red_cards.0.draw(canvas, true, false);
        self.red_cards.1.draw(canvas, true, false);
        self.blue_cards.0.draw(canvas, false, false);
        self.blue_cards.1.draw(canvas, false, false);
        self.transfer_card.draw(canvas, red_to_move, false);
        if let Some(card) = &self.selected_card {
            card.draw(canvas, red_to_move, true);
        }
    }
    /// Swaps selected card with transfer card
    /// Assumes a card is selected, panics otherwise
    pub fn swap_cards(&mut self) {
        let selected_card = self.selected_card().unwrap();
        if selected_card.game_card == self.red_cards.0.game_card {
            std::mem::swap(
                &mut self.red_cards.0.game_card,
                &mut self.transfer_card.game_card,
            );
        } else if selected_card.game_card == self.red_cards.1.game_card {
            std::mem::swap(
                &mut self.red_cards.1.game_card,
                &mut self.transfer_card.game_card,
            );
        } else if selected_card.game_card == self.blue_cards.0.game_card {
            std::mem::swap(
                &mut self.blue_cards.0.game_card,
                &mut self.transfer_card.game_card,
            );
        } else if selected_card.game_card == self.blue_cards.1.game_card {
            std::mem::swap(
                &mut self.blue_cards.1.game_card,
                &mut self.transfer_card.game_card,
            );
        }
        self.selected_card = None;
    }
}

pub struct MoveAnimator {
    animating: bool,
    time: f32,
    start_point: (i32, i32),
    end_point: (i32, i32),
}
impl MoveAnimator {
    pub fn new(start_point: (i32, i32), end_point: (i32, i32)) -> Self {
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
    /// Animates the piece, returns true if the animation is over
    pub fn animate(&mut self, piece: &mut GraphicPiece, delta_time: f32) -> bool {
        self.time += delta_time;
        if self.time >= ANIM_TIME {
            self.time = ANIM_TIME;
            self.animating = false
        }
        fn lerp(a: i32, b: i32, t: f32) -> i32 {
            (a as f32 * (1.0 - t) + b as f32 * t) as i32
        }
        piece.x = lerp(self.start_point.0, self.end_point.0, self.time / ANIM_TIME);
        piece.y = lerp(self.start_point.1, self.end_point.1, self.time / ANIM_TIME);
        !self.animating
    }
}
