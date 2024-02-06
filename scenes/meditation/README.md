# Meditation

Help Winnie to concentrate.

Player controls a character called Hoshi.
Hoshi protects Winnie from Polpos that are trying to distract her.
The closer Hoshi is to a Polpo, the higher chance of it cracking (aka taking damage) or, if fully cracked up, destroying it.

The chance of cracking a Polpo is proportional to the distance between Hoshi and the Polpo and whether a climate's light ray is being cast on the Polpo.

## Controls

Hoshi should feel floaty as if one was playing Smashbros Puff after 3 Red Bulls®.

The controls are classically the global movement keys (defaults to `WASD`) and
the interact key (defaults to `Space`) to activate the special.
The interact key must be pressed in conjunction with a movement key.

## Score

Player gets points for each destroyed Polpo.
The further away from the origin at `(0, 0)` the Polpo is, the more points player gets.
This favours players who manage to keep the Polpos away from the origin.

However, each N seconds the score is reduced slightly.
The player is in a race against time.
The total score equation now takes into account how fast the player can destroy
the Polpos, not merely how long can they play for.

For example, if we deduct 100 points each 5 seconds then the player is directed
to destroying a Polpo at least every ~5 seconds.
Modest goal as it is, I postulate that it will have a desirable effect on the
player's behavior.
The player cannot just doodle on the screen, they have to be active.

The blue light (see section [Lighting](#lighting) below) slows down the tempo of score reduction.
The red light speeds it up.

## Lightning

The climate has several rotating light rays.
The light transitions between two colors.
Every time the player activates the special, the light changes color.
From bright yellowy which burns the Polpo faster, to dark blue which slows the Polpos down so that the player can collect them more easily.

## Cargo features

There are two debug features which help visualize the game state:
`dev` and `dev-poisson`.
