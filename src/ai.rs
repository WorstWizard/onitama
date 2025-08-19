use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle, time::Duration};

use crate::game::*;
use tinyrand::{Rand, RandRange, Seeded, StdRand};
use tinyrand_std::ClockSeed;

pub struct AsyncAI {
    cancel_signal: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<GameMove>>,
    ai_oppponent: Arc<dyn AIOpponent>,
}
impl AsyncAI {
    pub fn new(ai_oppponent: Arc<dyn AIOpponent>) -> Self {
        Self {
            cancel_signal: Arc::new(AtomicBool::new(false)),
            ai_oppponent,
            thread_handle: None
        }
    }

    /// Called to start a search for a move. If a time limit is specified, the bot is free to search
    /// for an amount of time up to the time limit, otherwise the bot may search until `stop_search` is called
    /// The time limit must still be enforced on the caller end, the parameter is just a hint for the bot (eg time left on a chess clock)
    pub fn start_search(&mut self, board: Board, remaining_time: Option<Duration>) {
        let ai_oppponent = self.ai_oppponent.clone();
        self.cancel_signal.store(false, Ordering::Relaxed);
        let cancel_signal = self.cancel_signal.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            ai_oppponent.search(cancel_signal, board, remaining_time)
        }));
    }

    /// Interrupt an ongoing search and immediately return a gamemove
    /// Panics if the search hasn't been started first with `start_search`
    pub fn stop_search(&mut self) -> GameMove {
        self.cancel_signal.store(true, Ordering::Relaxed); // Signal detached thread to stop
        self.thread_handle.take()
            .expect("Search stopped before it was started")
            .join() // Wait on return value
            .expect("Search thread panicked")
    }

    /// Should return true while searching, false if the search has concluded (eg. if the bot chooses to search for less time than permitted)
    pub fn is_thinking(&self) -> bool {
        !self.cancel_signal.load(Ordering::Relaxed)
    }
}

pub trait AIOpponent : Send + Sync {
    /// Searches for a gamemove, must return early when the cancel signal turns true
    /// If the search finishes early, should set the cancel signal itself, it also serves as a "finished" signal
    fn search(&self, cancel_signal: Arc<AtomicBool>, board: Board, remaining_time: Option<Duration>) -> GameMove;

    #[deprecated]
    fn suggest_move(&self, board: Board) -> GameMove {
        board.legal_moves()[0].clone()
    }
}

#[derive(Default)]
pub struct RandomMover;
impl AIOpponent for RandomMover {
    fn search(&self, cancel_signal: Arc<AtomicBool>, board: Board, _remaining_time: Option<Duration>) -> GameMove {
        cancel_signal.store(true, Ordering::Relaxed);
        self.suggest_move(board)
    }
    fn suggest_move(&self, board: Board) -> GameMove {
        let legal_moves = board.legal_moves();
        let mut rng = StdRand::seed(ClockSeed.next_u64());
        let i = rng.next_range(0..legal_moves.len());
        legal_moves.into_iter().nth(i).unwrap()
    }
}

pub struct MinMaxV0 {
    max_depth: u32,
}
impl AIOpponent for MinMaxV0 {
    fn search(&self, cancel_signal: Arc<AtomicBool>, board: Board, remaining_time: Option<Duration>) -> GameMove {
        todo!()
    }
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
impl MinMaxV0 {
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
        const WIN_SCORE: i32 = i32::MAX / 2;

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