#![feature(vec_remove_item)]

#[macro_use]
extern crate enum_map;
extern crate rand;

pub mod card;
mod state;
mod rules;

pub use card::{Card, CardSet};
pub use rules::{Players, Rules};
pub use state::{BoardState, Mutations, Player, Mutation, Reveal, PlayerSet, PlayerState, PlayerPhase};

use state::RNGSeed;

use rand::random;

/// Current state of the game
///
/// This indirectly implies what actions are valid against the game
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// Action phase of the current player
    ///
    /// Specifically this means we are waiting for the active player to play an action card or
    /// go to the buy phase
    ActionPhase,
    /// Buy phase of the current player
    BuyPhase,
}

// TODO: Is there a better way to encode what actions are permissible by different states?
// Encoding explicit transitions is difficult, but maybe could be modeled as a non deterministic
// state machine to deal with hidden information from draws / complexity?
/// Actions that can be performed in the game
///
/// Actions are higher level than mutations and guide the unfolding of the game where
/// there are choices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// End action phase
    EndAction,
    /// End buy phase
    EndBuy,
}

/// Holds an in progress game update
///
/// Most high level game actions consist of multiple smaller board state mutations. This struct wraps
/// a `BoardState` with the series of `Mutations` that have been applied to it to diverge it from the
/// original `game`. Once all the updates and logic is complete the update can be applied to the original
/// `game`.
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
    fn try_append(&mut self, mutation: Mutation) -> Option<()> {
        if let Some(state) = self.state.clone().mutate(mutation) {
            self.state = state;
            self.updates.push(mutation);
            Some(())
        } else {
            None
        }
    }
    fn apply(self) -> Mutations {
        self.game.state = self.state;
        self.updates
    }
    fn try_draw_card(&mut self, player: Player) -> Option<()> {
        // Attempt to shuffle + reveal + draw
        self.try_append(Mutation::ShuffleDiscard(player));
        if let Some(card) = self.state.get_player(player).and_then(|p| p.draw_iter().next()) {
            self.try_append(Mutation::DrawCard(player, card))
        } else {
            None
        }
    }
    fn begin_turn(&mut self, player: Player) -> Option<()> {
        // end the current players turn if neccessary
        let last_active = self.state.active_player();
        self.state.get_player(last_active).map(|p| p.get_phase())
            .filter(|phase| *phase != PlayerPhase::NotTurn)
            .map(|_|
                self.try_append(Mutation::SetPhase(last_active, PlayerPhase::NotTurn))
            );
        self.try_append(Mutation::ChangeTurn(player))?;
        self.try_append(Mutation::SetPhase(player, PlayerPhase::Action))?;
        self.try_append(Mutation::SetBuys(player, 1))?;
        self.try_append(Mutation::SetActions(player, 1))?;
        self.try_append(Mutation::SetGold(player, 0))
    }
    fn end_turn(&mut self, player: Player) -> Option<()>{
        // Create a copy of the hand to get around borrowing whilst updating problems
        let hand = self.state.get_player(player)?.hand_iter().collect::<Vec<Option<Card>>>();
        // discard hand
        for card in hand {
            self.try_append(Mutation::DiscardHand(player, card?))?;
        }
        // discard played cards
        self.try_append(Mutation::DiscardPlayed(player))?;
        // draw a new hand. These draws can fail as our deck may be too small
        for _ in 0..5 {
            self.try_draw_card(player);
        }
        // change phase
        self.try_append(Mutation::SetPhase(player, PlayerPhase::NotTurn))
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
    /// Create new game with given rules
    fn new_from_seed(rules: Rules, seed: RNGSeed) -> (Game, Mutations) {
        let mut game =
            Game {
                state: BoardState::new(Some(seed)),
            };
        let mutations;
        {
            let mut up = Update::from(&mut game);
            up.try_append(Mutation::SetPlayers(rules.players));
            let stacks = card::lists::BASE_TREASURE.iter()
                .chain(card::lists::BASE_VICTORY.iter())
                .chain(rules.set.iter())
                .chain([Card::Curse].iter());
            for card in stacks {
                up.try_append(Mutation::AddStack(*card, card.starting_count(rules.players)));
            }
            for player in Player::iter_players(rules.players) {
                for _ in 0..3 {
                    up.try_append(Mutation::GainCard(*player, Card::Estate));
                }
                for _ in 0..7 {
                    up.try_append(Mutation::GainCard(*player, Card::Copper));
                }
                up.try_append(Mutation::ShuffleDiscard(*player));
                for _ in 0..5 {
                    up.try_draw_card(*player);
                }
            }
            up.begin_turn(Player::P0);
            mutations = up.apply();
        }
        (game, mutations)
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
    pub fn state(&self) -> State {
        let active = self.board_state().get_player(self.board_state().active_player()).unwrap();
        match active.get_phase() {
            PlayerPhase::Action => State::ActionPhase,
            PlayerPhase::Buy => State::BuyPhase,
            _ => unimplemented!(),
        }
    }
    pub fn board_state(&self) -> &BoardState {
        &self.state
    }
    /// Perform an action against the game
    ///
    /// If the action can be successfully performed the internal game state is updated
    /// and the list of mutations that were performed is returned.
    pub fn act(&mut self, action: Action) -> Option<Mutations> {
        let state = self.state();
        let active = self.board_state().active_player();
        let mut up = Update::from(self);
        match action {
            Action::EndAction if state == State::ActionPhase => {up.try_append(Mutation::SetPhase(active,PlayerPhase::Buy)); Some(up.apply())},
            Action::EndBuy if state == State::BuyPhase => {
                let next = active.next(up.state.num_players().unwrap());
                up.end_turn(active)?;
                up.begin_turn(next)?;
                Some(up.apply())
            },
            _ => None
        }
    }
    pub fn apply_mutations(&mut self, mutations: &Mutations) -> bool {
        match self.state.clone().mutate_multi(mutations) {
            Some(new_state) => {self.state = new_state; true},
            None => false,
        }
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
    #[test]
    fn new_game_first_turn_state() {
        let g = Game::new_first_game(Players::Two).0;
        assert_eq!(g.board_state().active_player(), Player::P0);
        let p0 = g.board_state().get_player(Player::P0).unwrap();
        assert_eq!(p0.hand_iter().count(), 5);
        assert_eq!(p0.get_phase(), PlayerPhase::Action);
        assert_ne!(g.board_state().get_player(Player::P1).unwrap().get_phase(), PlayerPhase::Action);
        assert_eq!(p0.get_buys(), 1);
        assert_eq!(p0.get_actions(), 1);
        assert_eq!(p0.get_gold(), 0);
        assert_eq!(g.state(), State::ActionPhase);
    }
}
