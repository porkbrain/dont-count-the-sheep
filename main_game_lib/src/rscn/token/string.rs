use bevy::utils::default;

use super::*;

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
            "texture" => {
                Expecting::SectionKey(SectionKeyBuilder::Texture(default()))
            }
            "position" => {
                Expecting::SectionKey(SectionKeyBuilder::Position(default()))
            }
            "sprite_frames" => Expecting::SectionKey(
                SectionKeyBuilder::SpriteFrames(default()),
            ),
            "frame" => Expecting::SectionKey(SectionKeyBuilder::FrameIndex),
            "frame_progress" => {
                Expecting::SectionKey(SectionKeyBuilder::FrameProgress)
            }
            "autoplay" => Expecting::SectionKey(SectionKeyBuilder::Autoplay),
            "visible" => Expecting::SectionKey(SectionKeyBuilder::Visibility),
            "self_modulate" => Expecting::SectionKey(
                SectionKeyBuilder::SelfModulate(default()),
            ),
            "flip_h" => {
                Expecting::SectionKey(SectionKeyBuilder::FlipHorizontally)
            }
            "flip_v" => {
                Expecting::SectionKey(SectionKeyBuilder::FlipVertically)
            }
            s if s.starts_with("metadata/") => {
                Expecting::SectionKey(SectionKeyBuilder::StringMetadata(
                    s["metadata/".len()..].to_ascii_lowercase(),
                ))
            }
            _ => {
                panic!("Unknown section key: '{s}' for {expecting:?}")
            }
        },

        Expecting::SectionKey(SectionKeyBuilder::Autoplay) => {
            assert_eq!(
                "default", s,
                "Name of atlas animation must be 'default' not '{s}'"
            );
            state
                .nodes
                .last_mut()
                .unwrap()
                .section_keys
                .push(SectionKey::Autoplay);

            Expecting::HeadingOrSectionKey
        }

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
        Expecting::SectionKey(SectionKeyBuilder::StringMetadata(
            with_param,
        )) => {
            state
                .nodes
                .last_mut()
                .expect("node to come before metadata")
                .section_keys
                .push(SectionKey::StringMetadata(with_param, s.to_string()));
            Expecting::HeadingOrSectionKey
        }

        Expecting::SectionKey(SectionKeyBuilder::SelfModulate(
            ColorExpecting::Color,
        )) => Expecting::SectionKey(SectionKeyBuilder::SelfModulate(
            ColorExpecting::ParenOpen,
        )),

        _ => {
            panic!("Unexpected string {s} for {expecting:?}")
        }
    }
}
