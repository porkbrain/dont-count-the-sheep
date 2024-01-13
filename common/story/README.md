1. text needs to be cut up into chunks so that they fit into the dialog window
2. dialog needs to be started based on what happens in the game - pieces
   of logic will request a dialog to be started
3. different types of dialogs need to be supported, all will consume text
   at different pace and continue to the next dialog at different condition,
   some with choices, some without
4. dialog also entails sounds, animations, acquiring items, etc.
5. conversations cannot be static resources, they depend on the history and
   might use randomness
6. the seed for randomness can be global and refresh in an interval
