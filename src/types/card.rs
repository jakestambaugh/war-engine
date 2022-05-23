use std::char;
use std::fmt;

use serde::Serialize;
use serde_with::rust::display_fromstr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Diamonds,
    Clubs,
    Hearts,
    Spades,
}

impl Suit {
    pub const fn all() -> [Suit; 4] {
        [Self::Diamonds, Self::Clubs, Self::Hearts, Self::Spades]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const fn all() -> [Rank; 13] {
        [
            Self::Two,
            Self::Three,
            Self::Four,
            Self::Five,
            Self::Six,
            Self::Seven,
            Self::Eight,
            Self::Nine,
            Self::Ten,
            Self::Jack,
            Self::Queen,
            Self::King,
            Self::Ace,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://en.wikipedia.org/wiki/Playing_cards_in_Unicode
        let base: u32 = 0x1F0A0;
        let rank_offset = match self.rank {
            Rank::Ace => 1,
            x => x as u32,
        };
        let suit_offset = match self.suit {
            Suit::Spades => 0x0,
            Suit::Hearts => 0x10,
            Suit::Diamonds => 0x20,
            Suit::Clubs => 0x30,
        };
        let value = base + rank_offset + suit_offset;
        write!(f, "{}", char::from_u32(value).unwrap())
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} of {:?}", self.rank, self.suit)
    }
}

impl Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        display_fromstr::serialize(self, serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_debug_correctly() {
        let c = Card {
            rank: Rank::Ace,
            suit: Suit::Spades,
        };
        let s = format!("{:?}", c);
        assert_eq!(s, "Ace of Spades");
    }
}
