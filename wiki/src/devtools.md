# Using Godot Scene Maker

Since Bevy does not have a built-in editor, we utilize Godot to create scenes.
Godot exports each scene into a text file using its custom configuration format [`.tscn`][godot-tscn].
We parse these `.tscn` files into a representation that can be spawned into a Bevy world.
It's important to note that we currently support only a limited subset of what a Godot scene (and `.tscn` files by extension) can contain.
Any unexpected input is handled with aggressive error reporting.

Here's a brief overview of the Godot editor:

![Godot editor overview](assets/godot-editor-overview.png)

1. Each scene in Godot includes a root node.
   For our purposes, we use the `Node2D` type and name it after the specific scene.
2. Both Bevy and 2D Godot are centered at `(0px, 0px)`.
3. The root node's children consist of images and atlases rendered in the scene.
   While nodes can be nested arbitrarily, it's recommended to keep the scene flat for [ysorting](#ysorting).
   We support the following:
   - `Sprite2D` for a single image
   - `AnimatedSprite2D` for an [atlas](#atlases)
   - `Node2D` for other positional data like waypoints or [ysorting](#ysorting)
   - `Node` for [components](#components)
4. Godot utilizes the same assets as Bevy.
5. Positioning 2D nodes can be done by dragging them or through numeric input.
   Sometimes, to align objects perfectly, you may need to use numeric input to adjust by half a pixel.
   However, rotation, scale, and skew are not supported.
6. Z-index determines the rendering order, with higher values indicating closer proximity to the camera.
   This input sets the z-index to a static number, with values equal to or smaller than -1, or equal to or larger than 2.
   The interval `(-1, 2)` is reserved for [ysorting](#ysorting).
7. The `Node` type is utilized for [specific in-game logic](#components).
   It doesn't have a position and can only contain metadata.
8. Metadata can be added and modified here.
   Refer to the [documentation](#components) for the relevant component to understand the required metadata.
9. [`YSort`](#ysorting) is the name of a `Node2D` that's a child of either `Sprite2D` or `AnimatedSprite2D`.
   Any node with this child must have its z-index set to 0.
   Typically, it's positioned at the bottom of the sprite.
   As the name suggests, only the y-coordinate is used to determine the rendering order.
   - **a.** demonstrates the child node
   - **b.** shows how it can be positioned

## Atlases

Godot utilizes `AnimatedSprite2D` for atlases.
When creating this node, navigate to the _Inspector_ in the left panel, and under the _Animation_ section, add a new _Sprite Frames_.

10. This action opens a window where you can load an image and divide it into frames.
11. Adjusts the frames per second (FPS).
    This setting is only relevant if auto-play is enabled (see below).
12. Specifies whether the animation loops.
    By default, once the animation finishes, it remains on the last frame.
13. Automatically plays the animation.

## Ysorting

Ysorting is a technique to render sprites in a top-down scene.
It's a way to ensure that sprites are rendered in the correct order based on their y-coordinate.

To apply ysorting to a node, the y-index must be set to **0** on the sprite node (default) and a new child node of type `Node2D` named `YSort` must be added.
Ysorting only works for sibling nodes.

## Components

The purpose of components is to enhance the functionality of sprites.
There are scene-specific components and others that can be utilized across multiple scenes.

Components sometimes require metadata to specify or alter their behavior.

![Godot editor metadata](assets/godot-editor-metadata.png)

1. Add a new metadata in the _Inspector_
2. A quirk of Godot is that it capitalizes the first letter of the key in the editor, although it stores it in the text file as it is.
   For this reason, metadata keys are not case-sensitive, but the values are case-sensitive.
3. Metadata values are always strings.

Here's an exhaustive list of components (defined as `Node` type with their names) and their metadata:

- `InspectLabel`: Defines interaction with an in-game object.
  See also [_Inspecting the World_](ability_to_inspect.md).
  This component is limited to [top-down scenes](top_down.md).

  - `label`: The text to display on the object.
  - `action`: An optional name of the event to fire when the object is interacted with.
  - `zone`: An optional name of a local tile kind that the player must be in to interact with the object.
  - `category`: Optional. See [_Inspecting the World: Categories_](ability_to_inspect.md#categories).

- `HallwayEntity`: A component with no metadata in the `Apartment` scene.

<!-- List of References -->

[godot-tscn]: https://docs.godotengine.org/en/stable/contributing/development/file_formats/tscn.html
