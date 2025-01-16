use glam::{vec2, Vec2};

use super::board::GraphicBoard;
use super::renderer::*;
use super::colors;
use crate::game::{Pos, Piece, Board};

/// Tracks and draws an individual piece
#[derive(Clone)]
pub struct GraphicPiece {
    pub origin: Vec2,
    pub size: Vec2,
    pub board_pos: Pos,
    piece: Piece,
    texture: TexHandle,
}
impl GraphicPiece {
    pub fn new(texture: TexHandle, size: Vec2, piece: Piece, board_pos: Pos) -> Self {
        GraphicPiece {
            origin: vec2(0.0, 0.0),
            size,
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
        renderer.draw_textured_rect(
            self.origin,
            self.size.x,
            self.size.y,
            color,
            self.texture
        );
    }
}

pub struct PieceGraphicsManager<'board> {
    graphic_board: &'board GraphicBoard,
    piece_graphics: [Option<GraphicPiece>; 25],
    selected_index: Option<usize>,
}
impl<'board> PieceGraphicsManager<'board> {
    pub fn new(
        graphic_board: &'board GraphicBoard,
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
                new_piece.origin = *corner;
                piece_graphics[i] = Some(new_piece);
            }
        }
        PieceGraphicsManager {
            graphic_board,
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
    pub fn selected_piece(&mut self) -> Option<&GraphicPiece> {
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
        if let Some(i) = self.selected_index {
            self.piece_graphics[i].as_ref().unwrap().draw(renderer)
        }
    }
    /// Moves a piece from one board position to another, deleting a piece if one is already present
    /// Unselects any held piece
    /// Does not check whether the move is legal, or the move is on top of itself
    pub fn make_move(&mut self, from: Pos, to: Pos) {
        self.unselect();
        self.remove_at_pos(to);
        self.piece_graphics[to.to_index()] = self.piece_graphics[from.to_index()].take();
        // self.piece_graphics[from.to_index()] = None;
        let to_corner = self.graphic_board.tile_corners()[to.to_index()];
        let piece_mut = self.piece_graphics[to.to_index()].as_mut().unwrap();
        piece_mut.board_pos = to;
        piece_mut.origin = to_corner;
    }

    pub fn unselect(&mut self) {
        self.selected_index = None;
    }
}