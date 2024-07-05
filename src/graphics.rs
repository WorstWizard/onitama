use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::cards::Card;
use crate::game::*;

mod Colors {
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
                // Very important: If the selected piece is the last one, the index should be swapped too
                if self
                    .selected_piece
                    .is_some_and(|selected_i| selected_i == self.piece_graphics.len() - 1)
                {
                    self.selected_piece = Some(i)
                }
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
        if let Some(idx) = self.selected_piece {
            self.piece_graphics[idx].draw(canvas)
        }
    }
    /// Moves currently selected piece from one board position to another, deleting a piece if one is already present
    /// Does not check whether the move is legal, or the move is on top of itself
    pub fn make_move(&mut self, graphic_board: &GraphicBoard, to: Pos) {
        self.remove_at_pos(to);
        let piece_mut = self.selected_piece_mut().unwrap();
        piece_mut.board_pos = to;
        let corner = graphic_board.tile_corners()[to.to_index()];
        piece_mut.x = corner.0;
        piece_mut.y = corner.1;
    }

    pub fn unselect(&mut self) {
        self.selected_piece = None;
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
        canvas.set_draw_color(Colors::BOARD_BG);
        canvas
            .fill_rect(Rect::new(
                self.x,
                self.y,
                self.board_width,
                self.board_width,
            ))
            .unwrap();

        canvas.set_draw_color(Colors::BOARD_TILE);
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
        canvas.set_draw_color(Colors::BOARD_RED_TEMPLE);
        canvas.fill_rect(Rect::new(red_start_corner.0 + w as i32, red_start_corner.1 + 3*h as i32, w, h)).unwrap();
        canvas.set_draw_color(Colors::BOARD_BLUE_TEMPLE);
        canvas.fill_rect(Rect::new(blue_start_corner.0 + w as i32, blue_start_corner.1, w, h)).unwrap();
    }
    pub fn highlight_tiles(&self, canvas: &mut Canvas<Window>, positions: &[Pos]) {
        for pos in positions {
            let (x, y) = self.tile_corners[pos.to_index()];
            canvas.set_draw_color(Colors::BOARD_HIGHLIGHT);
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
            canvas.set_draw_color(Colors::CARD_SELECTED);
        } else {
            canvas.set_draw_color(Colors::CARD_BG);
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
        canvas.set_draw_color(Colors::CARD_TILE_BG);
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
        canvas.set_draw_color(Colors::CARD_TILE);
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
        canvas.set_draw_color(Colors::CARD_CENTER);
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
    pub fn select_card(&mut self, clicked_pos: (i32, i32), red_to_move: bool) {
        if red_to_move {
            if self.red_cards.0.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.red_cards.0.clone())
            } else if self.red_cards.1.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.red_cards.1.clone())
            }
        } else {
            if self.blue_cards.0.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.blue_cards.0.clone())
            } else if self.blue_cards.1.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.blue_cards.1.clone())
            }
        }
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
                &mut self.blue_cards.0.game_card,
                &mut self.transfer_card.game_card,
            );
        }
    }
}
