use crate::cards::{self, Card};

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    RedDisciple = 0b00,
    RedSensei = 0b01,
    BlueDisciple = 0b10,
    BlueSensei = 0b11,
}
impl Piece {
    pub fn is_red(&self) -> bool {
        match self {
            Piece::RedDisciple | Piece::RedSensei => true,
            _ => false,
        }
    }
    pub fn is_blue(&self) -> bool {
        match self {
            Piece::BlueDisciple | Piece::BlueSensei => true,
            _ => false,
        }
    }
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
    pub fn offset(&self, offset: &Pos) -> Self {
        Pos(self.0 + offset.0, self.1 + offset.1)
    }
    pub fn in_bounds(&self) -> bool {
        self.0 >= 0 && self.0 < 5 && self.1 >= 0 && self.1 < 5
    }
}

pub struct GameMove {
    pub start_pos: Pos,
    pub end_pos: Pos,
    pub used_card: Card,
    pub moved_piece: Piece,
    pub captured_piece: Option<Piece>,
}

pub struct Board {
    squares: [Option<Piece>; 25],
    red_to_move: bool,
    red_cards: (Card, Card),
    blue_cards: (Card, Card),
    transfer_card: Card,
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
        let rand_cards = cards::random_cards();
        Board {
            red_to_move: true,
            squares,
            red_cards: (rand_cards[0], rand_cards[1]),
            blue_cards: (rand_cards[2], rand_cards[3]),
            transfer_card: rand_cards[4]
        }
    }
    /// Given a card, start and end position, makes a game move if it is legal and returns it
    pub fn make_move(&mut self, card: Card, start_pos: Pos, end_pos: Pos) -> Option<GameMove> {
        // Is a piece chosen, and does it belong to the current player
        let moved_piece = self.squares[start_pos.to_index()];
        if !moved_piece.is_some_and(|piece| piece.is_red() == self.red_to_move) {
            return None;
        }

        // Do the positions correspond with a move possible by that card
        let offset = Pos(end_pos.0 - start_pos.0, end_pos.1 - start_pos.1);
        let card_offsets = if self.red_to_move {
            card.offsets()
        } else {
            card.rev_offsets()
        };
        if !card_offsets.contains(&offset) {
            return None;
        }

        // Does the used card belong to the current player
        // Since this is the final check, the cards can be swapped immediately
        if self.red_to_move && card == self.red_cards.0 {
            self.red_cards.0 = self.transfer_card
        } else if self.red_to_move && card == self.red_cards.1 {
            self.red_cards.1 = self.transfer_card
        } else if !self.red_to_move && card == self.blue_cards.0 {
            self.blue_cards.0 = self.transfer_card
        } else if !self.red_to_move && card == self.blue_cards.1 {
            self.blue_cards.1 = self.transfer_card
        } else {
            return None;
        }
        self.transfer_card = card;

        // Make move
        let captured_piece = self.squares[end_pos.to_index()];
        self.red_to_move = !self.red_to_move;
        self.squares[start_pos.to_index()] = None;
        self.squares[end_pos.to_index()] = Some(moved_piece.unwrap());

        Some(GameMove {
            start_pos,
            end_pos,
            captured_piece,
            used_card: card,
            moved_piece: moved_piece.unwrap(),
        })
    }

    /// Undo the previous move
    pub fn undo_move(&mut self) {
        !todo!()
    }

    pub fn legal_moves_from_pos(&self, start_pos: Pos) -> Vec<GameMove> {
        let mut legal_moves = Vec::with_capacity(2 * cards::LARGEST_CARD);
        let moved_piece = match self.squares[start_pos.to_index()] {
            Some(piece) => piece,
            None => return legal_moves,
        };

        let mut make_moves = |card: Card| {
            let offsets = if self.red_to_move {
                card.offsets()
            } else {
                card.rev_offsets()
            };
            for offset in offsets {
                let end_pos = start_pos.offset(offset);
                if end_pos.in_bounds() {
                    let captured_piece = self.squares[end_pos.to_index()];
                    if captured_piece.is_none()
                        || captured_piece
                            .is_some_and(|piece| piece.is_red() != moved_piece.is_red())
                    {
                        legal_moves.push(GameMove {
                            start_pos,
                            end_pos,
                            used_card: card,
                            moved_piece,
                            captured_piece,
                        })
                    }
                }
            }
        };

        match (moved_piece.is_red(), self.red_to_move) {
            (true, true) => {
                make_moves(self.red_cards.0);
                make_moves(self.red_cards.1);
            }
            (false, false) => {
                make_moves(self.blue_cards.0);
                make_moves(self.blue_cards.1);
            }
            _ => (),
        }
        legal_moves
    }

    pub fn squares(&self) -> &[Option<Piece>; 25] {
        &self.squares
    }

    pub fn cards(&self) -> [Card; 5] {
        [
            self.red_cards.0,
            self.red_cards.1,
            self.blue_cards.0,
            self.blue_cards.1,
            self.transfer_card,
        ]
    }

    pub fn red_to_move(&self) -> bool {
        self.red_to_move
    }
}
