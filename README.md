# Don't Count The Sheep

![Master CI](https://github.com/porkbrain/winnie/actions/workflows/master.yml/badge.svg?branch=master)

A top down pixelart game created with [Bevy](https://bevyengine.org/).

# Repo organization

There are crates in the [`common`](common/) directory that help with [animation](common/visuals/), [input handling](common/action/), [loading screen](common/loading_screen/), [map layout, npc and player control](common/top_down/) and more.
These crates typically either export plugins or systems that one has to register themselves.

Then there's the [main game lib](main_game_lib/).
This crate exports logic that did not fit into the common crates.
For example, the `GlobalGameState` enum that directs the game flow lives here.
It also sets up default plugins and alike.

Then we have the [scenes](scenes/).
Scenes are specific game locations or minigames within the game.
The game state transitions between scenes.
The scenes will typically use the common crates or the main game lib for most work, only implementing specific logic for the scene.

We use some external dependencies.
It's paramount that we keep the bevy related dependencies to a minimum with the current Bevy release schedule.
With every extra dependency that also depends on Bevy it potentially takes longer to start upgrading.

- [`bevy_pixel_camera`][bevy_pixel_camera]
- [`bevy_webp_anim`][bevy_webp_anim] is a crate we maintain so not a problem
- [`bevy-inspector-egui`][bevy-inspector-egui]
- [`leafwing-input-manager`][leafwing-input-manager] is maintained by a core Bevy contributor
- [`bevy_magic_light`](bevy_magic_light/) is a fork of [this][original-bevy_magic_light] we maintain but ideally we'd like to remove it in favor of Bevy's official 2D lighting solution
- [`bevy_egui`][bevy_egui] is a dependency of `bevy-inspector-egui` and `leafwing-input-manager`

[bevy_pixel_camera]: https://github.com/drakmaniso/bevy_pixel_camera
[bevy_webp_anim]: https://github.com/bausano/bevy-webp-anim
[bevy-inspector-egui]: https://github.com/jakobhellermann/bevy-inspector-egui
[leafwing-input-manager]: https://github.com/Leafwing-Studios/leafwing-input-manager
[original-bevy_magic_light]: https://github.com/zaycev/bevy-magic-light-2d
[bevy_egui]: https://github.com/mvlabat/bevy_egui

# Dev environment

Some crates export `dev` feature that enable additional debug and/or dev tooling functionality.
For example, the [`common/top_down`](common/top_down/) crate has a `dev` feature that spawns a grid of tiles to help with level design.

There's also a whole dedicated scene for prototyping and testing: [`scenes/dev_playground`](scenes/dev_playground/).
Run this scene with `$ ./bin/dev_playground`.
