This crates deals with the most common view in the game: the top down.
It contains logic for layout of the scenes, and the actors in them.

## `layout`

What kind of different tiles are there. There can be multiple tiles assigned to a single square.
Squares are coordinates and tiles dictate what happens on a square.
A tile is uniquely identified by `x` and `y` of the square and a layer index.

## `actor`

Moving around the pixel world, managing NPCs and the player character.
