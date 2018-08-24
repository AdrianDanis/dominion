extern crate dom_core;

use std::fmt;
use dom_core::card;

struct SupplyCard {
    card: dom_core::Card,
    quantity: u32,
}

impl<'a> From<(dom_core::Card, &'a u32)> for SupplyCard {
    fn from(sup: (dom_core::Card, &'a u32)) -> SupplyCard {
        SupplyCard { card: sup.0, quantity: *sup.1}
    }
}

impl fmt::Display for SupplyCard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self.card, if self.quantity == 0 { format!("DEPLETED")} else { format!("{}", self.quantity)})
    }
}

fn print_supply_cards(state: &dom_core::BoardState, cards: &[dom_core::Card]) {
    for kv in state.supply_stacks().filter(|(key, _)| cards.contains(key)) {
        println!("\t{}", SupplyCard::from(kv));
    }
}

fn is_base_card(card: dom_core::Card) -> bool {
    card::lists::BASE_TREASURE.contains(&card) ||
        card::lists::BASE_VICTORY.contains(&card) ||
        card == dom_core::Card::Curse
}

fn print_board_state(state: &dom_core::BoardState) {
    println!("Supply:");
    print_supply_cards(state, &card::lists::BASE_TREASURE);
    print_supply_cards(state, &card::lists::BASE_VICTORY);
    print_supply_cards(state, &[dom_core::Card::Curse]);
    let v: Vec<dom_core::Card> = state.supply_stacks().map(|(key, v)| key).filter(|key| !is_base_card(*key)).collect();
    print_supply_cards(state, &v);
    println!("Trash:");
    println!("\tNOT DISPLAYED");
}

fn main() {
    let (mut game, mutations) = dom_core::Game::new_first_game(dom_core::Players::Two);
    let mut game_p0 = dom_core::Game::from_mutations(&mutations).unwrap();
    let mut game_p1 = dom_core::Game::from_mutations(&mutations).unwrap();
//    println!("Build initial game\n{:?}\nThen using mutations\n{:?}\nBuilt perspective p0\n{:?}\nAnd perspective p1\n{:?}\n", game, mutations, game_p0, game_p1);
    print_board_state(game.board_state());
}
