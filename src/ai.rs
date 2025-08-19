use crate::game::*;
use tinyrand::{Rand, RandRange, Seeded, StdRand};
use tinyrand_std::ClockSeed;

pub trait AIOpponent {
    // fn stop_search(&mut self) -> GameMove;
    // fn is_thinking(&self) -> bool;
    fn suggest_move(&self, board: Board) -> GameMove {
        board.legal_moves()[0].clone()
    }
}

pub struct RandomMover {}
impl AIOpponent for RandomMover {
    fn suggest_move(&self, board: Board) -> GameMove {
        let legal_moves = board.legal_moves();
        let mut rng = StdRand::seed(ClockSeed.next_u64());
        let i = rng.next_range(0..legal_moves.len());
        legal_moves.into_iter().nth(i).unwrap()
    }
}

const WIN_SCORE: i32 = i32::MAX / 2;
pub struct MinMax {
    max_depth: u32,
}
impl AIOpponent for MinMax {
    fn suggest_move(&self, mut board: Board) -> GameMove {
        let red_to_move = board.red_to_move();
        let legal_moves = board.legal_moves();
        let mut best_move = (legal_moves[0].clone(), i32::MIN);
        for candidate_move in legal_moves {
            board.make_move_unchecked(candidate_move.clone());
            let eval = -self.minmax(&mut board, !red_to_move, 1);
            if eval > best_move.1 {
                best_move = (candidate_move, eval)
            }
            board.undo_move();
        }
        // println!("AI moves {:?} to {:?}, evaluation {}", best_move.0.start_pos, best_move.0.end_pos, best_move.1);
        best_move.0
    }
}
impl MinMax {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
    fn minmax(&self, board: &mut Board, red_to_move: bool, depth: u32) -> i32 {
        if depth == self.max_depth || board.winner().is_some() {
            return Self::board_eval(board, red_to_move);
        }
        let mut best_eval = i32::MIN;
        for candidate_move in board.legal_moves() {
            board.make_move_unchecked(candidate_move.clone());
            let eval = -self.minmax(board, !red_to_move, depth + 1);
            best_eval = best_eval.max(eval);
            board.undo_move();
        }
        best_eval
    }
    fn board_eval(board: &Board, red_to_move: bool) -> i32 {
        if let Some(red_won) = board.winner() {
            if red_won == red_to_move {
                return WIN_SCORE;
            } else {
                return -WIN_SCORE;
            }
        }
        let mut piece_val_sum = 0;
        for piece in board.squares().iter().flatten() {
            if piece.is_red() == red_to_move {
                piece_val_sum += 100;
            } else {
                piece_val_sum -= 100;
            }
        }
        piece_val_sum
    }
}