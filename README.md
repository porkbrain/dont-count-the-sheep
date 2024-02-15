# Don't Count The Sheep

![Master CI](https://github.com/porkbrain/winnie/actions/workflows/master.yml/badge.svg?branch=master)

A top down pixelart game created with [Bevy](https://bevyengine.org/).

# Organization

There are crates with common logic that help with for example [animation](common/visuals/), [input handling](common/action/), [loading screen](common/loading_screen/), [map layout, npc and player control](common/top_down/) and more in the [`common`](common/) directory.
These crates typically either export plugins or systems that one has to register themselves.

Then there's the [main game lib](main_game_lib/).
This crate exports logic that did not fit into the common crates.
For example, the `GlobalGameState` enum that directs the game flow lives here.
It also sets up default plugins and alike.

Then we have the [scenes](scenes/).
Scenes are specific game locations or minigames within the game.
The game state transitions between scenes.
The scenes will typically use the common crates or the main game lib for most work, only implementing specific logic for the scene.

# Development

Some crates export `dev` feature that enable additional debug and/or dev tooling functionality.
For example, the [`common/top_down`](common/top_down/) crate has a `dev` feature that spawns a grid of tiles to help with level design.

There's also a whole dedicated scene for prototyping and testing: [`scenes/dev_playground`](scenes/dev_playground/).
Run this scene with `$ ./bin/dev_playground`.
