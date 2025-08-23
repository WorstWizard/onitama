use super::*;

static mut TERMINAL_NODES: u32 = 0;

pub struct MinMaxV0 {
    max_depth: u32,
}
impl AIOpponent for MinMaxV0 {
    fn search(
        &self,
        cancel_signal: Arc<AtomicBool>,
        mut board: Board,
        _remaining_time: Option<Duration>,
    ) -> GameMove {
        unsafe { TERMINAL_NODES = 0 };
        let red_to_move = board.red_to_move();
        let legal_moves = board.legal_moves();
        let mut best_move = (legal_moves[0].clone(), i32::MIN);
        for candidate_move in legal_moves {
            board.make_move_unchecked(candidate_move.clone());
            let eval = -self.minmax(&cancel_signal, &mut board, !red_to_move, 1);
            if eval > best_move.1 && !cancel_signal.load(Ordering::Relaxed) {
                best_move = (candidate_move, eval)
            }
            board.undo_move();
        }
        cancel_signal.store(true, Ordering::Relaxed);

        println!("V0: End nodes touched: {}, eval {}", unsafe {TERMINAL_NODES}, best_move.1);
    
        best_move.0
    }
}
impl Default for MinMaxV0 {
    fn default() -> Self {
        Self { max_depth: 5 }
    }
}
impl MinMaxV0 {
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }
    fn minmax(
        &self,
        cancel_signal: &Arc<AtomicBool>,
        board: &mut Board,
        red_to_move: bool,
        depth: u32,
    ) -> i32 {
        if depth == self.max_depth || board.finished() {
            unsafe { TERMINAL_NODES += 1 };
            return Self::board_eval(board, red_to_move);
        }
        let mut best_eval = i32::MIN;
        for candidate_move in board.legal_moves() {
            board.make_move_unchecked(candidate_move);
            let eval = -self.minmax(cancel_signal, board, !red_to_move, depth + 1);
            board.undo_move();
            best_eval = best_eval.max(eval);

            // If search is cancelled, leave immediately, assume this move is bad since we can't guarantee the quality
            // Have to do the check *after* the minmax call, to avoid the zero leaking into the real evaluation
            if cancel_signal.load(Ordering::Relaxed) {
                return 0;
            }
        }
        best_eval
    }
    fn board_eval(board: &Board, red_to_move: bool) -> i32 {
        const WIN_SCORE: i32 = i32::MAX / 2;

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