//! While we have [Value] enum that can represent any .tscn value, we know what
//! specific values to expect from Godot.
//!
//! In this module we declare specific expectations we have for the .tscn values
//! that Godot produces.

use std::collections::BTreeMap;

use miette::LabeledSpan;

use super::value::SpannedValue;

/// Represents a parsed Godot scene.
#[derive(Default, Debug)]
pub struct Scene {
    /// Headers are attributes of the initial "gd_scene" section.
    pub headers: BTreeMap<String, SpannedValue>,
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
        attributes: BTreeMap<String, SpannedValue>,
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
    pub section: BTreeMap<SubResourceSectionKey, SpannedValue>,
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
    pub section: BTreeMap<NodeSectionKey, SpannedValue>,
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
    /// e.g. `atlas = ExtResource("12_um7ei")`
    AtlasExtResource,
    /// e.g. `region = Rect2(0, 0, 32, 62)`
    Region,
    /// e.g.
    /// ```text
    /// animations = [{
    ///     "frames": [{
    ///         "duration": 1.0,
    ///         "texture": SubResource("AtlasTexture_yvafp")
    ///     }, {
    ///         "duration": 1.0,
    ///         "texture": SubResource("AtlasTexture_l80js")
    ///     }],
    ///     "loop": true,
    ///     "name": &"default",
    ///     "speed": 5.0
    /// }]
    /// ```
    Animations,
    /// Catch all for any other kind of key.
    Other(String),
}

/// Section keys we expect in the `[nodes]` section.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeSectionKey {
    /// e.g. `z_index = -3`
    ZIndex,
    /// e.g. `texture = ExtResource("10_kymjx")`
    TextureExtResource,
    /// e.g. `position = Vector2(-23, 14)`
    Position,
    /// e.g. `sprite_frames = SubResource("SpriteFrames_ns3ui")`
    SpriteFrames,
    /// e.g.
    /// ```
    /// metadata/key1 = "A"
    /// metadata/key2 = "B"
    /// metadata/anotherkey = "C"
    /// ```
    StringMetadata,
    /// e.g. `frame = 17`
    FrameIndex,
    /// e.g. `autoplay = "default"`
    Autoplay,
    /// `e.g. visible = false`
    Visible,
    /// e.g. `flip_h = true`
    FlipHorizontally,
    /// e.g. `flip_v = true`
    FlipVertically,
    /// e.g. `self_modulate = Color(1, 1, 1, 0.823529)`
    SelfModulate,
    /// e.g. `frame_progress = 0.00329857`
    FrameProgress,
    /// Catch all for any other kind of key.
    Other(String),
}

/// Represents an animation in a sprite frames resource.
#[derive(Debug)]
pub struct SpriteFramesAnimation {
    /// Each frame is presented by some texture resource and duration of the
    /// frame in parts of the speed.
    /// Ie. you need to sum up all durations of the frames to get `SUM`, then
    /// the duration of the frame is `duration / SUM * speed`.
    /// E.g.
    /// ```text
    /// frames = [{
    ///    "duration": 2.0,
    ///   "texture": SubResource("A")
    /// }, {
    ///   "duration": 1.0,
    ///  "texture": SubResource("B")
    /// }, {
    ///   "duration": 1.0,
    ///  "texture": SubResource("C")
    /// }]
    ///
    /// and speed = 5.0, then
    ///
    /// frame 1: 2.0 / 4.0 * 5.0 = 2.5
    /// frame 2: 1.0 / 4.0 * 5.0 = 1.25
    /// frame 3: 1.0 / 4.0 * 5.0 = 1.25
    /// ```
    pub frames: Vec<(SubResourceId, f32)>,
    /// Whether the animation should loop indefinitely.
    pub loop_: bool,
    /// The name of the animation, defaults to "default".
    pub name: String,
    /// The speed of the animation.
    pub speed: f32,
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
    pub fn into_vector2(self) -> miette::Result<(f64, f64)> {
        let [x, y] = self.try_into_this_class_of_len("Vector2")?;

        let (_, x) = x.try_into_number()?;
        let (_, y) = y.try_into_number()?;

        Ok((x, y))
    }

    /// Interprets the value as a "ExtResource" class.
    pub fn try_into_ext_resource(self) -> miette::Result<ExtResourceId> {
        let [id] = self.try_into_this_class_of_len("ExtResource")?;
        let (_, id) = id.try_into_string()?;
        Ok(ExtResourceId(id))
    }

    /// Interprets the value as a "SubResource" class.
    pub fn try_into_sub_resource(self) -> miette::Result<SubResourceId> {
        let [id] = self.try_into_this_class_of_len("SubResource")?;
        let (_, id) = id.try_into_string()?;
        Ok(SubResourceId(id))
    }

    /// Interprets the value as a "Rect2" class as a tuple of `(x, y, w, h)`.
    pub fn try_into_rect2(self) -> miette::Result<(f64, f64, f64, f64)> {
        let [x, y, w, h] = self.try_into_this_class_of_len("Rect2")?;

        let (_, x) = x.try_into_number()?;
        let (_, y) = y.try_into_number()?;
        let (_, w) = w.try_into_number()?;
        let (_, h) = h.try_into_number()?;

        Ok((x, y, w, h))
    }

    /// Interprets the value as an array of sprite frame animations.
    /// Value part of [SubResourceSectionKey::Animations].
    pub fn try_into_sprite_frames_animations(
        self,
    ) -> miette::Result<Vec<SpriteFramesAnimation>> {
        let (_, animations) = self.try_into_array()?;

        animations
            .into_iter()
            .map(|v| {
                let (span, mut v) = v.try_into_object()?;

                let name = v
                    .remove("name")
                    .map(|n| n.try_into_string().map(|(_, s)| s))
                    .transpose()?
                    .unwrap_or_else(|| "default".to_owned());
                let speed = v
                    .remove("speed")
                    .map(|s| s.try_into_number().map(|(_, n)| n as _))
                    .transpose()?
                    .unwrap_or(1.0);
                let loop_ = v
                    .remove("loop")
                    .map(|l| l.try_into_bool().map(|(_, b)| b))
                    .transpose()?
                    .unwrap_or_default();
                let (_, frames) = v
                    .remove("frames")
                    .ok_or_else(|| {
                        miette::miette! {
                            labels = vec![
                                LabeledSpan::at(span.clone(), "in this object"),
                            ],
                            "Expected key 'frames'",
                        }
                    })?
                    .try_into_array()?;
                let frames = frames.into_iter().map(|f| {
                    let (span, mut f) = f.try_into_object()?;
                    let texture = f
                        .remove("texture")
                        .map(|t| t.try_into_sub_resource())
                        .transpose()?
                        .ok_or_else(|| {
                            miette::miette! {
                                labels = vec![
                                    LabeledSpan::at(span.clone(), "in this object"),
                                ],
                                "Expected key 'texture'",
                            }
                        })?;
                    let duration = f
                        .remove("duration")
                        .map(|d| d.try_into_number().map(|(_, n)| n))
                        .transpose()?
                        .ok_or_else(|| {
                            miette::miette! {
                                labels = vec![
                                    LabeledSpan::at(span.clone(), "in this object"),
                                ],
                                "Expected key 'duration'",
                            }
                        })?;
                    Ok((texture, duration as _))
                }).collect::<miette::Result<Vec<_>>>()?;

                Ok(SpriteFramesAnimation {
                    name,
                    speed,
                    loop_,
                    frames,
                })
            })
            .collect()
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
