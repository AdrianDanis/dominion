extern crate dom_core;

fn print_board_state(state: &dom_core::BoardState) {
    println!("Supply:");
    for (key, value) in state.supply_stacks().filter(|(key, value)| [dom_core::Card::Copper, dom_core::Card::Silver, dom_core::Card::Gold].contains(key)) {
        println!("\t{:?}: {}", key, if *value == 0 {format!("DEPLETED")} else {format!("{}", value)});
    }
}

fn main() {
    let (mut game, mutations) = dom_core::Game::new_first_game(dom_core::Players::Two);
    let mut game_p0 = dom_core::Game::from_mutations(&mutations).unwrap();
    let mut game_p1 = dom_core::Game::from_mutations(&mutations).unwrap();
//    println!("Build initial game\n{:?}\nThen using mutations\n{:?}\nBuilt perspective p0\n{:?}\nAnd perspective p1\n{:?}\n", game, mutations, game_p0, game_p1);
    print_board_state(game.board_state());
}
