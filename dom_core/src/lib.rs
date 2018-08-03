enum Player {
    P0,
    P1,
    P2,
    P3,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
enum Players {
    Two = 2,
    Three = 3,
    Four = 4,
}

struct PlayerSet {
}

/// Cards are revealed from the hand of a player and are shown to a single player
/// or all players. Having an 'all' option instead of requiring multiple reveals
/// provides an indication of whether a reveal was public or directed
enum Reveal {
    All,
    None,
    Players(PlayerSet),
}

/// Enumeration of all different cards
#[derive(Debug, Clone, Copy)]
enum Card {
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
    fn starting_count(&self, players: Players) -> u32 {
        unimplemented!()
    }
}

/// Optimized datastructure for storing strictly unordered sets of cards
///
/// Provides a way to store sets of cards that have no ordering. The lack of ordering
/// is so that no information can be informed by the order cards are iterated from the
/// set.
struct CardSet {
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
enum Mutation {
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
}

/// Convenience alias for grouping ordered mutations
type Mutations = Vec<Mutation>;

/// Definition of the state of a board
///
/// This structure is immutable and any mutations must be done through an explicit `Mutation`.
struct BoardState {
}

impl BoardState {
    fn new() -> Self {
        unimplemented!()
    }
    fn mutate(self, m: Mutation) -> Option<BoardState> {
        unimplemented!()
    }
    /// Counts how many of a certain card are presently in the supply
    ///
    /// Returns `None`if the requested card was never in the supply. This is to distinguish
    /// a card that was never in the game, `None`, versus an empty pile `Some(0)`
    fn count_supply(&self, card: Card) -> Option<u32> {
        unimplemented!()
    }
    /// Perform multiple mutations
    ///
    /// Only returns a board state if *all* mutations apply successfully
    fn mutate_multi(self, m: &Mutations) -> Option<BoardState> {
        unimplemented!()
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
struct Game {
    state: BoardState,
}

impl Game {
    fn start_stack(c: Card, players: Players) -> Mutation {
        Mutation::AddStack(c, c.starting_count(players))
    }
    /// Create new game with given rules
    fn new(rules: Rules) -> (Game, Mutations) {
        let init_muts = vec![
            Mutation::SetPlayers(rules.players),
            Self::start_stack(Card::Copper, rules.players),
        ];
        (
            Game {
                state: BoardState::new().mutate_multi(&init_muts).unwrap(),
            },
            init_muts
        )
    }
    /// Initialize 'First Game' layout
    fn new_first_game(players: Players) -> (Game, Mutations) {
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
