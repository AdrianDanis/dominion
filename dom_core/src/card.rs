use rules::Players;
use enum_map;

/// Enumeration of all different cards
#[repr(u32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
pub enum Card {
    // Teasure
    Copper,
    Silver,
    Gold,
    // Victory
    Estate,
    Duchy,
    Province,
    // Curse
    Curse,
    // Action
    Cellar,
    Market,
    Militia,
    Mine,
    Moat,
    Remodel,
    Smithy,
    Village,
    Woodcutter,
    Workshop,
}

impl Card {
    fn player_victories(players: Players) -> u32 {
        if players == Players::Two {
            8
        } else {
            12
        }
    }
    pub fn starting_count(&self, players: Players) -> u32 {
        match *self {
            Card::Copper => 60,
            Card::Silver => 40,
            Card::Gold => 30,
            Card::Estate => players as u32 * 3 + Self::player_victories(players),
            Card::Duchy | Card::Province => Self::player_victories(players),
            Card::Curse => (players as u32 - 1) * 10,
            _ => 10,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CardSet {
    map: enum_map::EnumMap<Card, u32>,
}

pub struct CardSetIterator {
    cards: Vec<(Card, u32)>,
}

impl Iterator for CardSetIterator {
    type Item = Card;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((card, count)) = self.cards.pop() {
            let ret = card;
            if count > 1 {
                self.cards.push((card, count - 1));
            }
            return Some(ret);
        }
        None
    }
}

impl CardSet {
    pub fn empty() -> CardSet {
        CardSet {
            map: enum_map::EnumMap::new(),
        }
    }
    pub fn insert(&mut self, card: Card, count: u32) {
        self.map[card] += count;
    }
    pub fn take(&mut self, card: Card, count: u32) -> bool {
        if self.map[card] < count {
            false
        } else {
            self.map[card] -= count;
            true
        }
    }
    pub fn count(&self, card: Card) -> u32 {
        self.map[card]
    }
    /// Check if there are non zero copies of a card in the set
    pub fn contains(&self, card: Card) -> bool {
        self.count(card) > 0
    }
    pub fn drain(&mut self) -> CardSetIterator {
        let it = self.into_iter();
        *self = Self::empty();
        return it;
    }
    pub fn count_iter(&self) -> enum_map::Iter<Card, u32> {
        self.map.iter()
    }
}

impl IntoIterator for CardSet {
    type Item = Card;
    type IntoIter = CardSetIterator;

    fn into_iter(self) -> CardSetIterator {
        let mut cards = Vec::new();
        for (card, value) in self.map {
            if value > 0 {
                cards.push((card, value));
            }
        }
        CardSetIterator { cards: cards }
    }
}

impl PartialEq for CardSet {
    fn eq(&self, other: &CardSet) -> bool {
        self.into_iter().eq(other.into_iter())
    }
}

pub mod lists {
    use super::Card;
    pub const FIRST_SET: [Card; 10] = [Card::Cellar, Card::Market, Card::Militia, Card::Mine, Card::Moat, Card::Remodel, Card::Smithy, Card::Village, Card::Woodcutter, Card::Workshop];

    pub const BASE_TREASURE: [Card; 3] = [Card::Copper, Card::Silver, Card::Gold];
    pub const BASE_VICTORY: [Card; 3] = [Card::Estate, Card::Duchy, Card::Province];
}
