As of now, bevy has no built-in editor.
Plugins are available with very simple implementations.
After experimenting with Godot, I ended up liking the editor a lot.

The decision has been made to use Godot for the editor and bevy for the game engine.
This crate parses `.tscn` files and provides a way to load them into bevy.

Everything aggressively panics.
We support very limited subset of what Godot supports, only things that are relevant to our use case.

The tree structure is parsed and converted into bevy entities.
2D nodes are entities (with relevant components) and child-parent relationships are preserved.
Plain nodes are typically components.
See the wiki for current status of what's supported and what custom nodes are available.
