use super::*;
use crate::AnimationFrame;

pub(super) fn parse(expecting: Expecting, s: &str) -> Expecting {
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
            _ => {
                panic!("Unknown section key: {s}")
            }
        },
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::String,
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::ParenClose(s.to_string()),
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
        _ => {
            panic!("Unexpected string {s} for {expecting:?}")
        }
    }
}
