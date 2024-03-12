// TODO: forbid undocumented public items

mod token;

pub use token::{parse, parse_with_conf};

#[derive(Default)]
pub struct ParseConf {}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct State {
    pub ext_resources: Vec<ExtResource>,
    pub sub_resources: Vec<SubResource>,
    nodes: Vec<()>,
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
pub enum ExtResourceAttribute {
    TypeTexture2D,
    Uid(String),
    Path(String),
    Id(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubResourceAttribute {
    TypeAtlasTexture,
    TypeSpriteFrames,
    Id(SubResourceId),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SubResourceId(pub String);

#[derive(Debug, PartialEq, Eq)]
pub enum SectionKey {
    AtlasExtResource(String),
    RegionRect2(i64, i64, i64, i64),
    SingleAnim(Animation),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub loop_: bool,
    pub name: String,
    pub speed: Fps,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AnimationFrame {
    pub texture: SubResourceId,
}

#[derive(Debug, PartialEq)]
pub struct Fps(pub f32);

impl Eq for Fps {}

impl From<String> for SubResourceId {
    fn from(s: String) -> Self {
        SubResourceId(s)
    }
}

impl Default for Animation {
    fn default() -> Self {
        Animation {
            frames: vec![],
            loop_: false,
            name: "default".to_string(),
            speed: Fps(0.0),
        }
    }
}
