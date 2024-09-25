# `rscn`

This library serves as lexer and parser for Godot's [.tscn][tscn-format] file format.

> Why we use Godot as editor is justified in our [wiki](../wiki).

It is aware of some common Godot constructs such as

- what sections a .tscn file has: `[gd_scene]`, `[ext_resource]`, `[sub_resource]` and `[node]`
- what sorts of values can appear, such as `Color`, `Vector2`, etc.

<!-- List of Links -->

[tscn-format]: https://docs.godotengine.org/en/stable/contributing/development/file_formats/tscn.html
