use onitama::{cards, game::{Board, GameStatus}};

#[test]
fn save_default() {
    let board = Board::default();
    let saved_string = board.save_game(false);
    assert_eq!(saved_string, "BCQKD")
}
#[test]
fn load_default() {
    let default_board = Board::default();
    let loaded_board = Board::load_game("BCQKD").unwrap();
    assert_eq!(default_board, loaded_board);
}
#[test]

#[rustfmt::skip]
fn load_example() {
    use onitama::game::Piece::*;
    let loaded_board = Board::load_game(
        "
        # Example spec, specifies an initial board position and
        # three moves of game history

        # Non-standard start, the senseis begin one step forward
        # Board positions in comments for reference
        11.11  #  abcde
        ..3..  #  fghij
        .....  #  klmno
        ..2..  #  pqrst
        00.00  #  uvwxy

        # The five cards in use
        BXLUT

        # Moves
        Brs # red sensei moves right using boar
        Lhl # blue sensei moves down and left using elephant
        Tvl # red disciple captures blue sensei using tiger, game over
    ",
    )
    .unwrap();
    assert_eq!(
        *loaded_board.squares(),
        [
            Some(BlueDisciple), Some(BlueDisciple), None, Some(BlueDisciple), Some(BlueDisciple),
            None,               None,               None, None,               None,
            None,               Some(RedDisciple),  None, None,               None,
            None,               None,               None, Some(RedSensei),    None,
            Some(RedDisciple),  None,               None, Some(RedDisciple),  Some(RedDisciple),
        ]
    );
    assert!(loaded_board.cards().contains(&cards::BOAR));
    assert!(loaded_board.cards().contains(&cards::MONKEY));
    assert!(loaded_board.cards().contains(&cards::ELEPHANT));
    assert!(loaded_board.cards().contains(&cards::ROOSTER));
    assert!(loaded_board.cards().contains(&cards::TIGER));
    assert_eq!(loaded_board.status(), GameStatus::RedWon);
}
mod before_matches_after {
    use onitama::game::{Board, GameMove};
    use tinyrand::{RandRange, StdRand};

    #[test]
    fn random_ten_initial() {
        for _ in 0..10 {
            let rand_board = Board::random_cards();
            let saved_string = rand_board.save_game(false);
            let loaded_board = Board::load_game(&saved_string).unwrap();
            assert_eq!(rand_board, loaded_board);
        }
    }
    #[test]
    fn random_ten_initial_whitespaced() {
        for _ in 0..10 {
            let rand_board = Board::random_cards();
            let saved_string = rand_board.save_game(true);
            let loaded_board = Board::load_game(&saved_string).unwrap();
            assert_eq!(rand_board, loaded_board);
        }
    }
    #[test]
    fn default_ten_random_moves() {
        let mut board = Board::default();
        let mut rng = StdRand::default();
        for _ in 0..10 {
            let positions = if board.red_to_move() {
                board.red_positions()
            } else {
                board.blue_positions()
            };
            let legal_moves: Vec<GameMove> = positions
                .into_iter()
                .flat_map(|pos| board.legal_moves_from_pos(pos))
                .collect();
            let i = rng.next_range(0..legal_moves.len());
            let rand_move = legal_moves[i].clone();
            board.make_move_unchecked(rand_move);
        }
        let saved_string = board.save_game(false);
        let loaded_board = Board::load_game(&saved_string).unwrap();
        assert_eq!(board, loaded_board);
    }
}
