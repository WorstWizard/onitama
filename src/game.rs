// use std::hash::{Hash, Hasher};

use crate::cards::{self, Card};

#[derive(Clone, Copy, Debug, Hash)]
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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

#[derive(Clone, Hash)]
pub struct GameMove {
    pub start_pos: Pos,
    pub end_pos: Pos,
    pub used_card: Card,
    pub transferred_card: Card,
    pub moved_piece: Piece,
    pub captured_piece: Option<Piece>,
}

#[derive(Clone, Hash)]
pub struct Board {
    squares: [Option<Piece>; 25],
    red_to_move: bool,
    red_cards: (Card, Card),
    blue_cards: (Card, Card),
    transfer_card: Card,
    winner: Option<bool>, // true if red, false if blue, None if neither
    move_history: Vec<GameMove>
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
            transfer_card: rand_cards[4],
            winner: None,
            move_history: Vec::with_capacity(20)
        }
    }
    /// Given a card, start and end position, makes a game move if it is legal and returns it
    pub fn make_move(&mut self, card: Card, start_pos: Pos, end_pos: Pos) -> Option<GameMove> {
        // Is a piece chosen, and does it belong to the current player
        let moved_piece = self.squares[start_pos.to_index()];
        if !moved_piece.is_some_and(|piece| piece.is_red() == self.red_to_move) {
            return None;
        }
        // Is a piece captured, and does it belong to the current player
        let captured_piece = self.squares[end_pos.to_index()];
        if captured_piece.is_some_and(|piece| piece.is_red() == moved_piece.unwrap().is_red()) {
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
        let belongs_to_player = (self.red_to_move && (card == self.red_cards.0 || card == self.red_cards.1)) || (!self.red_to_move && (card == self.blue_cards.0 || card == self.blue_cards.1));
        if !belongs_to_player { return None }

        let game_move = GameMove {
            start_pos,
            end_pos,
            captured_piece,
            used_card: card,
            transferred_card: self.transfer_card,
            moved_piece: moved_piece.unwrap(),
        };

        self.make_move_unchecked(game_move.clone());
        Some(game_move)
    }

    /// Takes a `Gamemove` and performs it, ignoring legality
    pub fn make_move_unchecked(&mut self, game_move: GameMove) {
        let captured_piece = self.squares[game_move.end_pos.to_index()];
        match captured_piece {
            Some(Piece::RedSensei) => self.winner = Some(false),
            Some(Piece::BlueSensei) => self.winner = Some(true),
            _ => (),
        }
        match (game_move.moved_piece, game_move.end_pos) {
            (Piece::RedSensei, Pos(0, 2)) => self.winner = Some(true),
            (Piece::BlueSensei, Pos(4, 2)) => self.winner = Some(false),
            _ => (),
        }
        if self.red_cards.0 == game_move.used_card {
            self.red_cards.0 = self.transfer_card
        } else if self.red_cards.1 == game_move.used_card {
            self.red_cards.1 = self.transfer_card
        } else if self.blue_cards.0 == game_move.used_card {
            self.blue_cards.0 = self.transfer_card
        } else if self.blue_cards.1 == game_move.used_card {
            self.blue_cards.1 = self.transfer_card
        }
        self.transfer_card = game_move.used_card;
        self.red_to_move = !self.red_to_move;
        self.squares[game_move.start_pos.to_index()] = None;
        self.squares[game_move.end_pos.to_index()] = Some(game_move.moved_piece);
        self.move_history.push(game_move);
    }

    pub fn winner(&self) -> Option<bool> {
        self.winner
    }

    pub fn red_positions(&self) -> Vec<Pos> {
        self.color_positions(true)
    }
    pub fn blue_positions(&self) -> Vec<Pos> {
        self.color_positions(false)
    }
    fn color_positions(&self, red_pieces: bool) -> Vec<Pos> {
        self.squares
            .into_iter()
            .enumerate()
            .filter_map(|(i, opt)| {
                opt.is_some_and(|piece| piece.is_red() == red_pieces)
                    .then_some(Pos::from_index(i))
            })
            .collect()
    }

    /// Undo the previous move
    pub fn undo_move(&mut self) {
        // let mut hasher = std::hash::DefaultHasher::default();
        // self.hash(&mut hasher);
        // println!("hash of board before undo {}", hasher.finish());
        let last_move = self.move_history.pop().unwrap();
        self.winner = None;
        self.red_to_move = !self.red_to_move;
        self.squares[last_move.start_pos.to_index()] = Some(last_move.moved_piece);
        self.squares[last_move.end_pos.to_index()] = last_move.captured_piece;
        self.transfer_card = last_move.transferred_card;
        if self.red_cards.0 == last_move.transferred_card {
            self.red_cards.0 = last_move.used_card;
        } else if self.red_cards.1 == last_move.transferred_card {
            self.red_cards.1 = last_move.used_card;
        } else if self.blue_cards.0 == last_move.transferred_card {
            self.blue_cards.0 = last_move.used_card;
        } else if self.blue_cards.1 == last_move.transferred_card {
            self.blue_cards.1 = last_move.used_card;
        }
        // let mut hasher = std::hash::DefaultHasher::default();
        // self.hash(&mut hasher);
        // println!("hash of board after undo {}", hasher.finish());
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
                            transferred_card: self.transfer_card,
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
