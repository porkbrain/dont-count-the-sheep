use super::*;

pub(super) fn parse(mut expecting: Expecting, s: &str) -> Expecting {
    let mut split = s.split('=');
    let key = split.next().expect("non empty key");
    let value = split.next().expect("non empty value");
    let value = &value[1..value.len() - 1]; // remove quotes

    match expecting {
        Expecting::ExtResourceAttributes {
            ref mut id,
            ref mut kind,
            ref mut path,
        } => {
            match (key, value) {
                ("type", "Texture2D") => {
                    assert!(kind.replace(ExtResourceKind::Texture2D).is_none());
                }
                ("path", _) => {
                    assert!(path.replace(value.to_string()).is_none());
                }
                ("id", _) => {
                    assert!(id
                        .replace(ExtResourceId(value.to_string().into()))
                        .is_none());
                }
                // we don't care
                ("uid", _) => {}
                _ => {
                    panic!("Unknown ExtResourceAttribute {key}={value}")
                }
            };
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
        Expecting::NodeAttributes {
            ref mut name,
            ref mut parent,
            ref mut kind,
        } => {
            match (key, value) {
                // each attr can be present only once, hence the assertions
                ("type", "Node2D") => {
                    assert!(kind.replace(ParsedNodeKind::Node2D).is_none())
                }
                ("type", "Sprite2D") => {
                    assert!(kind.replace(ParsedNodeKind::Sprite2D).is_none())
                }
                ("type", "AnimatedSprite2D") => {
                    assert!(kind
                        .replace(ParsedNodeKind::AnimatedSprite2D)
                        .is_none())
                }
                ("type", "Node") => {
                    assert!(kind.replace(ParsedNodeKind::Node).is_none())
                }
                ("name", _) => {
                    assert!(name.replace(value.to_string()).is_none())
                }
                ("parent", _) => {
                    assert!(parent.replace(value.to_string()).is_none())
                }
                _ => {
                    panic!("Unknown NodeAttribute {key}={value}")
                }
            };
        }
        _ => {
            panic!("Unexpected string attribute for {expecting:?}")
        }
    }

    expecting
}
