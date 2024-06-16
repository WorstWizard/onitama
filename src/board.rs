use crate::game::*;

pub struct Board {
    squares: [Option<Piece>; 25],
    red_to_move: bool
}
impl Board {
    #[rustfmt::skip]
    pub fn new() -> Self {
        use Piece::*;
        let squares = [
            Some(BlueDisciple), Some(BlueDisciple), Some(BlueSensei), Some(BlueDisciple), Some(BlueDisciple),
            None,               None,               None,             None,               None,
            None,               None,               None,             None,               None,
            None,               None,               None,             None,               None,
            Some(RedDisciple),  Some(RedDisciple),  Some(RedSensei),  Some(RedDisciple),  Some(RedDisciple),
        ];
        Board { red_to_move: true, squares }
    }
    // Does not check legality of move, just makes it
    // Returns which piece is captured if any
    pub fn make_move(&mut self, from: (u8,u8), to: (u8,u8)) -> Option<Piece> {
        self.red_to_move = !self.red_to_move;
        let from_idx = (from.0*5 + from.1) as usize;
        let to_idx = (from.0*5 + from.1) as usize;
        let from_piece = self.squares[]
    }
}
