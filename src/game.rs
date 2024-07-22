// use std::hash::{Hash, Hasher};

use std::error::Error;

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
        matches!(self, Piece::RedDisciple | Piece::RedSensei)
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
impl GameMove {
    /// Encodes the gamemove as string notation for saving/loading
    pub fn as_encoded_bytes(&self) -> [u8; 3] {
        fn pos_index_to_alphabet(idx: usize) -> u8 {
            b'a' + idx as u8
        }
        [
            cards::card_identifier(&self.used_card),
            pos_index_to_alphabet(self.start_pos.to_index()),
            pos_index_to_alphabet(self.end_pos.to_index()),
        ]
    }
}

#[derive(Clone, Hash)]
pub struct Board {
    squares: [Option<Piece>; 25],
    red_to_move: bool,
    red_cards: (Card, Card),
    blue_cards: (Card, Card),
    transfer_card: Card,
    winner: Option<bool>, // true if red, false if blue, None if neither
    move_history: Vec<GameMove>,
    default_start: bool,
}
impl Board {
    pub fn new() -> Self {
        let squares = Self::default_squares();
        let rand_cards = cards::random_cards();
        Board {
            red_to_move: true,
            squares,
            red_cards: (rand_cards[0], rand_cards[1]),
            blue_cards: (rand_cards[2], rand_cards[3]),
            transfer_card: rand_cards[4],
            winner: None,
            move_history: Vec::with_capacity(20),
            default_start: true,
        }
    }
    #[rustfmt::skip]
    fn default_squares() -> [Option<Piece>; 25] {
        use Piece::*;
        [
            Some(BlueDisciple), Some(BlueDisciple), Some(BlueSensei), Some(BlueDisciple), Some(BlueDisciple),
            None,               None,               None,             None,               None,
            None,               None,               None,             None,               None,
            None,               None,               None,             None,               None,
            Some(RedDisciple),  Some(RedDisciple),  Some(RedSensei),  Some(RedDisciple),  Some(RedDisciple),
        ]
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
        let belongs_to_player = (self.red_to_move
            && (card == self.red_cards.0 || card == self.red_cards.1))
            || (!self.red_to_move && (card == self.blue_cards.0 || card == self.blue_cards.1));
        if !belongs_to_player {
            return None;
        }

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

    /// Saves board history to a string in .oni format
    pub fn save_game(&self) -> String {
        let mut move_history_bytes = Vec::with_capacity(3 * self.move_history.len());
        for game_move in &self.move_history {
            move_history_bytes.extend(game_move.as_encoded_bytes())
        }
        let move_history_str = String::from_utf8(move_history_bytes).unwrap();

        if self.default_start {
            move_history_str
        } else {
            todo!("save game history from non-default start")
        }
    }

    /// Loads a saved game from .oni format
    pub fn load_game(text: String) -> Result<Self, LoadGameError> {
        // Ignore comment lines, whitespace and characters not of relevance
        let filter_comments =
            String::from_iter(text.lines().map(|line| match line.split_once('#') {
                Some((pre, post)) => pre,
                None => line,
            }));

        fn is_board_spec_byte(byte: u8) -> bool {
            byte.is_ascii_digit() || byte == b'.'
        }
        let filtered_bytes: Vec<u8> = filter_comments
            .bytes()
            .filter(|byte| is_board_spec_byte(*byte))
            .collect();

        // Only the cards definition is strictly required for loading,
        // so if the file is empty after filtering, that's the error to return
        if filtered_bytes.is_empty() {
            return Err(LoadGameError::CardsParse);
        }

        // If the first non-filtered character is a board spec character, try to load a board
        let squares = if !is_board_spec_byte(filtered_bytes[0]) {
            Self::default_squares()
        } else {
            // Load a non-default initial board state
            let board_spec_bytes = filtered_bytes.get(0..25).ok_or(LoadGameError::BoardParse)?;
            if board_spec_bytes
                .iter()
                .find(|&byte| is_board_spec_byte(*byte))
                .is_some()
            {
                return Err(LoadGameError::BoardParse);
            }
            let mut squares = [None; 25];
            for (i, byte) in board_spec_bytes.iter().enumerate() {
                squares[i] = match byte {
                    b'.' => None,
                    b'0' => Some(Piece::RedDisciple),
                    b'1' => Some(Piece::BlueDisciple),
                    b'2' => Some(Piece::RedSensei),
                    b'3' => Some(Piece::BlueSensei),
                    _ => return Err(LoadGameError::BoardParse),
                }
            }
            squares
        };

        let game_board = Board::new();
        Ok(game_board)
    }
}

enum LoadGameError {
    BoardParse,
    CardsParse,
    MoveHistoryParse,
}

impl std::fmt::Display for LoadGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoardParse => write!(f, "board state was expected but failed to parse"),
            Self::CardsParse => write!(f, "failed to parse cards"),
            Self::MoveHistoryParse => write!(f, "failed to parse move history"),
        }
    }
}
