use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::game::*;
use strum::EnumIter;
use tinyrand::{Rand, RandRange, Seeded, StdRand};
use tinyrand_std::ClockSeed;

// All the bots
mod min_max_v0;
pub use min_max_v0::MinMaxV0;
mod min_max_v1;
pub use min_max_v1::MinMaxV1;
mod min_max_v2;
pub use min_max_v2::MinMaxV2;
mod min_max_v3;
pub use min_max_v3::MinMaxV3;

#[derive(Clone, Copy, EnumIter, strum::Display, PartialEq)]
pub enum AIVersion {
    Dummy,
    Random,
    MinMaxV0,
    MinMaxV1,
    MinMaxV2,
    MinMaxV3,
}
impl AIVersion {
    pub fn make_ai(&self) -> AsyncAI {
        let ai_opponent: Arc<dyn AIOpponent> = match self {
            Self::Dummy => Arc::new(Dummy),
            Self::Random => Arc::new(RandomMover),
            Self::MinMaxV0 => Arc::new(MinMaxV0::default()),
            Self::MinMaxV1 => Arc::new(MinMaxV1::default()),
            Self::MinMaxV2 => Arc::new(MinMaxV2::default()),
            Self::MinMaxV3 => Arc::new(MinMaxV3::default()),
        };
        AsyncAI::new(ai_opponent)
    }
}


/// Wrapper struct for the individual bots, provides synchronous interface for asynchronous searching
/// Use `start_search` to begin looking for a good move, then `stop_search` to recover the found move
/// `start_search` is non-blocking, `stop_search` blocks until a move is given by the bot.
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
            thread_handle: None,
        }
    }

    /// Called to start a search for a move. If a time limit is specified, the bot is free to search
    /// for an amount of time up to the time limit, otherwise the bot may search until `stop_search` is called
    /// The time limit must still be enforced on the caller end using `stop_search`, the parameter is just a hint for the bot (eg time left on a chess clock)
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
        self.thread_handle
            .take()
            .expect("Search stopped before it was started")
            .join() // Wait on return value
            .expect("Search thread panicked")
    }

    /// Should return true while searching, false if the search has concluded (eg. if the bot chooses to search for less time than permitted)
    pub fn is_thinking(&self) -> bool {
        !self.cancel_signal.load(Ordering::Relaxed)
    }
}

pub trait AIOpponent: Send + Sync {
    /// Searches for a gamemove, must return early when the cancel signal turns true
    /// If the search finishes early, should set the cancel signal itself, it also serves as a "finished" signal
    fn search(
        &self,
        cancel_signal: Arc<AtomicBool>,
        board: Board,
        remaining_time: Option<Duration>,
    ) -> GameMove;
}

#[derive(Default)]
pub struct RandomMover;
impl AIOpponent for RandomMover {
    fn search(
        &self,
        cancel_signal: Arc<AtomicBool>,
        board: Board,
        _remaining_time: Option<Duration>,
    ) -> GameMove {
        cancel_signal.store(true, Ordering::Relaxed);
        let legal_moves = board.legal_moves();
        let mut rng = StdRand::seed(ClockSeed.next_u64());
        let i = rng.next_range(0..legal_moves.len());
        legal_moves.into_iter().nth(i).unwrap()
    }
}

#[derive(Default)]
pub struct Dummy;
impl AIOpponent for Dummy {
    fn search(
        &self,
        cancel_signal: Arc<AtomicBool>,
        board: Board,
        _remaining_time: Option<Duration>,
    ) -> GameMove {
        cancel_signal.store(true, Ordering::Relaxed);
        let legal_moves = board.legal_moves();
        legal_moves.first().unwrap().clone()
    }
}