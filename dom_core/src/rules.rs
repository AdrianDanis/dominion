use card::Card;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Players {
    Two = 2,
    Three = 3,
    Four = 4,
}

pub struct Rules {
    pub players: Players,
    pub set: [Card; 10],
}
