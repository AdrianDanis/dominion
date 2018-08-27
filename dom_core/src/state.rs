use rand::SeedableRng;
use rand::Rng;

use card::{Card, CardSet};
use rules::Players;

#[derive(Debug, Clone, Copy, Enum)]
#[repr(u32)]
pub enum Player {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerSet {
    bitset: u32,
}

impl PlayerSet {
    pub fn just(p: Player) -> PlayerSet{
        PlayerSet {
            bitset: 1 << (p as u32),
        }
    }
    pub fn contains(&self, p: Player) -> bool {
        ((self.bitset >> (p as u32)) & 1) == 1
    }
}

/// Cards are revealed from the hand of a player and are shown to a single player
/// or all players. Having an 'all' option instead of requiring multiple reveals
/// provides an indication of whether a reveal was public or directed
#[derive(Debug, Clone, Copy)]
pub enum Reveal {
    All,
    Just(PlayerSet),
}

/// Defines a change to the board state
///
/// Mutations are meant to be the smallest indivisible operations that occur on the
/// game board. Important a game state can always be recreated simply by replying
/// a list of mutations. Mutations should not have implicit state changes even if they
/// could. For example if a player runs out of actions there should still be an explicit
/// mutation to move to the buy phase.
///
/// Mutations that involve shuffling will use a defined randomness source. This means that
/// recreating a board state that can be *continued* requires knowing the rng seed to
/// get the hidden information. Replaying up until the current state does not need the seed
/// as state that may be hidden has an explicit reveal `Mutation` before being used.
/// Reveals can be directed to a subset of players to describe partial information.
#[derive(Debug, Clone, Copy)]
pub enum Mutation {
    /// Add players to the game
    ///
    /// Players can only be set once and must be set prior to performing any mutations
    /// that require a `Player`.
    SetPlayers(Players),
    /// Add card stack
    ///
    /// Card stacks are treated specially by the state as you can only add a kind of stack
    /// once and a count of
    AddStack(Card, u32),
    /// Reveal hand card(s)
    ///
    /// A players hand is treated as a set of cards and so no IDs are associated with
    /// cards. Multiple reveals from a single player could be showing previously revealed
    /// cards. This prevents any kind of positional based hand tracking.
    ///
    /// The cards are optional to allow for game states that do not have full knowledge to
    /// still talk about a reveal happening.
    RevealHandCards(Player, Option<CardSet>, Reveal),
    /// Reveal top deck
    ///
    /// Reveals the top card of a players deck
    ///
    /// The card is optional to allow for game states that do not have full knowledge to
    /// still talk about a reveal happening.
    RevealTopDeck(Player, Option<Card>, Reveal),
    /// Draw top card of the deck
    ///
    /// Takes the top card of the players deck and puts it in their hand
    DrawCard(Player),
    /// Move a card from hand to play area
    PlayCard(Player, Card),
    /// Gain a card from supply to discard
    GainCard(Player, Card),
    /// Shuffle discard and make it the deck
    ///
    /// This implies that there is no current deck
    ShuffleDiscard(Player),
}

/// Convenience alias for grouping ordered mutations
pub type Mutations = Vec<Mutation>;

#[derive(Debug, Clone)]
pub struct PlayerState {
    hand: Vec<Option<Card>>,
    played: CardSet,
    discard: CardSet,
    draw: Vec<Option<Card>>,
}

impl PlayerState {
    pub fn hand_iter(&self) -> impl Iterator<Item = Option<Card>> {
        self.hand.clone().into_iter()
    }
    pub fn played_iter(&self) -> impl Iterator<Item = Card> {
        self.played.into_iter()
    }
    pub fn discard_iter(&self) -> impl Iterator<Item = Card> {
        self.discard.into_iter()
    }
    pub fn draw_iter(&self) -> impl Iterator<Item = Option<Card>> {
        self.draw.clone().into_iter().rev()
    }
}

impl PartialEq for PlayerState {
    fn eq(&self, other: &PlayerState) -> bool {
        self.hand == other.hand
            && self.played == other.played
            && self.discard == other.discard
            && self.draw.iter().eq(other.draw.iter())
    }
}

pub type RNGSource = ::rand::prng::chacha::ChaChaRng;
pub type RNGSeed = [u8; 32];

/// Definition of the state of a board
///
/// This structure is immutable and any mutations must be done through an explicit `Mutation`.
#[derive(Debug, Clone)]
pub struct BoardState {
    supply: CardSet,
    trash: Vec<Card>,
    stacks: CardSet,
    players: Vec<PlayerState>,
    rand: Option<RNGSource>,
    turn: Player,
}

impl PartialEq for BoardState {
    fn eq(&self, other: &BoardState) -> bool {
        self.supply == other.supply
            && self.stacks == other.stacks
            && self.trash.iter().eq(other.trash.iter())
            && self.players.iter().eq(other.players.iter())
    }
}

impl BoardState {
    pub fn new(seed: Option<RNGSeed>) -> Self {
        BoardState {
            supply: CardSet::empty(),
            trash: Vec::new(),
            stacks: CardSet::empty(),
            players: Vec::new(),
            rand: seed.map(RNGSource::from_seed),
            turn: Player::P0,
        }
    }
    pub fn supply_stacks(&self) -> impl Iterator<Item = (Card, &u32)> {
        self.supply.count_iter().filter(move |(key, _)| self.stacks.contains(*key))
    }
    pub fn active_player(&self) -> Player {
        self.turn
    }
    pub fn get_player(&self, p: Player) -> Option<&PlayerState> {
        self.players.get(p as u32 as usize)
    }
    fn set_players(self, p: Players) -> Option<BoardState> {
        Some(self)
            .filter(|x| x.players.len() == 0)
            .map(|mut x| {
                x.players.append(&mut [PlayerState {
                    hand: Vec::new(),
                    played: CardSet::empty(),
                    discard: CardSet::empty(),
                    draw: Vec::new(),
                }].iter().cycle().take(p as usize).cloned().collect());
                x
            })
    }
    fn add_stack(self, card: Card, count: u32) -> Option<BoardState> {
        Some(self)
            .filter(|x| !x.stacks.contains(card))
            .map(|mut x| {
                x.stacks.insert(card, 1);
                x.supply.insert(card, count);
                x}
            )
    }
    fn gain_card(self, player: Player, card: Card) -> Option<BoardState> {
        let mut b = self;
        if b.supply.take(card, 1) {
            if let Some(player) = b.players.get_mut(player as usize) {
                player.discard.insert(card, 1);
            } else {
                return None;
            }
            Some(b)
        } else {
            None
        }
    }
    fn shuffle(self, player: Player) -> Option<BoardState> {
        let mut b = self;
        if let Some(player) = b.players.get_mut(player as usize) {
            if player.draw.len() == 0 {
                let discard = player.discard.drain();
                // check if we have rng powers to shuffle
                if let Some(mut rng) = b.rand.as_mut() {
                    player.draw = discard.map(|x| Some(x)).collect();
                    rng.shuffle(player.draw.as_mut_slice());
                } else {
                    player.draw = discard.map(|_| None).collect();
                }
            } else {
                return None
            }
        } else {
            return None;
        }
        Some(b)
    }
    fn reveal_top_deck(self, player: Player, card: Option<Card>, reveal: Reveal) -> Option<BoardState> {
        let mut b = self;
        if let Some(player) = b.players.get_mut(player as usize) {
            if let Some(_) = player.draw.pop() {
                // Should probably check that if we already knew the card we are not changing its information
                player.draw.push(card);
            } else {
                return None;
            }
        } else {
            return None;
        }
        Some(b)
    }
    fn draw_card(self, player: Player) -> Option<BoardState> {
        let mut b = self;
        if let Some(player) = b.players.get_mut(player as usize) {
            if let Some(card) = player.draw.pop() {
                player.hand.push(card);
            } else {
                return None;
            }
        } else {
            return None;
        }
        Some(b)
    }
    pub fn mutate(self, m: Mutation) -> Option<BoardState> {
        match m {
            Mutation::SetPlayers(p) => self.set_players(p),
            Mutation::AddStack(card, count) => self.add_stack(card, count),
            Mutation::GainCard(p, c) => self.gain_card(p, c),
            Mutation::ShuffleDiscard(p) => self.shuffle(p),
            Mutation::RevealTopDeck(p, c, r) => self.reveal_top_deck(p, c, r),
            Mutation::DrawCard(p) => self.draw_card(p),
            _ => unimplemented!()
        }
    }
    /// Counts how many of a certain card are presently in the supply
    ///
    /// Returns `None`if the requested card was never in the supply. This is to distinguish
    /// a card that was never in the game, `None`, versus an empty pile `Some(0)`
    pub fn count_supply(&self, card: Card) -> Option<u32> {
        if self.stacks.contains(card) {
            Some(self.supply.count(card))
        } else {
            None
        }
    }
    /// Perform multiple mutations
    ///
    /// Only returns a board state if *all* mutations apply successfully
    pub fn mutate_multi(self, mutations: &Mutations) -> Option<BoardState> {
        let mut state = Some(self);
        for m in mutations {
            state = state.and_then(|s| s.mutate(*m));
        }
        state
    }
    pub fn from_mutations(mutations: &Mutations) -> Option<BoardState> {
        Self::new(None).mutate_multi(mutations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cannot_insert_stack_twice() {
        let gs = BoardState::new(None);
        assert_eq!(gs.supply.count(Card::Copper), 0);
        let gs2 = gs.mutate(Mutation::AddStack(Card::Copper, 10)).unwrap();
        assert_eq!(gs2.supply.count(Card::Copper), 10);
        assert_eq!(gs2.mutate(Mutation::AddStack(Card::Copper, 10)), None);
    }
    #[test]
    fn cannot_insert_players_twice() {
        let gs = BoardState::new(None);
        assert_eq!(gs.get_player(Player::P0), None);
        assert_eq!(gs.get_player(Player::P1), None);
        assert_eq!(gs.get_player(Player::P2), None);
        let gs2 = gs.mutate(Mutation::SetPlayers(Players::Two)).unwrap();
        assert_ne!(gs2.get_player(Player::P0), None);
        assert_ne!(gs2.get_player(Player::P1), None);
        assert_eq!(gs2.get_player(Player::P2), None);
        assert_eq!(gs2.clone().mutate(Mutation::SetPlayers(Players::Two)), None);
        assert_eq!(gs2.mutate(Mutation::SetPlayers(Players::Three)), None);
    }
}
