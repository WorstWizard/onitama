use crate::game::Pos;

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

pub const BOAR: Card = new_card![
    Pos(-1,0),
    Pos(0,-1),
    Pos(0,1)
];
pub const RABBIT: Card = new_card![
    Pos(1,-1),
    Pos(-1,1),
    Pos(0,2)
];