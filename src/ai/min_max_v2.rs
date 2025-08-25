use super::*;

// static mut TERMINAL_NODES: u32 = 0;
const WIN_SCORE: i32 = 9999;
const UPPER_LIM: i32 = i32::MAX / 2;
const LOWER_LIM: i32 = i32::MIN / 2;

pub struct MinMaxV2 {
    max_depth: u32,
}
impl AIOpponent for MinMaxV2 {
    fn search(
        &self,
        cancel_signal: Arc<AtomicBool>,
        mut board: Board,
        _remaining_time: Option<Duration>,
    ) -> GameMove {
        // unsafe { TERMINAL_NODES = 0 };
        let red_to_move = board.red_to_move();
        
        let mut candidate_moves: Vec<(GameMove, i32)> = board.legal_moves()
            .into_iter()
            .map(|m| (m, i32::MIN))
            .collect();

        let mut best_move = candidate_moves[0].clone();
        'outer: for d in 1..self.max_depth {
            candidate_moves.sort_by_key(|(_, e)| std::cmp::Reverse(*e));
            for (game_move, eval) in &mut candidate_moves {
                board.make_move_unchecked(game_move.clone());
                *eval = -alphabeta(&cancel_signal, &mut board, !red_to_move, d, LOWER_LIM, UPPER_LIM);
                board.undo_move();
                if cancel_signal.load(Ordering::Relaxed) {
                    // println!("V2: Depth {}, eval {}, nodes touched {}", d, best_move.1, unsafe {TERMINAL_NODES});
                    break 'outer
                }

                if *eval > best_move.1 {
                    best_move = (game_move.clone(), *eval)
                }
            }
            // println!("V2: Depth {}, eval {}, nodes touched {}", d, best_move.1, unsafe {TERMINAL_NODES});
            best_move.1 = LOWER_LIM; // Reset evaluation before next iteration
            // unsafe { TERMINAL_NODES = 0 };
        }
        cancel_signal.store(true, Ordering::Relaxed);


        best_move.0
    }
}
impl Default for MinMaxV2 {
    fn default() -> Self {
        Self { max_depth: 20 }
    }
}
impl MinMaxV2 {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
}

fn alphabeta(
    cancel_signal: &Arc<AtomicBool>,
    board: &mut Board,
    red_to_move: bool,
    depth: u32,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if depth == 0 || board.finished() {
        // unsafe { TERMINAL_NODES += 1 };
        return evaluation(board, red_to_move);
    }
    let mut best_eval = LOWER_LIM;
    let mut candidate_moves = board.legal_moves();
    reorder_moves(&mut candidate_moves);
    for candidate_move in candidate_moves {
        board.make_move_unchecked(candidate_move);
        let eval = -alphabeta(cancel_signal, board, !red_to_move, depth - 1, -beta, -alpha);
        board.undo_move();
        best_eval = best_eval.max(eval);

        // Explicit check for a win to avoid doing more work than necessary
        if eval >= WIN_SCORE { return eval }

        // If search is cancelled, leave immediately, assume this move is bad since we can't guarantee the quality
        // Have to do the check *after* the minmax call, to avoid the zero leaking into the real evaluation
        if cancel_signal.load(Ordering::Relaxed) {
            return 0;
        }

        // Alpha-beta cutoff
        alpha = alpha.max(best_eval);
        if alpha >= beta { break }
    }
    best_eval
}

fn evaluation(board: &Board, red_to_move: bool) -> i32 {
    match board.status() {
        // Evaluation only occurs right *after* a winning move (red_to_move has been flipped),
        // so no matter who won, we should return the negative of the win score
        GameStatus::RedWon | GameStatus::BlueWon => {
            return -WIN_SCORE;
        }
        // Stalemates are even, regardless of material difference
        // Winning positions will tend to avoid it, losing positions will tend to seek it?
        GameStatus::Stalemate => {
            return 0;
        }
        GameStatus::Playing => (),
    }
    let mut piece_val_sum = 0;
    let mut piece_placement_sum = 0;
    for (piece, pos) in board.pieces() {
        let place_value = PIECE_SQUARE_TABLE[pos.to_index()];
        if piece.is_red() == red_to_move {
            piece_val_sum += 100;
            piece_placement_sum += place_value;
        } else {
            piece_val_sum -= 100;
            piece_placement_sum -= place_value;
        }
    }
    piece_val_sum + piece_placement_sum
}

// Guessed good values
const PIECE_SQUARE_TABLE: [i32; 25] = [
    0, 0, 0, 0, 0,
    1, 5, 5, 5, 1,
    2, 5, 10, 5, 2,
    1, 5, 5, 5, 1,
    0, 0, 0, 0, 0,
];

use std::cmp;
fn reorder_moves(candidate_moves: &mut [GameMove]) {
    // Rust sorts in ascending order, so better moves should be *less* than worse moves
    candidate_moves.sort_by(|move_a, move_b| {
        match (move_a.captured_piece, move_b.captured_piece) {
            (Some(_), None) => cmp::Ordering::Less,
            (None, Some(_)) => cmp::Ordering::Greater,
            _ => cmp::Ordering::Equal
        }
    });
}