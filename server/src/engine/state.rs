use crate::types::card::*;
use crate::types::deck::*;
use rand::Rng;

use std::collections::HashSet;
use std::iter::IntoIterator;

#[derive(Default)]
pub struct PlayerState {
    pub deck: Deck,
    pub wagered: HashSet<Card>,
    pub won: HashSet<Card>,
}

impl PlayerState {
    pub fn blind_wager(&mut self) {
        let card = self.deck.draw().unwrap();
        self.wagered.insert(card);
    }
}

#[derive(Default)]
pub struct GameState {
    pub a: PlayerState,
    pub b: PlayerState,
}

impl GameState {
    pub fn deck_is_empty(&self) -> bool {
        self.a.deck.is_empty() && self.b.deck.is_empty()
    }

    pub fn wager<I>(&mut self, a_wagers: I, b_wagers: I)
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
