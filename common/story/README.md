We store dialogs in `.toml` format.
Dialog is a directed graph.
Each node of the graph is a dialog state.
Nodes can either be vocative, meaning that they are a part of the dialog, or they can guards, meaning that they condition the transition between the nodes and/or mutate global game state.

See the wiki for more information on the keys each node can have and the supported guards.
