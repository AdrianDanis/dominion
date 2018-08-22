#[macro_use]
extern crate enum_map;
extern crate rand;

use rand::SeedableRng;

use enum_map::{Enum, EnumMap};

#[derive(Debug, Clone, Copy, Enum)]
#[repr(u32)]
pub enum Player {
    P0 = 0,
    P1 = 1,
    P2 = 2,
    P3 = 3,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Players {
    Two = 2,
    Three = 3,
    Four = 4,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerSet {
}

/// Cards are revealed from the hand of a player and are shown to a single player
/// or all players. Having an 'all' option instead of requiring multiple reveals
/// provides an indication of whether a reveal was public or directed
#[derive(Debug, Clone, Copy)]
pub enum Reveal {
    All,
    None,
    Players(PlayerSet),
}

/// Enumeration of all different cards
#[repr(u32)]
#[derive(Debug, Clone, Copy, Enum)]
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
    fn starting_count(&self, players: Players) -> u32 {
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

/// Optimized datastructure for storing strictly unordered sets of cards
///
/// Provides a way to store sets of cards that have no ordering. The lack of ordering
/// is so that no information can be informed by the order cards are iterated from the
/// set.
#[derive(Debug, Clone, Copy)]
pub struct CardSet {
    map: EnumMap<Card, u32>,
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
    fn empty() -> CardSet {
        CardSet {
            map: EnumMap::new(),
        }
    }
    fn insert(&mut self, card: Card, count: u32) {
        self.map[card] += count;
    }
    fn take(&mut self, card: Card, count: u32) -> bool {
        if self.map[card] < count {
            false
        } else {
            self.map[card] -= count;
            true
        }
    }
    fn count(&self, card: Card) -> u32 {
        self.map[card]
    }
    /// Check if there are non zero copies of a card in the set
    fn contains(&self, card: Card) -> bool {
        self.count(card) > 0
    }
    fn drain(&mut self) -> CardSetIterator {
        let it = self.into_iter();
        *self = Self::empty();
        return it;
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
    /// Move a card from hand to play area
    PlayCard(Player, Card, Reveal),
    /// Gain a card from supply to discard
    GainCard(Player, Card),
    /// Shuffle discard and make it the deck
    ///
    /// This implies that there is no current deck
    ShuffleDiscard(Player),
}

/// Convenience alias for grouping ordered mutations
type Mutations = Vec<Mutation>;

#[derive(Debug, Clone)]
struct PlayerState {
    hand: CardSet,
    played: CardSet,
    discard: CardSet,
    draw: Vec<Option<Card>>,
}

type RNGSource = rand::prng::chacha::ChaChaRng;
type RNGSeed = [u8; 32];

const DUMMY_SEED: RNGSeed = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

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
}

impl BoardState {
    fn new(seed: Option<RNGSeed>) -> Self {
        BoardState {
            supply: CardSet::empty(),
            trash: Vec::new(),
            stacks: CardSet::empty(),
            players: Vec::new(),
            rand: seed.map(RNGSource::from_seed),
        }
    }
    fn set_players(self, p: Players) -> Option<BoardState> {
        if self.players.len() != 0 {
            None
        } else {
            let mut b = self;
            for _ in 0..(p as u32) {
                b.players.push(
                    PlayerState {
                        hand: CardSet::empty(),
                        played: CardSet::empty(),
                        discard: CardSet::empty(),
                        draw: Vec::new(),
                    }
                );
            }
            Some(b)
        }
    }
    fn add_stack(self, card: Card, count: u32) -> Option<BoardState> {
        if self.stacks.contains(card) {
            None
        } else {
            let mut b = self;
            b.stacks.insert(card, 1);
            b.supply.insert(card, count);
            Some(b)
        }
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
                player.draw = player.discard.drain().map(|x| Some(x)).collect();
                // check if we have rng powers to shuffle
                unimplemented!()
            } else {
                return None
            }
        } else {
            return None;
        }
        Some(b)
    }
    fn mutate(self, m: Mutation) -> Option<BoardState> {
        match m {
            Mutation::SetPlayers(p) => self.set_players(p),
            Mutation::AddStack(card, count) => self.add_stack(card, count),
            Mutation::GainCard(p, c) => self.gain_card(p, c),
            Mutation::ShuffleDiscard(p) => self.shuffle(p),
            _ => unimplemented!()
        }
    }
    /// Counts how many of a certain card are presently in the supply
    ///
    /// Returns `None`if the requested card was never in the supply. This is to distinguish
    /// a card that was never in the game, `None`, versus an empty pile `Some(0)`
    fn count_supply(&self, card: Card) -> Option<u32> {
        if self.stacks.contains(card) {
            Some(self.supply.count(card))
        } else {
            None
        }
    }
    /// Perform multiple mutations
    ///
    /// Only returns a board state if *all* mutations apply successfully
    fn mutate_multi(self, mutations: &Mutations) -> Option<BoardState> {
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

/// Current state of the game
///
/// This indirectly implies what actions are valid against the game
enum State {
    /// Action phase of the current player
    ActionPhase,
}

/// Actions that can be performed in the game
///
/// Actions are higher level than mutations and guide the unfolding of the game where
/// there are choices.
enum Action {
    /// End action phase
    EndAction
}

struct Rules {
    players: Players,
    set: [Card; 10],
}

/// Defines and runs the rules and logic of a dominion game
///
/// Internally has a `BoardState` and performs actions against it.
#[derive(Debug, Clone)]
pub struct Game {
    state: BoardState,
}

impl Game {
    fn start_stack(c: Card, players: Players) -> Mutation {
        Mutation::AddStack(c, c.starting_count(players))
    }
    pub fn from_state(state: BoardState) -> Option<Game> {
        Some(Game {state: state})
    }
    pub fn from_mutations(mutations: &Mutations) -> Option<Game> {
        BoardState::from_mutations(mutations).and_then(Self::from_state)
    }
    /// Create new game with given rules
    fn new(rules: Rules) -> (Game, Mutations) {
        let mut init_muts = vec![
            Mutation::SetPlayers(rules.players),
            Self::start_stack(Card::Copper, rules.players),
            Self::start_stack(Card::Silver, rules.players),
            Self::start_stack(Card::Gold, rules.players),
            Self::start_stack(Card::Estate, rules.players),
            Self::start_stack(Card::Duchy, rules.players),
            Self::start_stack(Card::Province, rules.players),
            Self::start_stack(Card::Curse, rules.players),
            Self::start_stack(rules.set[0], rules.players),
            Self::start_stack(rules.set[1], rules.players),
            Self::start_stack(rules.set[2], rules.players),
            Self::start_stack(rules.set[3], rules.players),
            Self::start_stack(rules.set[4], rules.players),
            Self::start_stack(rules.set[5], rules.players),
            Self::start_stack(rules.set[6], rules.players),
            Self::start_stack(rules.set[7], rules.players),
            Self::start_stack(rules.set[8], rules.players),
            Self::start_stack(rules.set[9], rules.players),
        ];
        for p in 0..(rules.players as u32) {
            let player = Enum::<u32>::from_usize(p as usize);
            for _ in 0..3 {
                init_muts.push(Mutation::GainCard(player, Card::Estate));
            }
            for _ in 0..7 {
                init_muts.push(Mutation::GainCard(player, Card::Copper));
            }
            init_muts.push(Mutation::ShuffleDiscard(player));
        }
        (
            Game {
                state: BoardState::new().mutate_multi(&init_muts).unwrap(),
            },
            init_muts
        )
    }
    /// Initialize 'First Game' layout
    pub fn new_first_game(players: Players) -> (Game, Mutations) {
        Self::new( Rules {
            players: players,
            set: [Card::Cellar, Card::Market, Card::Militia, Card::Mine, Card::Moat, Card::Remodel, Card::Smithy, Card::Village, Card::Woodcutter, Card::Workshop],
        })
    }
    fn state(&self) -> State {
        unimplemented!()
    }
    fn board_state(&self) -> &BoardState {
        &self.state
    }
    /// Perform an action against the game
    ///
    /// If the action can be successfully performed the internal game state is updated
    /// and the list of mutations that were performed is returned.
    fn act(&mut self, action: Action) -> Option<Mutations> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_player_victory_cards() {
        let g = Game::new_first_game(Players::Two).0;
        assert_eq!(g.board_state().count_supply(Card::Estate), Some(8));
        assert_eq!(g.board_state().count_supply(Card::Duchy), Some(8));
        assert_eq!(g.board_state().count_supply(Card::Province), Some(8));
    }
    #[test]
    fn many_player_victory_cards() {
        let mut g = Game::new_first_game(Players::Three).0;
        assert_eq!(g.board_state().count_supply(Card::Estate), Some(12));
        assert_eq!(g.board_state().count_supply(Card::Duchy), Some(12));
        assert_eq!(g.board_state().count_supply(Card::Province), Some(12));
        g = Game::new_first_game(Players::Four).0;
        assert_eq!(g.board_state().count_supply(Card::Estate), Some(12));
        assert_eq!(g.board_state().count_supply(Card::Duchy), Some(12));
        assert_eq!(g.board_state().count_supply(Card::Province), Some(12));
    }
}
