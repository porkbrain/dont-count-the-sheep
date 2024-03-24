# Dialog Authoring

The dialog system consists of backend and frontend abstractions.
The backend parses dialog `.toml` files into a directed cyclic graph and traverses the graph.
Frontends are responsible for rendering the dialog and handling user input.

## Backend

The process of writing a dialog begins with a `.toml` file known as the _dialog file_.
The filename must be in [`snake_case`](wiki-snake-case) and located within the `dialogs` directory relative to the root of the game repository.

A graph comprises nodes and edges, and the dialog file represents this graph.
Nodes are represented as TOML tables.

```toml
# example syntax of two nodes

[[node]]
name = "Optional name"
en = "The English text to display"
next = "Another node"

[[node]]
name = "Another node"
en = "Another English text to display"
```

Each node possesses several keys:

- `name`: Optional key.
  Along with the file path, this creates a globally unique identifier for the node, which can be used for reference.
- `guard`: Optional key
  It must be the name of a [guard](#guards).
  If present, this key designates the node as a "guard node."
- `en`: Optional key.
  This is the English text to display.
  It must be present if `guard` is not.
  In other words, a node must have either `en` or `guard`.
  Having this key designates the node as a "vocative node."
- `params`: Optional key.
  It can only be present if `guard` is present.
  Refer to the [guards](#guards) section for further details.
- `next`: Optional key.
  It can either be a string containing the name of the next node or an array containing the names of multiple possible next nodes.
  This key represents the edges of the graph.
  Every node must be connected to at least one other node.
  If this key is absent, the next node is automatically the subsequent node in the file.
  (Sometimes, the next node is also referred to as an "outbound node.")

```toml
# example sequence of nodes where first goes always to second, but then
# we display player choice between two nodes

# notice a convention: new line if there are multiple next nodes, no new line if there is only one

[[node]]
en = "First"
[[node]]
en = "Second"
next = ["Second A", "Second B"]

[[node]]
name = "Second A"
en = "Some choice"

[[node]]
name = "Second B"
en = "Another choice"
```

Certain node names hold special significance:

- When a node with the name `_end_dialog` is reached, the dialog concludes.
- `_emerge` denotes a node whose behavior needs contextualization;
  refer to the [emerging](#emerging) section for details.
- `_root` represents the starting point of the dialog.
  Every dialog file must contain a root table `[root]`, which shares the same keys as any other node except for name.
  The name key does not apply to the root node.

```toml
# finally, we have the first example of a complete dialog file

[root]
en = "This is where the dialog starts"
[[node]]
en = "Then we go here"
next = ["Cycle", "End"]

[[node]]
name = "Cycle"
en = "This is a cycle!"
next = "_root"

[[node]]
name = "End"
en = "This is the end"
next = "_end_dialog"
```

### Guards

Guards elevate the functionality of the dialog system to new levels.
They can remember information, branch the dialog based on game state, change game state, and much more.
Some examples of what guards could do include:

- Checking if a player has a certain item
- Removing or adding items from the inventory
- Displaying UI or animations
- Checking if a player possesses a specific skill
- Altering NPC relationships with the player
- ...

Guards can accept parameters, which are passed to the guard as an [inline table][toml-inline-table].
The parameters vary from guard to guard; some guards require none.

Typically, what's crucial for a guard are the outbound nodes (nodes that the guard can lead to).

Here's a list of supported guards with their parameters and descriptions of their functions with respect to outbound nodes:

#### `exhaustive_alternatives`

No parameters.
Associate it with multiple outbound nodes, and each time the dialog reaches this node, it will proceed to the next node in the list.
When the last node is reached, this guard will prevent the dialog from revisiting this branch.
If you assign a name to this node, it will remember its state, so even if the player exits the dialog and returns to this node, it will resume from where it left off.
If left unnamed, once the dialog concludes, it will start again from the first node the next time the dialog initiates.

```toml
# example

[[node]]
name = "start"
en = "Start here"
# when player selects B for the first time, shows alt 1
# for the second time, shows alt 2
# for the third time, shows alt 3
# after that it will only show "A"
next = ["A", "B"]

[[node]]
name = "A"
next = "_end_dialog"

[[node]]
name = "B"
guard = "exhaustive_alternatives"
next = ["alt 1", "alt 2", "alt 3"]

[[node]]
name = "alt 1"
en = "This is the first alternative"
next = "start"

[[node]]
name = "alt 2"
en = "This is the second alternative"
next = "start"

[[node]]
name = "alt 3"
en = "This is the third alternative"
next = "start"
```

#### `reach_last_alternative`

Similar to [`exhaustive_alternatives`](#exhaustive_alternatives), but once the last alternative is reached, it will always show the last alternative from that point onward.

#### `add_dialog_to_npc`

Accepts two parameters: `npc` and `file_path`:

```toml
params = { npc = "name of the NPC", file_path = "your_dialog_file.toml" }
```

The `npc` param is optional and defaults to the currently speaking NPC.
Must be present if the player is currently speaking.

Next time the player speaks to the NPC, the dialog from the specified file will be added to the NPC's dialog.

#### `remove_dialog_from_npc`

Same parameters as [`add_dialog_to_npc`](#add_dialog_to_npc).

The dialog file will not be attached to the NPC anymore.

### Emerging

TODO

### Variables

TODO

## Frontend

### Portrait dialog

A frontend that pauses player movement.
It can be employed for dialogues with NPCs or for cutscenes.
This frontend displays the portrait of the character who is speaking.

<!-- List of References -->

[wiki-snake-case]: https://en.wikipedia.org/wiki/Snake_case
[toml-inline-table]: https://toml.io/en/v1.0.0#inline-table
