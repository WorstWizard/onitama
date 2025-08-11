use glam::{Vec2, vec2};

use crate::{
    game::Board,
    graphics::{
        Rect,
        board::GraphicBoard,
        card::CardGraphicManager,
        piece::PieceGraphicsManager,
        renderer::{SimpleRenderer, TexHandle},
    },
};

// Shared/static game UI
const PREF_RATIO: f32 = 12.0 / 8.0;
pub struct GameGraphics {
    pub board: GraphicBoard,
    pub cards: CardGraphicManager,
    pub pieces: PieceGraphicsManager,
}
impl GameGraphics {
    pub fn new(
        rect: Rect,
        game_board: &Board,
        disciple_tex: TexHandle,
        sensei_tex: TexHandle,
    ) -> Self {
        let ratio = rect.size.x / rect.size.y;
        let actual_rect = if ratio >= PREF_RATIO {
            Rect::new(rect.origin, vec2(PREF_RATIO * rect.size.y, rect.size.y))
        } else {
            Rect::new(rect.origin, vec2(rect.size.x, rect.size.x / PREF_RATIO))
        };
        let board_rect = Rect::new(rect.origin, vec2(actual_rect.size.y, actual_rect.size.y));
        let board = GraphicBoard::new(board_rect);
        let card_rect = Rect::new(
            vec2(rect.origin.x + board_rect.size.x, rect.origin.y),
            vec2(actual_rect.size.x - board_rect.size.x, actual_rect.size.y),
        );
        let cards = CardGraphicManager::new(game_board, card_rect);
        let pieces = PieceGraphicsManager::new(&board, game_board, disciple_tex, sensei_tex);
        GameGraphics {
            board,
            cards,
            pieces,
        }
    }
    pub fn draw(&self, renderer: &mut SimpleRenderer, red_to_move: bool) {
        self.board.draw_board(renderer);
        self.cards.draw(renderer, red_to_move);
        self.pieces.draw(renderer);
    }
}

// GUI for the main game
pub struct OnitamaGame {
    pub graphics: GameGraphics,
    pub board: Board,
}
impl OnitamaGame {
    pub fn new(graphics: GameGraphics, board: Board) -> Self {
        OnitamaGame { graphics, board }
    }
    pub fn handle_mouse_input(&mut self, pressed: bool, pos: Vec2) {
        // If a piece is held
        if let Some(piece) = self.graphics.pieces.selected_piece_mut() {
            // Piece released
            if !pressed {
                self.graphics.pieces.unselect();
            } else {
                piece.rect.origin = pos - piece.rect.size * 0.5;
            }
        } else if pressed {
            // Piece clicked
            self.graphics
                .pieces
                .select_by_click(pos, self.board.red_to_move());
            // Card clicked
            self.graphics
                .cards
                .select_by_click(pos, self.board.red_to_move());
        }
    }
}
