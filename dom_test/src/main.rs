extern crate dom_core;

use std::{io, fmt};
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
    println!("It is player {}'s turn", (state.active_player() as u32) + 1);
}

#[derive(Debug, Clone, Copy)]
enum MaybeCardRange {
    Known(dom_core::Card),
    Unknown(u32),
}

impl fmt::Display for MaybeCardRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MaybeCardRange::Known(card) => write!(f, "{}", format!("{:?}", card)),
            MaybeCardRange::Unknown(count) => write!(f, "{} unknown", format!("{}", count)),
        }
    }
}

struct MaybeCardList {
    ranges: Vec<MaybeCardRange>,
}

impl<T: Iterator<Item = Option<dom_core::Card>> + Sized> From<T> for MaybeCardList {
    fn from(iter: T) -> Self {
        MaybeCardList {
            ranges:
                iter.fold(Vec::new(), |acc_old, x|
                    {
                        let mut acc = acc_old;
                        if let Some(card) = x {
                            acc.push(MaybeCardRange::Known(card))
                        } else {
                            match acc.pop() {
                                Some(old@MaybeCardRange::Known(_)) => {
                                    acc.push(old);
                                    acc.push(MaybeCardRange::Unknown(1))
                                },
                                Some(MaybeCardRange::Unknown(count)) =>
                                    acc.push(MaybeCardRange::Unknown(count + 1)),
                                None =>
                                    acc.push(MaybeCardRange::Unknown(1)),
                            }
                        }
                        acc
                    }
                ),
        }
    }
}

impl fmt::Display for MaybeCardList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ranges.iter()
            .map(|x| format!("{}", x))
            // Now turn into a single joined string
            .collect::<Vec<String>>().join(", ")
        )
    }
}

fn show_player(player: &dom_core::PlayerState) {
    println!("Hand: [{}]", MaybeCardList::from(player.hand_iter()));
    let played_vec: Vec<dom_core::Card> = player.played_iter().collect();
    println!("Played: {:?}", played_vec);
    println!("Deck: [{}]", MaybeCardList::from(player.draw_iter()));
    println!("Discard: NOT DISPLAYED");
    println!("Actions: {} Buys: {} Gold: {}", player.get_actions(), player.get_buys(), player.get_gold());
}

fn mutations_for_player(mutations: dom_core::Mutations, player: dom_core::Player) -> dom_core::Mutations {
    mutations.into_iter().map(|x|
        match x {
            dom_core::Mutation::RevealHandCards(p, s, r) => unimplemented!(),
            dom_core::Mutation::DrawCard(p, c) if p != player => dom_core::Mutation::DrawCard(p, None),
            other => other
        }
    ).collect()
}

fn make_action(game: &dom_core::Game, input: &str) -> Option<dom_core::Action> {
    if input == "buy" && game.state() == dom_core::State::ActionPhase {
        return Some(dom_core::Action::EndAction);
    }
    if input == "turn" {
        return Some(dom_core::Action::EndBuy);
    }
    None
}

fn main() {
    let (mut game, mutations) = dom_core::Game::new_first_game(dom_core::Players::Two);
    let mut game_p0 = dom_core::Game::from_mutations(&mutations_for_player(mutations.clone(), dom_core::Player::P0)).unwrap();
    let mut game_p1 = dom_core::Game::from_mutations(&mutations_for_player(mutations.clone(), dom_core::Player::P1)).unwrap();
//    println!("Build initial game\n{:?}\nThen using mutations\n{:?}\nBuilt perspective p0\n{:?}\nAnd perspective p1\n{:?}\n", game, mutations, game_p0, game_p1);
    loop {
        print_board_state(game.board_state());
        {
            let perspective = match game.board_state().active_player() {
                dom_core::Player::P0 => &game_p0,
                dom_core::Player::P1 => &game_p1,
                _ => panic!("Game should only have two players"),
            };
            println!("");
            println!("Game from active player perspective");
            println!("Player 1");
            show_player(perspective.board_state().get_player(dom_core::Player::P0).unwrap());
            println!("Player 2");
            show_player(perspective.board_state().get_player(dom_core::Player::P1).unwrap());
            println!("Game transition is expecting: {:?}", perspective.state());
        }
        println!("");
        println!("");
        println!("");
        println!("");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let maybe_action = match input.trim_right().trim_left() {
            "quit" => return,
            s => make_action(&game, s),
        };
        if let Some(action) = maybe_action {
            if let Some(mutations) = game.act(action) {
                let r0 = game_p0.apply_mutations(&mutations_for_player(mutations.clone(), dom_core::Player::P0));
                let r1 = game_p1.apply_mutations(&mutations_for_player(mutations.clone(), dom_core::Player::P1));
                if !r0 || !r1 {
                    panic!("Failed to apply main game mutations");
                }
            } else {
                println!("Game refused action {:?}", action);
            }
        } else {
            println!("Unknown request");
        }
    }
}
