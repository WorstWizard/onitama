use glam::vec2;

use crate::{
    game::Board,
    graphics::{
        board::GraphicBoard,
        card::CardGraphicManager,
        piece::PieceGraphicsManager,
        renderer::{SimpleRenderer, TexHandle},
        Rect,
    },
};

const PREF_RATIO: f32 = 12.0 / 8.0;
pub struct GameGraphics {
    board: GraphicBoard,
    cards: CardGraphicManager,
    pieces: PieceGraphicsManager,
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
