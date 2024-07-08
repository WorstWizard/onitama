use crate::game::*;
use tinyrand::{Rand, RandRange, StdRand, Seeded};
use tinyrand_std::ClockSeed;

pub trait AIOpponent {
    fn suggest_move(board: Board, red_to_move: bool) -> GameMove;
}

pub struct RandomMover {}
impl AIOpponent for RandomMover {
    fn suggest_move(board: Board, red_to_move: bool) -> GameMove {
        let my_piece_positions = if red_to_move {
            board.red_positions()
        } else {
            board.blue_positions()
        };
        let legal_moves: Vec<GameMove> = my_piece_positions
            .iter()
            .flat_map(|pos| board.legal_moves_from_pos(*pos))
            .collect();
        // for game_move in &legal_moves {
        //     println!("{:?}", game_move.end_pos);
        // }
        let mut rng = StdRand::seed(ClockSeed::default().next_u64());
        let i = rng.next_range(0..legal_moves.len());
        legal_moves.into_iter().nth(i).unwrap()
    }
}