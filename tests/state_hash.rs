use onitama::game::Board;

#[test]
fn hash_default() {
    let board = Board::default();
    let hash = board.state_hash();
    println!("hash: {hash:#066b}")
}
#[test]
fn hash_different() {
    let mut board = Board::default();
    let hash_1 = board.state_hash();
    let game_move = board.legal_moves()[0].clone();
    board.make_move_unchecked(game_move);
    let hash_2 = board.state_hash();
    assert_ne!(hash_1, hash_2);
}
#[test]
fn hash_same_state() {
    let mut board = Board::default();
    let hash_1 = board.state_hash();
    let game_move = board.legal_moves()[0].clone();
    board.make_move_unchecked(game_move);
    board.undo_move();
    let hash_2 = board.state_hash();
    assert_eq!(hash_1, hash_2);
}
