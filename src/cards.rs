use crate::game::Pos;

pub struct Card {
    pub offsets: Vec<Pos>,
}

impl<const N: usize> From<[Pos; N]> for Card {
    fn from(value: [Pos; N]) -> Self {
        Card { offsets: value.to_vec() }
    }
}

pub const BOAR: [Pos; 3] = [
    Pos(-1,0),
    Pos(0,-1),
    Pos(0,1)
];

