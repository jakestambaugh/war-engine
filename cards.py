from enum import Enum, auto
from typing import Callable


class Suit(Enum):
    DIAMONDS = "Diamonds"
    CLUBS = "Clubs"
    HEARTS = "Hearts"
    SPADES = "Spades"


class Card:
    def __init__(self, name: str, suit: Suit, weight: int, function: Callable):
        self.name = name
        self.suit = suit
        self.weight = weight
        self.function = function

    def __repr__(self):
        return f"{self.name} of {self.suit.value}"


def nothing(mine: Card, theirs: Card):
    pass


def create_library():
    library = []
    for x in range(2, 15):
        name = str(x)
        if x < 11:
            pass
        elif x == 11:
            name = "Jack"
        elif x == 12:
            name = "Queen"
        elif x == 13:
            name = "King"
        elif x == 14:
            name = "Ace"

        for suit in Suit:
            card = Card(name, suit, x, nothing)
            library.append(card)
    return library
