use crate::game::Pos;
use tinyrand::{Rand, RandRange, Seeded, StdRand};
use tinyrand_std::ClockSeed;

pub const LARGEST_CARD: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Card {
    offsets: &'static [Pos],
    rev_offsets: &'static [Pos],
    name: &'static str,
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
    [$name:ident $(Pos($row:literal,$col:literal)),+] => {
        Card {
            offsets: &[$(Pos($row,$col)),+],
            rev_offsets: &[$(Pos(-$row, -$col)),+],
            name: stringify!($name)
        }
    };
}

pub fn random_cards() -> [Card; 5] {
    let seed = ClockSeed::default().next_u64();
    let mut rand = StdRand::seed(seed);
    let mut indices = Vec::with_capacity(5);
    while indices.len() < 5 {
        let next_i = rand.next_range(0..16);
        if !indices.contains(&next_i) {
            indices.push(next_i);
        }
    }
    [
        ALL_CARDS[indices[0]],
        ALL_CARDS[indices[1]],
        ALL_CARDS[indices[2]],
        ALL_CARDS[indices[3]],
        ALL_CARDS[indices[4]],
    ]
}

pub const ALL_CARDS: [Card; 16] = [
    BOAR, COBRA, CRAB, CRANE, DRAGON, EEL, ELEPHANT, FROG, GOOSE, HORSE, MANTIS, MONKEY, OX,
    RABBIT, ROOSTER, TIGER,
];

/// Should only be used for testing/debugging/initialization
// pub const NULL: Card = Card { offsets: &[], rev_offsets: &[],  };
pub const BOAR: Card = new_card![BOAR
    Pos(-1,0),
    Pos(0,-1),
    Pos(0,1)
];
pub const COBRA: Card = new_card![COBRA
    Pos(0,-1),
    Pos(-1,1),
    Pos(1,1)
];
pub const CRAB: Card = new_card![CRAB
    Pos(0,-2),
    Pos(-1,0),
    Pos(0,2)
];
pub const CRANE: Card = new_card![CRANE
    Pos(1,-1),
    Pos(-1,0),
    Pos(1,1)
];
pub const DRAGON: Card = new_card![DRAGON
    Pos(-1,-2),
    Pos(1,-1),
    Pos(1,1),
    Pos(-1,2)
];
pub const EEL: Card = new_card![EEL
    Pos(-1,-1),
    Pos(1,-1),
    Pos(0,1)
];
pub const ELEPHANT: Card = new_card![ELEPHANT
    Pos(-1,-1),
    Pos(0,-1),
    Pos(-1,1),
    Pos(0,1)
];
pub const FROG: Card = new_card![FROG
    Pos(0,-2),
    Pos(-1,-1),
    Pos(1,1)
];
pub const GOOSE: Card = new_card![GOOSE
    Pos(-1,-1),
    Pos(0,-1),
    Pos(0,1),
    Pos(1,1)
];
pub const HORSE: Card = new_card![HORSE
    Pos(0,-1),
    Pos(-1,0),
    Pos(1,0)
];
pub const MANTIS: Card = new_card![MANTIS
    Pos(-1,-1),
    Pos(1,0),
    Pos(-1,1)
];
pub const MONKEY: Card = new_card![MONKEY
    Pos(-1,-1),
    Pos(1,-1),
    Pos(-1,1),
    Pos(1,1)
];
pub const OX: Card = new_card![OX
    Pos(-1,0),
    Pos(1,0),
    Pos(0,1)
];
pub const RABBIT: Card = new_card![RABBIT
    Pos(1,-1),
    Pos(-1,1),
    Pos(0,2)
];
pub const ROOSTER: Card = new_card![ROOSTER
    Pos(0,-1),
    Pos(1,-1),
    Pos(-1,1),
    Pos(0,1)
];
pub const TIGER: Card = new_card![TIGER
    Pos(-2,0),
    Pos(1,0)
];
