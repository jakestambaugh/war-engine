from cards import *
from resolver import *

from random import shuffle

a_deck = create_library()
b_deck = create_library()

shuffle(a_deck)
shuffle(b_deck)

# play_match(a_deck, b_deck)

c_deck = create_library()
d_deck = create_library()

play_match(c_deck, d_deck)
