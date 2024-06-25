use crate::cards::{self, Card};

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    RedDisciple = 0b00,
    RedSensei = 0b01,
    BlueDisciple = 0b10,
    BlueSensei = 0b11,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pos(pub i8, pub i8);
impl Pos {
    pub fn to_index(self) -> usize {
        (self.0 * 5 + self.1) as usize
    }
    pub fn from_index(idx: usize) -> Self {
        let idx = idx as i8;
        Self(idx / 5, idx % 5)
    }
}

pub struct Board {
    squares: [Option<Piece>; 25],
    red_to_move: bool,
    red_cards: (Card, Card),
    blue_cards: (Card, Card),
    transfer_card: Card
}
impl Board {
    #[rustfmt::skip]
    pub fn new() -> Self {
        use Piece::*;
        let squares = [
            Some(BlueDisciple), Some(BlueDisciple), Some(BlueSensei), Some(BlueDisciple), Some(BlueDisciple),
            None,               None,               None,             None,               None,
            None,               None,               None,             None,               None,
            None,               None,               None,             None,               None,
            Some(RedDisciple),  Some(RedDisciple),  Some(RedSensei),  Some(RedDisciple),  Some(RedDisciple),
        ];
        Board {
            red_to_move: true,
            squares,
            red_cards: (cards::BOAR, cards::BOAR),
            blue_cards: (cards::BOAR, cards::BOAR),
            transfer_card: cards::BOAR
        }
    }
    // Does not check legality of move, just makes it
    // Returns which piece is captured if any
    pub fn make_move(&mut self, from: Pos, to: Pos) -> Option<Piece> {
        self.red_to_move = !self.red_to_move;
        let from_piece = self.squares[from.to_index()];
        let to_piece = self.squares[to.to_index()];

        self.squares[from.to_index()] = None;
        self.squares[to.to_index()] = from_piece;

        to_piece
    }

    /// Undo the previous move
    pub fn undo_move(&mut self) {
        !todo!()
    }

    pub fn squares(&self) -> &[Option<Piece>; 25] {
        &self.squares
    }

    pub fn red_to_move(&self) -> bool {
        self.red_to_move
    }
}
