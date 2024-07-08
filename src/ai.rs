use crate::game::*;
use tinyrand::{Rand, RandRange, StdRand, Seeded};
use tinyrand_std::ClockSeed;

pub trait AIOpponent {
    fn suggest_move(board: Board, red_to_move: bool) -> GameMove;
}

pub struct RandomMover {}
impl AIOpponent for RandomMover {
    fn suggest_move(board: Board, red_to_move: bool) -> GameMove {
        let legal_moves = get_legal_moves(&board, red_to_move);
        // for game_move in &legal_moves {
        //     println!("{:?}", game_move.end_pos);
        // }
        let mut rng = StdRand::seed(ClockSeed::default().next_u64());
        let i = rng.next_range(0..legal_moves.len());
        legal_moves.into_iter().nth(i).unwrap()
    }
}

const MAX_DEPTH: u32 = 2;
const WIN_SCORE: i32 = i32::MAX/2;
pub struct MinMax {}
impl AIOpponent for MinMax {
    fn suggest_move(mut board: Board, red_to_move: bool) -> GameMove {
        let legal_moves = get_legal_moves(&board, red_to_move);
        let mut best_move = (legal_moves[0].clone(), i32::MIN);
        for candidate_move in legal_moves {
            board.make_move_unchecked(candidate_move.clone());
            let eval = -Self::minmax(&mut board, !red_to_move, 1);
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
    fn minmax(board: &mut Board, red_to_move: bool, depth: u32) -> i32 {
        if depth == MAX_DEPTH || board.winner().is_some() {
            return Self::board_eval(board, red_to_move);
        }
        let legal_moves = get_legal_moves(&board, red_to_move);
        let mut best_eval = i32::MIN;
        for candidate_move in legal_moves {
            board.make_move_unchecked(candidate_move.clone());
            let eval = -Self::minmax(board, !red_to_move, depth + 1);
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
        for square in board.squares() {
            if let Some(piece) = square {
                if piece.is_red() == red_to_move {
                    piece_val_sum += 100;
                } else {
                    piece_val_sum -= 100;
                }
            }
        }
        piece_val_sum
    }
}

fn get_legal_moves(board: &Board, red_to_move: bool) -> Vec<GameMove> {
    let my_piece_positions = if red_to_move {
        board.red_positions()
    } else {
        board.blue_positions()
    };
    my_piece_positions
        .iter()
        .flat_map(|pos| board.legal_moves_from_pos(*pos))
        .collect()
}