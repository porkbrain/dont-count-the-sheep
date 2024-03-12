// TODO: forbid undocumented public items

mod token;

pub use token::{parse, parse_with_conf};

#[derive(Default)]
pub struct ParseConf {}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct State {
    pub ext_resources: Vec<ExtResource>,
    pub sub_resources: Vec<SubResource>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExtResource {
    pub attrs: Vec<ExtResourceAttribute>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubResource {
    pub attrs: Vec<SubResourceAttribute>,
    pub section_keys: Vec<SectionKey>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub attrs: Vec<NodeAttribute>,
    pub section_keys: Vec<SectionKey>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExtResourceAttribute {
    TypeTexture2D,
    Uid(String),
    Path(String),
    Id(ExtResourceId),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ExtResourceId(pub String);

#[derive(Debug, PartialEq, Eq)]
pub enum SubResourceAttribute {
    TypeAtlasTexture,
    TypeSpriteFrames,
    Id(SubResourceId),
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeAttribute {
    TypeNode2D,
    TypeSprite2D,
    TypeAnimatedSprite2D,
    TypeNode,
    Name(String),
    Parent(String),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SubResourceId(pub String);

#[derive(Debug, PartialEq, Eq)]
pub enum SectionKey {
    AtlasExtResource(ExtResourceId),
    RegionRect2(X, Y, X, Y),
    SingleAnim(Animation),
    ZIndex(Number),
    TextureExtResource(ExtResourceId),
    Position(X, Y),
    SpriteFramesSubResource(SubResourceId),
    /// key - value metadata pair where the value is of type string
    StringMetadata(String, String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub loop_: bool,
    pub name: String,
    /// FPS
    pub speed: Number,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AnimationFrame {
    pub texture: SubResourceId,
}

#[derive(Debug, PartialEq)]
pub struct Number(pub f32);

#[derive(Debug, PartialEq)]
pub struct X(pub f32);

/// The Y coordinate in godot increases as it goes down.
#[derive(Debug, PartialEq)]
pub struct Y(pub f32);

impl Eq for Number {}
impl Eq for Y {}
impl Eq for X {}

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

impl From<f32> for Number {
    fn from(f: f32) -> Self {
        Self(f)
    }
}

impl Default for Animation {
    fn default() -> Self {
        Animation {
            frames: vec![],
            loop_: false,
            name: "default".to_string(),
            speed: Number(0.0),
        }
    }
}
