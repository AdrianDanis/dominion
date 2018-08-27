#[macro_use]
extern crate enum_map;
extern crate rand;

pub mod card;
mod state;
mod rules;

pub use card::{Card, CardSet};
pub use rules::{Players, Rules};
pub use state::{BoardState, Mutations, Player, Mutation, Reveal, PlayerSet, PlayerState};

use state::RNGSeed;

use rand::random;

use enum_map::{Enum};

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

struct Update<'a> {
    state: BoardState,
    updates: Mutations,
    game: &'a mut Game,
}

impl<'a> From<&'a mut Game> for Update<'a> {
    fn from(game: &'a mut Game) -> Update<'a> {
        Update {
            state: game.state.clone(),
            updates: Vec::new(),
            game: game,
        }
    }
}

impl<'a> Update<'a> {
    pub fn try_append(&mut self, mutation: Mutation) -> bool {
        if let Some(state) = self.state.clone().mutate(mutation) {
            self.state = state;
            self.updates.push(mutation);
            true
        } else {
            false
        }
    }
    pub fn apply(self) -> Mutations {
        self.game.state = self.state;
        self.updates
    }
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
    fn try_draw_card(&mut self, player: Player) -> Mutations {
        // Attempt to shuffle + reveal + draw
        let mut up = Update::from(self);
        up.try_append(Mutation::ShuffleDiscard(player));
        if let Some(card) = up.state.get_player(player).and_then(|p| p.draw_iter().next()) {
            up.try_append(Mutation::RevealTopDeck(player, card, Reveal::Just(PlayerSet::just(player))));
            up.try_append(Mutation::DrawCard(player));
        }
        up.apply()
    }
    /// Create new game with given rules
    fn new_from_seed(rules: Rules, seed: RNGSeed) -> (Game, Mutations) {
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
        let mut game =
            Game {
                state: BoardState::new(Some(seed)).mutate_multi(&init_muts).unwrap(),
            };
        for p in 0..(rules.players as u32) {
            let player = Enum::<u32>::from_usize(p as usize);
            for _ in 0..5 {
                let mut muts = game.try_draw_card(player);
                init_muts.append(&mut muts);
            }
        }
        (game, init_muts)
    }
    fn new(rules: Rules) -> (Game, Mutations) {
        let seed = [
            random(),random(),random(),random(),random(),random(),random(),random(),
            random(),random(),random(),random(),random(),random(),random(),random(),
            random(),random(),random(),random(),random(),random(),random(),random(),
            random(),random(),random(),random(),random(),random(),random(),random()];
        Self::new_from_seed(rules, seed)
    }
    /// Initialize 'First Game' layout
    pub fn new_first_game(players: Players) -> (Game, Mutations) {
        Self::new( Rules {
            players: players,
            set: card::lists::FIRST_SET,
        })
    }
    fn state(&self) -> State {
        unimplemented!()
    }
    pub fn board_state(&self) -> &BoardState {
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
    pub const DUMMY_SEED: RNGSeed = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
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
    #[test]
    fn rng_seeds_stable() {
        let (g, _) = Game::new_from_seed(
            Rules {
                players: Players::Two,
                set: card::lists::FIRST_SET,
            },
            DUMMY_SEED
        );
        let (g2, _) = Game::new_from_seed(
            Rules {
                players: Players::Two,
                set: card::lists::FIRST_SET,
            },
            DUMMY_SEED
        );
        assert_eq!(g.board_state(), g2.board_state());
    }
}
