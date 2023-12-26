# Meditation

Help Winnie to concentrate.

Player controls "weather" sprite.
The controls are WASD (or arrow keys) to move and move+space to activate the special.
The sprite should feel floaty as if you were playing Smashbros Puff after 3 Red Bulls®.

## Features

There are two debug features which help visualize the game state:
`dev` and `dev-poisson`.

## Score

Player gets points for each destroyed distraction.
The further away from the origin at `(0, 0)` the distraction is, the more points player gets.
This favours players who manage to keep the distractions away from the origin.

However, each N seconds the score is reduced slightly.
The player is in a race against the time.
The total score equation now takes into account how fast the player can destroy
the distractions, not merely how long can they play for.

For example, if we deduct 100 points each 5 seconds then the player is directed
to destroying a distraction at least every ~5 seconds.
Modest goal as it is, I postulate that it will have a desirable effect on the
player's behavior.
You cannot just doodle on the screen, you have to be active.

The blue light (see section [Lighting][#lighting] below) will also slow down the tempo of score reduction.

## Lightning

Strong light is something you'd expect in a spacy environment.
The light transitions between two colors.
Every time you enter the bubble, the light changes.
From bright yellowy which burns more, to dark blue which slows the distractions down so that you can collect them more easily.
You can stay in the middle to configure your light how you want it.
Moving across the bubble will change the color of the light from blue to yellow.
That increase cannot be taken back.

Every time there's a crack that wouldn't be without weather, do a little animation of
a lightning strike coming from weather to the crack.
Since weather only affects small distances, the lightning could always be short and not look too distorted.

## Other

The black hole should have some minimum life time.

When two screens meet, one or both of them should become 75% or 50% transparent respectively.
