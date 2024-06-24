use crate::game::Pos;

pub struct Card {
    pub offsets: &'static [Pos]
}
impl Card {
    pub const fn new(offsets: &'static [Pos]) -> Self {
        Card { offsets }
    }
}


pub const BOAR: Card = Card::new(&[
    Pos(-1,0),
    Pos(0,-1),
    Pos(0,1)
]);
pub const RABBIT: Card = Card::new(&[
    Pos(1,-1),
    Pos(-1,1),
    Pos(0,2)
]);