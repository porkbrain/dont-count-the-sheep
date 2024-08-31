# Don't Count The Sheep

![Master CI](https://github.com/porkbrain/winnie/actions/workflows/master.yml/badge.svg?branch=master)

A top down pixelart game created with [Bevy](https://bevyengine.org/).

Run `$ ./dev/wiki` to open the game's [wiki](wiki/README.md) locally in your browser.

Our primary color is `#0d0e1f`.

## Dependencies

- [Rust][rust-install]
- [Bevy][bevy-install]
- `apt install lld clang` for [faster builds][bevy-fast-compile]
- [Graphviz][graphviz-install]

## Repo organization

There are crates in the [`common`](common/) directory that help with [animation](common/visuals/), [input handling](common/action/), [loading screen](common/loading_screen/), and more.
These crates typically either export plugins or systems that one has to register themselves.

Then there's the [main game lib](main_game_lib/).
This crate exports logic that did not fit into the common crates.
For example, the `GlobalGameState` enum that directs the game flow lives here, or map layout, npc and player control.
It also sets up default plugins and alike.

Then we have the [scenes](scenes/).
Scenes are main menu, top down view, minigames within the game etc.
The game state transitions between scenes.
The scenes will typically use the common crates or the main game lib for most work, only implementing specific logic for the scene.

We use some external dependencies.
It's paramount that we keep the bevy related dependencies to a minimum with the current Bevy release schedule.
With every extra dependency that also depends on Bevy it potentially takes longer to start upgrading.

- [`bevy_pixel_camera`][bevy_pixel_camera] is used to scale the game to a pixel art resolution. This plugin seems to be no longer maintained and we will vendor it soon
- [`bevy_webp_anim`][bevy_webp_anim] is a crate we maintain so not a problem
- [`leafwing-input-manager`][leafwing-input-manager] is maintained by a core Bevy contributor
- [`bevy_kira_audio`][bevy_kira_audio] is used for audio. This decision was made based on some discord conversations that suggested it was better than the native Bevy audio plugin
- [`bevy_egui`][bevy_egui] and [`bevy-inspector-egui`][bevy-inspector-egui] are used for devtools

## Dev environment

Some crates export `devtools` feature that enable additional debug and/or dev tooling functionality: `$ cargo run --features devtools`

<!-- List of references -->

[bevy_egui]: https://github.com/mvlabat/bevy_egui
[bevy_kira_audio]: https://github.com/NiklasEi/bevy_kira_audio
[bevy_pixel_camera]: https://github.com/drakmaniso/bevy_pixel_camera
[bevy_webp_anim]: https://github.com/bausano/bevy-webp-anim
[bevy-inspector-egui]: https://github.com/jakobhellermann/bevy-inspector-egui
[leafwing-input-manager]: https://github.com/Leafwing-Studios/leafwing-input-manager
[original-bevy_magic_light]: https://github.com/zaycev/bevy-magic-light-2d
[rust-install]: https://www.rust-lang.org/tools/install
[bevy-install]: https://bevyengine.org/learn/quick-start/getting-started/setup
[graphviz-install]: https://graphviz.org/download
[bevy-fast-compile]: https://bevyengine.org/learn/quick-start/getting-started/setup/#enable-fast-compiles-optional
