use rand::Rng;
use serde::Serialize;

use crate::card::*;
use crate::deck::*;

use std::collections::HashSet;
use std::fmt::Debug;
use std::iter::IntoIterator;

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    A,
    B,
    War,
}

#[derive(Debug, Serialize)]
pub enum Winner {
    A,
    B,
}

struct Player {
    deck: Deck,
    wagered: HashSet<Card>,
    won: HashSet<Card>,
}

impl Player {
    fn blind_wager(&mut self) {
        let card = self.deck.draw().unwrap();
        self.wagered.insert(card);
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            deck: Deck::default(),
            wagered: HashSet::new(),
            won: HashSet::new(),
        }
    }
}

pub struct GameState {
    a: Player,
    b: Player,
}

impl GameState {
    pub fn deck_is_empty(&self) -> bool {
        self.a.deck.len() == 0 && self.b.deck.len() == 0
    }

    fn wager<I>(&mut self, a_wagers: I, b_wagers: I)
    where
        I: IntoIterator<Item = Card>,
    {
        for c in a_wagers {
            self.a.wagered.insert(c);
        }
        for c in b_wagers {
            self.b.wagered.insert(c);
        }
    }

    pub fn report(&self) -> String {
        format!("A:\n\tDeck: {:?}\n\tWagered: {:?}\n\tWon: {:?}\nB:\n\tDeck: {:?}\n\tWagered: {:?}\n\tWon: {:?}", self.a.deck, self.a.wagered, self.a.won, self.b.deck, self.b.wagered, self.b.won)
    }

    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        self.a.deck.shuffle(rng);
        self.b.deck.shuffle(rng);
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            a: Player::default(),
            b: Player::default(),
        }
    }
}

fn resolve(a: &Card, b: &Card) -> Outcome {
    if a.rank > b.rank {
        Outcome::A
    } else if b.rank > a.rank {
        Outcome::B
    } else {
        assert_eq!(a.rank, b.rank);
        Outcome::War
    }
}

#[derive(Serialize)]
pub struct GameLogEvent {
    pub description: String,
    pub winner: Option<Winner>,
}

impl Debug for GameLogEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.description.lines().next().unwrap_or("No events")
        )
    }
}

impl GameLogEvent {
    pub fn new() -> Self {
        Self {
            description: String::new(),
            winner: None,
        }
    }

    pub fn append(&mut self, s: &str) {
        self.description.push_str(s);
        self.description.push('\n');
    }
}

pub fn turn(gs: &mut GameState) -> GameLogEvent {
    let mut event = GameLogEvent::new();
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
            event.winner = Some(Winner::A);
        }
        Outcome::B => {
            let ai = gs.a.wagered.drain();
            let bi = gs.b.wagered.drain();
            for c in ai.chain(bi) {
                event.append(&format!("***{}", c));
                gs.b.won.insert(c);
            }
            event.winner = Some(Winner::B)
        }
        Outcome::War => {
            assert_eq!(gs.deck_is_empty(), true);
        }
    }
    event
}
