use super::*;

pub(super) fn parse(mut expecting: Expecting, s: &str) -> Expecting {
    let mut split = s.split('=');
    let key = split.next().expect("non empty key");
    let value = split.next().expect("non empty value");
    let value = &value[1..value.len() - 1]; // remove quotes

    match expecting {
        Expecting::ExtResourceAttributes(ref mut attrs) => {
            let attr = match (key, value) {
                ("type", "Texture2D") => ExtResourceAttribute::TypeTexture2D,
                ("uid", _) => ExtResourceAttribute::Uid(value.to_string()),
                ("path", _) => ExtResourceAttribute::Path(value.to_string()),
                ("id", _) => ExtResourceAttribute::Id(value.to_string().into()),
                _ => {
                    panic!("Unknown ExtResourceAttribute {key}={value}")
                }
            };
            attrs.push(attr);
        }
        Expecting::SubResourceAttributes(ref mut attrs) => {
            let attr = match (key, value) {
                ("type", "AtlasTexture") => {
                    SubResourceAttribute::TypeAtlasTexture
                }
                ("type", "SpriteFrames") => {
                    SubResourceAttribute::TypeSpriteFrames
                }
                ("id", _) => SubResourceAttribute::Id(value.to_string().into()),
                _ => {
                    panic!("Unknown SubResourceAttribute {key}={value}")
                }
            };
            attrs.push(attr);
        }
        Expecting::NodeAttributes(ref mut attrs) => {
            let attr = match (key, value) {
                ("type", "Node2D") => NodeAttribute::TypeNode2D,
                ("type", "Sprite2D") => NodeAttribute::TypeSprite2D,
                ("type", "AnimatedSprite2D") => {
                    NodeAttribute::TypeAnimatedSprite2D
                }
                ("type", "Node") => NodeAttribute::TypeNode,
                ("name", _) => NodeAttribute::Name(value.to_string()),
                ("parent", _) => NodeAttribute::Parent(value.to_string()),
                _ => {
                    panic!("Unknown NodeAttribute {key}={value}")
                }
            };
            attrs.push(attr);
        }
        _ => {
            panic!("Unexpected string attribute for {expecting:?}")
        }
    }

    expecting
}
