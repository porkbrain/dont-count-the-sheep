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
    Id(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SectionKey {
    AtlasExtResource(String),
    RegionRect2(i64, i64, i64, i64),
}
