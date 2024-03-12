use super::*;
use crate::AnimationFrame;

pub(super) fn parse(
    state: &mut State,
    expecting: Expecting,
    s: &str,
) -> Expecting {
    let s = s.trim_matches('"');

    match expecting {
        Expecting::HeadingOrSectionKey => match s {
            "atlas" => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                ExtResourceExpecting::ExtResource,
            )),
            "region" => Expecting::SectionKey(SectionKeyBuilder::Region(
                Rect2Expecting::Rect2,
            )),
            "animations" => {
                Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                    state: Animation::default(),
                    expecting: SingleAnimExpecting::default(),
                })
            }
            "z_index" => Expecting::SectionKey(SectionKeyBuilder::ZIndex),
            "texture" => Expecting::SectionKey(SectionKeyBuilder::Texture(
                Default::default(),
            )),
            "position" => Expecting::SectionKey(SectionKeyBuilder::Position(
                Default::default(),
            )),
            "sprite_frames" => Expecting::SectionKey(
                SectionKeyBuilder::SpriteFrames(Default::default()),
            ),
            s if s.starts_with("metadata/") => Expecting::SectionKey(
                SectionKeyBuilder::Metadata(s["metadata/".len()..].to_string()),
            ),
            _ => {
                panic!("Unknown section key: '{s}' for {expecting:?}")
            }
        },

        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::String,
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::ParenClose(s.to_string()),
        )),
        Expecting::SectionKey(SectionKeyBuilder::Texture(
            ExtResourceExpecting::String,
        )) => Expecting::SectionKey(SectionKeyBuilder::Texture(
            ExtResourceExpecting::ParenClose(s.to_string()),
        )),
        Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
            SubResourceExpecting::String,
        )) => Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
            SubResourceExpecting::ParenClose(s.to_string()),
        )),

        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::ReadNextParamOrDone,
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::NextParamColon(s.to_string()),
        }),
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamOrDone,
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamColon(s.to_string()),
        }),
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            mut state,
            expecting: SingleAnimExpecting::FrameNextParamValue(with_param),
        }) if with_param == "texture" => {
            state.frames.push(AnimationFrame {
                texture: s.to_string().into(),
            });
            Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                state,
                expecting: SingleAnimExpecting::FrameNextParamOrDone,
            })
        }
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            mut state,
            expecting: SingleAnimExpecting::NextParamValue(with_param),
        }) if with_param == "name" => {
            state.name = s.to_string();
            Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                state,
                expecting: SingleAnimExpecting::ReadNextParamOrDone,
            })
        }
        Expecting::SectionKey(SectionKeyBuilder::Metadata(with_param)) => {
            state
                .nodes
                .last_mut()
                .expect("node to come before metadata")
                .section_keys
                .push(SectionKey::StringMetadata(with_param, s.to_string()));
            Expecting::HeadingOrSectionKey
        }

        _ => {
            panic!("Unexpected string {s} for {expecting:?}")
        }
    }
}
