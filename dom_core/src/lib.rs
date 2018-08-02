/// Internal player indexing
///
/// For low level board state purposes a 'player' is a number from 0..num_players
type Player = u32;

/// Enumeration of all different cards
enum Card {
    Copper,
    Silver,
    Gold,
    Estate,
    Duchy,
    Province,
    Curse,
}

/// Definition of hidden card locations
///
/// We only need to talk about abstract locations of unknown cards, as a result this
/// enumeration only contains the specific cases where cards are potentially hidden
/// but we still want to talk about them (typically to reveal them)
enum Location {
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
    /// Reveal a card to a particular player, or all players
    ///
    /// The mutation does not hold the card itself so that it is sensible to replay the
    /// mutation list in cases where the rng (and hence the full state) is both known
    /// and unknown.
    ///
    /// In essence this is just recording the fact that an information reveal event happened,
    /// but not what was actually revealed.
    RevealCard(Location, Option<Player>),
    /// Move a card from hand to play area
    PlayCard,
}

/// Convenience alias for grouping ordered mutations
type Mutations = Vector<Mutation>;

struct Reveal {
    loc: Location,
    card: Card,
};

// Takes a list of mutations and works out what cards might be revealed for each mutation
// for a particular player.
//
// This is useful for translating a set of mutations to be given to a `BoardState` that does
// not have accurate rng. For example an untrusted game client.
fn make_card_reveals(muts: &Mutations, p: Player) -> Vector<Mutation, Option<Reveal>> {
    unimplemented!()
}

/// Definition of the state of a board
///
/// This structure is immutable and any mutations must be done through an explicit `Mutation`.
struct BoardState {
}

impl BoardState {
    fn mutate(self, m: Mutation) -> BoardState {
        unimplemented!()
    }
    fn reveal(self, rev: Reveal) -> BoardState {
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

/// Defines and runs the rules and logic of a dominion game
///
/// Internally has a `BoardState` and performs actions against it.
struct Game {
}

impl Game {
    fn state(&self) -> State {
        unimplemented!()
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
