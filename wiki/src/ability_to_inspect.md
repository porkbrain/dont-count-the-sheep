# Inspecting the World

Interacting with various objects and NPCs is a common action in any top-down scene.

Our goals are:

- You must understand the outcome of initiating an interaction by pressing the respective button.
- You should have clarity regarding what is within your reach.
- You shouldn't need to reposition yourself to interact with an object when multiple objects are nearby.

We considered alternative solutions:

- Highlighting the object that would be interacted with when you are close to it. However, this does not provide clarity about your surroundings and does not eliminate the need for repositioning.
- Having an emoji above your character's head to indicate the available type of interaction. However, this suffers from the same drawbacks as the previous solution.

Our preferred solution is to display text labels on objects and NPCs.

![Diablo example with highlighted items](assets/diablo-inspect-items.png)

In many games, there is a special button to press, typically `Alt`, to activate the inspection mode.
Monika likes the idea of not having the inspection mode always on, while Michael is annoyed when he has to press a button to see what's around him.

![Songs of Conquest example with highlighted locations](assets/songs-of-conquest-inspect-world.png)

A natural solution is to gamify the inspection mode, making it feel less like a game setting and more like a game mechanic.

The inspection mode can be leveled up.
When you observe an object or NPC for the first time, they are highlighted.
This increases your curiosity level, allowing you to see farther.

There are multiple categories for observable objects.
For example, you can observe plants, garbage, and so on.
Each category has its own radius, and the curiosity level is combined with other factors to determine the radius.
There are gadgets that can be attached to the [phone](phone.md) that can alter the inspection mode.

To avoid repositioning, we highlight the object that is closest to you.
That is the object you will interact with if you press the interaction button.
However, you can also change the highlighted object with directional input.
Pressing up changes selection to the next object closest to the highlighted object in the upward direction.
Allowing you to change the highlighted object you will interact with helps to avoid a bug where you cannot interact with an object because it's behind another object.
