pub mod log;
pub mod machine;
pub mod state;

use crate::types::card::*;

use std::cmp::Ordering;
use std::fmt::Debug;

use self::state::GameState;

enum PlayerId {
    A,
    B,
}

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    A,
    B,
    War,
}

fn resolve(a: &Card, b: &Card) -> Outcome {
    match a.rank.cmp(&b.rank) {
        Ordering::Greater => Outcome::A,
        Ordering::Less => Outcome::B,
        Ordering::Equal => Outcome::War,
    }
}

struct GameInput {}

pub fn turn(gs: &mut GameState) -> log::GameLogEvent {
    let mut event = log::GameLogEvent::new();
    let a: Card = gs.a.deck.draw().unwrap();
    let b: Card = gs.b.deck.draw().unwrap();

    let mut outcome = resolve(&a, &b);

    gs.wager([a], [b]);
    event.append(&format!("{} vs {}: {:?}", a, b, outcome));

    while outcome == Outcome::War && !gs.deck_is_empty() {
        let a: Card = gs.a.deck.draw().unwrap();
        let b: Card = gs.b.deck.draw().unwrap();

        // war_wager
        let mut wagered = 0;
        while gs.a.deck.len() > 1 && gs.b.deck.len() > 1 && wagered < 3 {
            gs.a.blind_wager();
            gs.b.blind_wager();
            wagered += 1;
        }

        event.append(&format!("--- A wagered: {:?}", gs.a.wagered));
        event.append(&format!("--- B wagered: {:?}", gs.b.wagered));
        outcome = resolve(&a, &b);
        event.append(&format!("{} vs {}: {:?}", a, b, outcome));
    }
    match &outcome {
        Outcome::A => {
            let ai = gs.a.wagered.drain();
            let bi = gs.b.wagered.drain();
            for c in ai.chain(bi) {
                event.append(&format!("***{}", c));
                gs.a.won.insert(c);
            }
            event.a_wins()
        }
        Outcome::B => {
            let ai = gs.a.wagered.drain();
            let bi = gs.b.wagered.drain();
            for c in ai.chain(bi) {
                event.append(&format!("***{}", c));
                gs.b.won.insert(c);
            }
            event.b_wins()
        }
        Outcome::War => {
            assert!(gs.deck_is_empty());
        }
    }
    event
}
