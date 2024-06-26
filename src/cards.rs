use crate::game::Pos;

pub const LARGEST_CARD: usize = 4;

#[derive(Clone, Copy)]
pub struct Card {
    offsets: &'static [Pos],
    rev_offsets: &'static [Pos],
}
impl Card {
    pub fn offsets(&self) -> &[Pos] {
        &self.offsets
    }
    pub fn rev_offsets(&self) -> &[Pos] {
        &self.rev_offsets
    }
}
macro_rules! new_card {
    [$(Pos($row:literal,$col:literal)),+] => {
        Card {
            offsets: &[$(Pos($row,$col)),+],
            rev_offsets: &[$(Pos(-$row, -$col)),+]
        }
    };
}

pub const ALL_CARDS: [&'static Card; 16] = [
    &BOAR,
    &COBRA,
    &CRAB,
    &CRANE,
    &DRAGON,
    &EEL,
    &ELEPHANT,
    &FROG,
    &GOOSE,
    &HORSE,
    &MANTIS,
    &MONKEY,
    &OX,
    &RABBIT,
    &ROOSTER,
    &TIGER
];

/// Should only be used for testing/debugging/initialization
pub const NULL: Card = Card { offsets: &[], rev_offsets: &[] };
pub const BOAR: Card = new_card![
    Pos(-1,0),
    Pos(0,-1),
    Pos(0,1)
];
pub const COBRA: Card = new_card![
    Pos(0,-1),
    Pos(-1,1),
    Pos(1,1)
];
pub const CRAB: Card = new_card![
    Pos(0,-2),
    Pos(-1,0),
    Pos(0,2)
];
pub const CRANE: Card = new_card![
    Pos(1,-1),
    Pos(-1,0),
    Pos(1,1)
];
pub const DRAGON: Card = new_card![
    Pos(-1,-2),
    Pos(1,-1),
    Pos(1,1),
    Pos(-1,2)
];
pub const EEL: Card = new_card![
    Pos(-1,-1),
    Pos(1,-1),
    Pos(0,1)
];
pub const ELEPHANT: Card = new_card![
    Pos(-1,-1),
    Pos(0,-1),
    Pos(-1,1),
    Pos(0,1)
];
pub const FROG: Card = new_card![
    Pos(0,-2),
    Pos(-1,-1),
    Pos(1,1)
];
pub const GOOSE: Card = new_card![
    Pos(-1,-1),
    Pos(0,-1),
    Pos(0,1),
    Pos(1,1)
];
pub const HORSE: Card = new_card![
    Pos(0,-1),
    Pos(-1,0),
    Pos(1,0)
];
pub const MANTIS: Card = new_card![
    Pos(-1,-1),
    Pos(1,0),
    Pos(-1,1)
];
pub const MONKEY: Card = new_card![
    Pos(-1,-1),
    Pos(1,-1),
    Pos(-1,1),
    Pos(1,1)
];
pub const OX: Card = new_card![
    Pos(-1,0),
    Pos(1,0),
    Pos(0,1)
];
pub const RABBIT: Card = new_card![
    Pos(1,-1),
    Pos(-1,1),
    Pos(0,2)
];
pub const ROOSTER: Card = new_card![
    Pos(0,-1),
    Pos(1,-1),
    Pos(-1,1),
    Pos(0,1)
];
pub const TIGER: Card = new_card![
    Pos(-2,0),
    Pos(1,0)
];