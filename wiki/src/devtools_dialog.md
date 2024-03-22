# Writing dialog

The dialog system comprises [backend](#backend) and [frontend](#frontend) abstractions.
Backend parses dialog `.toml` files we write into a directed cyclic graph and traverses the graph.
Frontends render the dialog and handle user input.

## Backend

The journey of writing a dialog starts with a `.toml` file called _dialog file_.
The name of the file must be in [`snake_case`](wiki-snake-case) and must reside in `dialogs` directory (with respect to game repository root.)
A graph is a collection of nodes and edges.
The dialog file represents a graph.
We represent nodes as TOML tables.

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

Each node has some keys:

- `name` is an optional key.
  Along with the file path this will create a globally unique identifier for the node.
  Nodes can be referred to by this name.
- `guard` is an optional key.
  It must be a name of a [guard](#guards).
  Having this key makes the node a _guard node_.
- `en` is an optional key.
  It is the English text to display.
  It must be present if `guard` is not present.
  In another words, a node must have either `en` or `guard`.
  Having this key makes the node a _vocative node_.
- `params` is an optional key.
  It can be present only if `guard` is present.
  See the [guards](#guards) section for more information.
- `next` is an optional key.
  It's either a string with the name of the next node or an array with the names of multiple possible next nodes.
  This key represents the edges of the graph.
  Each node must be connected to some other node.
  If this key is not present, the next node is automatically the next node in the file.
  (Sometimes next node is also called an _outbound node_.)

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

There are some special node names:

- When a node with name `_end_dialog` is reached, the dialog ends.
- `_emerge` is a node whose behavior must be put into context, see [this](#emerging) section.
- `_root` is a node that represents the starting point of the dialog.
  Each dialog file must have a root table `[root]`.
  This table will have the same keys as any other node with the exception of `name`.
  The name of the root node is always `_root`.

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

Guards extend the functionality of the dialog system into new heights.
Guards can remember, branch the dialog based on game state, change game state and much more.
Some example of what guards could/can do:

- check if player has some item
- remove or add items from inventory
- display some UI or animation
- check if player has some skill
- alter NPC relationship with the player
- whatever you need

Guards can accept parameters.
The parameters are passed to the guard as an [inline table][toml-inline-table].
The parameters differ from guard to guard, some have none.

Typically what's important for a guard are the outbound nodes (nodes that the guard can lead to).

Here's a list of supported guards with their parameters and description of what they do with the outbound nodes:

#### `exhaustive_alternatives`

No parameters.
Associate it with multiple outbound nodes and every time the dialog reaches this node, it will show the next node in the list.
When the last node is reached, this guard will prevent the dialog from showing the branch again.
When you give this node a name, it remembers the state and even if the player exists the dialog and comes back to this node, it will pick up where it left off.
If you don't name it, once the dialog is over, it will start from the first node again next time the dialog starts.

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

Similar to [`exhaustive_alternatives`](#exhaustive_alternatives), but once the last alternative is reached it will always shows the last alternative from here onwards.

### Emerging

TODO

### Variables

TODO

## Frontend

### Portrait dialog

Frontend that pauses player movement.
Can be used for dialog with NPCs or for cutscenes.
Displays the portrait of who is speaking.

<!-- List of References -->

[wiki-snake-case]: https://en.wikipedia.org/wiki/Snake_case
[toml-inline-table]: https://toml.io/en/v1.0.0#inline-table
