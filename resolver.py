from cards import *
from enum import Enum, auto
from typing import Sequence

import itertools


class LenGen(object):
    def __init__(self, gen, length):
        self.gen = gen
        self.n = 0
        self.max = length

    def __call__(self):
        return itertools.islice(self.gen(), self.length)

    def __len__(self):
        return self.max - self.n

    def __next__(self):
        self.n += 1
        return next(self.gen)


class Outcome(Enum):
    A = auto()
    B = auto()
    WAR = auto()


def resolve(a: Card, b: Card) -> Outcome:
    if a.weight > b.weight:
        return Outcome.A
    elif b.weight > a.weight:
        return Outcome.B
    else:
        assert a.weight == b.weight
        return Outcome.WAR


def play_match(a_deck: Sequence[Card], b_deck: Sequence[Card]):
    a_vs_b = LenGen(zip(a_deck, b_deck), len(a_deck))
    prizes_a = []
    prizes_b = []
    while len(a_vs_b) > 0:
        a, b = next(a_vs_b)
        outcome = resolve(a, b)
        print(f"{a.name} vs {b.name}: {outcome}")
        wagered_a = [a]
        wagered_b = [b]
        while outcome == Outcome.WAR and len(a_vs_b) > 0:
            wagered = 0
            while len(a_vs_b) > 1 and wagered < 3:
                wa, wb = next(a_vs_b)
                print(f"-- {wa} and {wb}")
                wagered_a.append(wa)
                wagered_b.append(wb)
                wagered += 1
            a, b = next(a_vs_b)
            outcome = resolve(a, b)
            wagered_a.append(a)
            wagered_b.append(b)
            print(f"{a.name} vs {b.name}: {outcome}")
        if outcome == Outcome.A:
            prizes_a.extend(wagered_b)
            print(f"* A won {wagered_b}")
        elif outcome == Outcome.B:
            prizes_b.extend(wagered_a)
            print(f"* B won {wagered_a}")
        else:
            assert outcome == Outcome.WAR
            assert len(a_vs_b) == 0
            print("Finished the deck in a war")
    print(f"{len(prizes_a)} prizes for A")
    print(f"{len(prizes_b)} prizes for B")
