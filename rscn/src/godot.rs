//! While we have [Value] enum that can represent any .tscn value, we know what
//! specific values to expect from Godot.
//!
//! In this module we declare specific expectations we have for the .tscn values
//! that Godot produces.

use super::value::{Map, SpannedValue};

/// Represents a parsed Godot scene.
#[derive(Default, Debug, PartialEq)]
pub struct Scene {
    /// Headers are attributes of the initial "gd_scene" section.
    pub headers: Map<String, SpannedValue>,
    /// List of `[ext_resources]`.
    pub ext_resources: Vec<ExtResource>,
    /// List of `[sub_resources]`.
    pub sub_resources: Vec<SubResource>,
    /// List of `[nodes]`.
    pub nodes: Vec<Node>,
}

/// The kind of external resources we expect in the .tscn file.
///
/// `[ext_resources]`
#[derive(Debug, PartialEq)]
#[allow(missing_docs)]
pub enum ExtResource {
    /// A 2D texture.
    Texture2D { uid: ExtResourceId, path: String },
    /// Catch all for any other kind of resource.
    Other {
        kind: String,
        uid: ExtResourceId,
        attributes: Map<String, SpannedValue>,
    },
}

/// `[sub_resources]`
#[derive(Debug, PartialEq)]
pub struct SubResource {
    /// The unique identifier of the sub resource.
    pub id: SubResourceId,
    /// If we know resource type, then we can use it to interpret the
    /// `section_keys`.
    pub kind: SubResourceKind,
    /// The keys and values of the sub resource.
    pub section_keys: Map<SubResourceSectionKey, SpannedValue>,
}

/// Represents Godot node tree.
///
/// `[nodes]`
#[derive(Debug, PartialEq)]
pub struct Node {
    /// The name of the node as shown in the Godot editor.
    pub name: String,
    /// Will be [None] for the root node.
    pub parent: Option<String>,
    /// There are many kinds of nodes in Godot, some of them we care about in
    /// this crate.
    pub kind: NodeKind,
    /// The keys and values of the node.
    ///
    /// Nested keys are mapped to a map value.
    pub section_keys: Map<NodeSectionKey, SpannedValue>,
}

/// The kind of external resources we expect in the .tscn file.
#[derive(Debug, PartialEq, Eq)]
pub enum ExtResourceKind {
    /// A 2D texture resource.
    Texture2D,
    /// Catch all for any other kind of resource.
    Other(String),
}

/// The unique identifier of an external resource.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ExtResourceId(pub String);

/// The kind of sub resources we expect in the .tscn file.
#[derive(Debug, PartialEq, Eq)]
pub enum SubResourceKind {
    /// A texture atlas.
    AtlasTexture,
    /// A sprite frames resource.
    SpriteFrames,
    /// Catch all for any other kind of resource.
    Other(String),
}

/// The kind of node we expect in the .tscn file.
#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// The top level node.
    Node,
    /// A 2D node.
    Node2D,
    /// A 2D node that is a sprite node.
    Sprite2D,
    /// A 2D node that is an animated sprite node.
    AnimatedSprite2D,
    /// Catch all for any other kind of node.
    Other(String),
}

/// The unique identifier of a sub resource.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SubResourceId(pub String);

/// Section keys we expect in the `[sub_resources]` section.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubResourceSectionKey {
    /// `AtlasExtResource(ExtResourceId)`
    AtlasExtResource,
    /// `RegionRect2(X, Y, X, Y)`
    Region,
    /// `SingleAnim(Animation)`
    Animations,
    /// Catch all for any other kind of key.
    Other(String),
}

