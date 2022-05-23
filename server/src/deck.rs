use crate::card::{Card, Rank, Suit};
use rand::{prelude::SliceRandom, Rng};
use std::{fmt::Debug, vec::Vec};

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
    }

    /// Returns the top card of the deck if the deck is not empty
    /// Returns `None` otherwise
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for Deck {
    fn default() -> Self {
        let mut c: Vec<Card> = Vec::new();
        for s in Suit::all() {
            for r in Rank::all() {
                c.push(Card { suit: s, rank: r })
            }
        }
        Deck { cards: c }
    }
}

impl Debug for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let strs: Vec<String> = self.cards.iter().map(|c| c.to_string()).collect();
        write!(f, "{:?}", strs)
    }
}

impl<'a> std::iter::IntoIterator for &'a Deck {
    type Item = <std::slice::Iter<'a, Card> as Iterator>::Item;
    type IntoIter = std::slice::Iter<'a, Card>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.as_slice().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_52_cards() {
        let d = Deck::default();
        assert_eq!(52, d.cards.len())
    }

    #[test]
    fn len_shows_52_cards() {
        let d = Deck::default();
        assert_eq!(52, d.len())
    }
}
