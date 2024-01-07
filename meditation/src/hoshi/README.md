# Meditation

Help Winnie to concentrate.

Player controls character called Hoshi.
The controls are classically the global movement keys (defaults to `WASD`) and
the interact key (defaults to `Space`) to activate the special.
The interact key must be pressed in conjunction with a movement key.
The sprite should feel floaty as if you were playing Smashbros Puff after 3 Red Bulls®.

## Features

There are two debug features which help visualize the game state:
`dev` and `dev-poisson`.

## Score

Player gets points for each destroyed distraction.
The further away from the origin at `(0, 0)` the distraction is, the more points player gets.
This favours players who manage to keep the distractions away from the origin.

However, each N seconds the score is reduced slightly.
The player is in a race against time.
The total score equation now takes into account how fast the player can destroy
the distractions, not merely how long can they play for.

For example, if we deduct 100 points each 5 seconds then the player is directed
to destroying a distraction at least every ~5 seconds.
Modest goal as it is, I postulate that it will have a desirable effect on the
player's behavior.
You cannot just doodle on the screen, you have to be active.

The blue light (see section [Lighting](#lighting) below) also slows down the tempo of score reduction.

## Lightning

The light transitions between two colors.
Every time you press `Shift`, the light changes.
From bright yellowy which burns the distraction faster, to dark blue which slows the distractions down so that you can collect them more easily.
