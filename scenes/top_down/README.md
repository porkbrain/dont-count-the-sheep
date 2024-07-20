Adds all top down scenes to the game.
Lots of the top down scene logic is in the `main_game_lib` crate because it tends to be referenced also by other systems.
In package we have scene specific logic rather than the shared one.