/// Section keys we expect in the `[nodes]` section.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeSectionKey {
    /// `ZIndex(Number)`
    ZIndex,
    /// `TextureExtResource(ExtResourceId)`
    TextureExtResource,
    /// `Position(X, Y)`
    Position,
    /// `SpriteFramesSubResource(SubResourceId)`
    SpriteFrames,
    /// key - value metadata pair where the value is of type string
    /// (String, String)`
    StringMetadata,
    /// `FrameIndex(Number)`
    FrameIndex,
    /// Whether the atlas should autoplay the animation.
    Autoplay,
    /// Whether the node is visible.
    /// If false we add a `Visibility::Hidden` component to the node.
    /// `Visibility(bool)`
    Visible,
    /// A texture should be flipped horizontally.
    /// `FlipHorizontally(bool)`
    FlipHorizontally,
    /// A texture should be flipped vertically.
    /// `FlipVertically(bool)`
    FlipVertically,
    /// RGBa
    /// `SelfModulateColor(Number, Number, Number, Number)`
    SelfModulate,
    /// `FrameProgress(Number)`
    FrameProgress,
    /// Catch all for any other kind of key.
    Other(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Animation {
    pub(crate) frames: Vec<AnimationFrame>,
    pub(crate) loop_: bool,
    pub(crate) name: String,
    /// FPS
    // pub(crate) speed: Number,
    pub(crate) autoload: bool,
    pub(crate) index: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AnimationFrame {
    pub(crate) texture: SubResourceId,
}

impl ExtResource {
    /// The unique identifier of the external resource is always present,
    /// irrelevant of the kind.
    pub fn uid(&self) -> &ExtResourceId {
        match self {
            Self::Texture2D { uid, .. } => uid,
            Self::Other { uid, .. } => uid,
        }
    }
}

impl SpannedValue {
    /// Interprets the value as a "Color" class.
    ///
    /// This could be achieved more modularly by some sort of generic system.
    pub fn into_self_modulate_color_rgba(
        self,
    ) -> miette::Result<(f64, f64, f64, f64)> {
        let [r, g, b, a] = self.try_into_this_class_of_len("Color")?;

        let (_, r) = r.try_into_number()?;
        let (_, g) = g.try_into_number()?;
        let (_, b) = b.try_into_number()?;
        let (_, a) = a.try_into_number()?;

        Ok((r, g, b, a))
    }

    /// Interprets the value as a "Vector2" class.
    ///
    /// This could be achieved more modularly by some sort of generic system.
    pub fn into_vector2(self) -> miette::Result<(f64, f64)> {
        let [x, y] = self.try_into_this_class_of_len("Vector2")?;

        let (_, x) = x.try_into_number()?;
        let (_, y) = y.try_into_number()?;

        Ok((x, y))
    }
}

impl From<String> for SubResourceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<String> for ExtResourceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<String> for SubResourceKind {
    fn from(s: String) -> Self {
        match s.as_str() {
            "AtlasTexture" => Self::AtlasTexture,
            "SpriteFrames" => Self::SpriteFrames,
            _ => Self::Other(s),
        }
    }
}

impl From<String> for NodeKind {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Node" => Self::Node,
            "Node2D" => Self::Node2D,
            "Sprite2D" => Self::Sprite2D,
            "AnimatedSprite2D" => Self::AnimatedSprite2D,
            _ => Self::Other(s),
        }
    }
}

// impl Default for Animation {
//     fn default() -> Self {
//         Animation {
//             name: "default".to_string(),
//             speed: Number(0.0),
//             frames: vec![],
//             index: 0,
//             loop_: false,
//             autoload: false,
//         }
//     }
// }

impl From<String> for NodeSectionKey {
    fn from(s: String) -> Self {
        match s.as_str() {
            "z_index" => Self::ZIndex,
            "texture" => Self::TextureExtResource,
            "position" => Self::Position,
            "sprite_frames" => Self::SpriteFrames,
            "metadata" => Self::StringMetadata,
            "frame" => Self::FrameIndex,
            "frame_progress" => Self::FrameProgress,
            "autoplay" => Self::Autoplay,
            "visible" => Self::Visible,
            "flip_h" => Self::FlipHorizontally,
            "flip_v" => Self::FlipVertically,
            "self_modulate" => Self::SelfModulate,
            _ => Self::Other(s),
        }
    }
}

impl From<String> for SubResourceSectionKey {
    fn from(s: String) -> Self {
        match s.as_str() {
            "atlas" => Self::AtlasExtResource,
            "region" => Self::Region,
            "animations" => Self::Animations,
            _ => Self::Other(s),
        }
    }
}
