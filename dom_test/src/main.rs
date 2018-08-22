extern crate dom_core;

fn main() {
    let (mut game, mutations) = dom_core::Game::new_first_game(dom_core::Players::Two);
    let mut game_p0 = dom_core::Game::from_mutations(&mutations).unwrap();
    let mut game_p1 = dom_core::Game::from_mutations(&mutations).unwrap();
    println!("Build initial game\n{:?}\nThen using mutations\n{:?}\nBuilt perspective p0\n{:?}\nAnd perspective p1\n{:?}\n", game, mutations, game_p0, game_p1);
}
