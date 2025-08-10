use glam::{Vec2, vec2};

use super::Rect;
use super::board::GraphicBoard;
use super::colors;
use super::renderer::*;
use crate::game::{Board, Piece, Pos};

/// Tracks and draws an individual piece
#[derive(Clone)]
pub struct GraphicPiece {
    pub rect: Rect,
    pub board_pos: Pos,
    piece: Piece,
    texture: TexHandle,
}
impl GraphicPiece {
    pub fn new(texture: TexHandle, size: Vec2, piece: Piece, board_pos: Pos) -> Self {
        GraphicPiece {
            rect: Rect::new(Vec2::ZERO, size),
            board_pos,
            piece,
            texture,
        }
    }
    pub fn draw(&self, renderer: &mut SimpleRenderer) {
        let color = if self.piece.is_red() {
            colors::PIECE_RED
        } else {
            colors::PIECE_BLUE
        };
        renderer.draw_textured_rect(self.rect, color, self.texture);
    }
}

pub struct PieceGraphicsManager {
    piece_graphics: [Option<GraphicPiece>; 25],
    selected_index: Option<usize>,
    selected_original_pos: Vec2,
}
impl PieceGraphicsManager {
    pub fn new(
        graphic_board: &GraphicBoard,
        game_board: &Board,
        disciple_tex: TexHandle,
        sensei_tex: TexHandle,
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
                    Piece::RedDisciple | Piece::BlueDisciple => disciple_tex,
                    Piece::RedSensei | Piece::BlueSensei => sensei_tex,
                };
                let board_pos = Pos::from_index(i);
                let size = vec2(graphic_board.tile_width(), graphic_board.tile_width());
                let mut new_piece = GraphicPiece::new(texture, size, *piece, board_pos);
                new_piece.rect.origin = *corner;
                piece_graphics[i] = Some(new_piece);
            }
        }
        PieceGraphicsManager {
            piece_graphics,
            selected_index: None,
            selected_original_pos: Vec2::ZERO
        }
    }
    pub fn remove_at_pos(&mut self, pos: Pos) {
        self.piece_graphics[pos.to_index()] = None;
    }
    pub fn select_at_pos(&mut self, pos: Pos) {
        if let Some(piece) = &self.piece_graphics[pos.to_index()] {
            self.selected_index = Some(pos.to_index());
            self.selected_original_pos = piece.rect.origin;
        } else {
            self.selected_index = None
        }
    }
    pub fn selected_piece(&self) -> Option<&GraphicPiece> {
        if let Some(i) = self.selected_index {
            return self.piece_graphics[i].as_ref();
        }
        None
    }
    pub fn selected_piece_mut(&mut self) -> Option<&mut GraphicPiece> {
        if let Some(i) = self.selected_index {
            return self.piece_graphics[i].as_mut();
        }
        None
    }
    pub fn draw(&self, renderer: &mut SimpleRenderer) {
        for piece in self.piece_graphics.iter().flatten() {
            piece.draw(renderer)
        }
        // Draw again to ensure the selected piece is on top
        if let Some(piece) = self.selected_piece() {
            piece.draw(renderer)
        }
    }
    /// Moves a piece from one board position to another, deleting a piece if one is already present
    /// Unselects any held piece
    /// Does not check whether the move is legal, or the move is on top of itself
    pub fn make_move(&mut self, graphic_board: &GraphicBoard, from: Pos, to: Pos) {
        self.unselect();
        self.remove_at_pos(to);
        self.piece_graphics[to.to_index()] = self.piece_graphics[from.to_index()].take();
        // self.piece_graphics[from.to_index()] = None;
        let to_corner = graphic_board.tile_corners()[to.to_index()];
        let piece_mut = self.piece_graphics[to.to_index()].as_mut().unwrap();
        piece_mut.board_pos = to;
        piece_mut.rect.origin = to_corner;
    }

    pub fn unselect(&mut self) {
        let original_pos = self.selected_original_pos;
        if let Some(piece) = self.selected_piece_mut() {
            piece.rect.origin = original_pos;
        }
        self.selected_index = None;
    }
}
