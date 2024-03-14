#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) struct State {
    pub(crate) ext_resources: Vec<ParsedExtResource>,
    pub(crate) sub_resources: Vec<ParsedSubResource>,
    pub(crate) nodes: Vec<ParsedNode>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedExtResource {
    pub(crate) kind: ExtResourceKind,
    pub(crate) path: String,
    pub(crate) id: ExtResourceId,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedSubResource {
    pub(crate) id: SubResourceId,
    pub(crate) kind: SubResourceKind,
    pub(crate) section_keys: Vec<SectionKey>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedNode {
    pub(crate) name: String,
    pub(crate) parent: Option<String>,
    pub(crate) kind: ParsedNodeKind,
    pub(crate) section_keys: Vec<SectionKey>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ExtResourceKind {
    Texture2D,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExtResourceId(pub(crate) String);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SubResourceKind {
    AtlasTexture,
    SpriteFrames,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParsedNodeKind {
    Node,
    Node2D,
    Sprite2D,
    AnimatedSprite2D,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct SubResourceId(pub(crate) String);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SectionKey {
    AtlasExtResource(ExtResourceId),
    RegionRect2(X, Y, X, Y),
    SingleAnim(Animation),
    ZIndex(Number),
    TextureExtResource(ExtResourceId),
    Position(X, Y),
    SpriteFramesSubResource(SubResourceId),
    /// key - value metadata pair where the value is of type string
    StringMetadata(String, String),
    FrameIndex(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Animation {
    pub(crate) frames: Vec<AnimationFrame>,
    pub(crate) loop_: bool,
    pub(crate) name: String,
    /// FPS
    pub(crate) speed: Number,
    pub(crate) autoload: bool,
    pub(crate) index: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AnimationFrame {
    pub(crate) texture: SubResourceId,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Number(pub(crate) f32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct X(pub(crate) f32);

/// The Y coordinate in godot increases as it goes down.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Y(pub(crate) f32);

impl Y {
    /// This is the conversion from godot to bevy coordinates.
    /// Note that not all Y coords should be converted.
    /// For example sprite atlas positions into textures in bevy follow the
    /// image processing convention where the origin is at the top left.
    pub(crate) fn into_bevy_position_coords(self) -> f32 {
        -self.0
    }
}

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

impl From<Number> for f32 {
    fn from(Number(n): Number) -> Self {
        n
    }
}

impl Default for Animation {
    fn default() -> Self {
        Animation {
            name: "default".to_string(),
            speed: Number(0.0),
            frames: vec![],
            index: 0,
            loop_: false,
            autoload: false,
        }
    }
}
