pub mod input;
pub mod log;
pub mod machine;
pub mod state;

use serde::Serialize;

use crate::types::card::*;

use std::cmp::Ordering;
use std::fmt::Debug;

use self::state::GameState;

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub enum Outcome {
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

pub fn turn(gs: &mut GameState) {
    let a: Card = gs.a.deck.draw().unwrap();
    let b: Card = gs.b.deck.draw().unwrap();

    gs.log(log::GameLogEvent::DrewCard(log::PlayerId::A, a));
    gs.log(log::GameLogEvent::DrewCard(log::PlayerId::B, b));

    let mut outcome = resolve(&a, &b);
    gs.log(log::GameLogEvent::ComparedMatch(log::Comparison {
        a_card: a,
        b_card: b,
    }));
    gs.log(log::GameLogEvent::ResolvedMatch(outcome.clone()));

    gs.wager([a], [b]);
    gs.log(log::GameLogEvent::WageredVisible(log::Wager {
        player: log::PlayerId::A,
        cards: vec![a],
    }));
    gs.log(log::GameLogEvent::WageredVisible(log::Wager {
        player: log::PlayerId::B,
        cards: vec![b],
    }));

    while outcome == Outcome::War && !gs.deck_is_empty() {
        // war_wager
        let mut wagered = 0;
        let mut a_w = Vec::new();
        let mut b_w = Vec::new();
        while gs.a.deck.len() > 1 && gs.b.deck.len() > 1 && wagered < 3 {
            a_w.push(gs.a.blind_wager());
            b_w.push(gs.b.blind_wager());
            wagered += 1;
        }

        gs.log(log::GameLogEvent::WageredHidden(log::Wager {
            player: log::PlayerId::A,
            cards: a_w,
        }));
        gs.log(log::GameLogEvent::WageredHidden(log::Wager {
            player: log::PlayerId::B,
            cards: b_w,
        }));

        let a: Card = gs.a.deck.draw().unwrap();
        let b: Card = gs.b.deck.draw().unwrap();

        gs.log(log::GameLogEvent::DrewCard(log::PlayerId::A, a));
        gs.log(log::GameLogEvent::DrewCard(log::PlayerId::B, b));

        outcome = resolve(&a, &b);
        gs.log(log::GameLogEvent::ComparedMatch(log::Comparison {
            a_card: a,
            b_card: b,
        }));
        gs.log(log::GameLogEvent::ResolvedMatch(outcome.clone()));

        gs.wager([a], [b]);
        gs.log(log::GameLogEvent::WageredVisible(log::Wager {
            player: log::PlayerId::A,
            cards: vec![a],
        }));
        gs.log(log::GameLogEvent::WageredVisible(log::Wager {
            player: log::PlayerId::B,
            cards: vec![b],
        }));
    }
    match &outcome {
        Outcome::A => {
            let ai = gs.a.wagered.drain();
            let bi = gs.b.wagered.drain();
            let winnings: Vec<Card> = ai.chain(bi).collect();

            gs.a.won.extend(winnings.iter());
            gs.log(log::GameLogEvent::ClaimedWager(log::Wager {
                player: log::PlayerId::A,
                cards: winnings,
            }));
        }
        Outcome::B => {
            let ai = gs.a.wagered.drain();
            let bi = gs.b.wagered.drain();
            let winnings: Vec<Card> = ai.chain(bi).collect();

            gs.b.won.extend(winnings.iter());
            gs.log(log::GameLogEvent::ClaimedWager(log::Wager {
                player: log::PlayerId::B,
                cards: winnings,
            }));
        }
        Outcome::War => {
            assert!(gs.deck_is_empty());
            gs.log(log::GameLogEvent::GameEndedInWar)
        }
    }
}
