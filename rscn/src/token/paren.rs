use super::*;

pub(super) fn parse_open(expecting: Expecting) -> Expecting {
    match expecting {
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::String,
        )),
        Expecting::SectionKey(SectionKeyBuilder::Texture(
            ExtResourceExpecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::Texture(
            ExtResourceExpecting::String,
        )),
        Expecting::SectionKey(SectionKeyBuilder::Region(
            Rect2Expecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::Region(
            Rect2Expecting::Int1,
        )),
        Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::Float1,
        )),
        Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
            SubResourceExpecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
            SubResourceExpecting::String,
        )),
        // just forward to the next token
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamValue(with_param),
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamValue(with_param),
        }),
        _ => {
            panic!("Unexpected paren open for {expecting:?}")
        }
    }
}

pub(super) fn parse_close(
    state: &mut State,
    expecting: Expecting,
) -> Expecting {
    match expecting {
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceExpecting::ParenClose(with_str),
        )) => {
            state
                .sub_resources
                .last_mut()
                .expect("sub resource to come before section key")
                .section_keys
                .push(SectionKey::AtlasExtResource(with_str.into()));
            Expecting::HeadingOrSectionKey
        }
        Expecting::SectionKey(SectionKeyBuilder::Region(
            Rect2Expecting::ParenClose(int1, int2, int3, int4),
        )) => {
            state
                .sub_resources
                .last_mut()
                .expect("sub resource to come before section key")
                .section_keys
                .push(SectionKey::RegionRect2(int1, int2, int3, int4));
            Expecting::HeadingOrSectionKey
        }

        Expecting::SectionKey(SectionKeyBuilder::Texture(
            ExtResourceExpecting::ParenClose(with_str),
        )) => {
            state
                .nodes
                .last_mut()
                .expect("node to come before section key")
                .section_keys
                .push(SectionKey::TextureExtResource(with_str.into()));
            Expecting::HeadingOrSectionKey
        }
        Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::ParenClose(x, y),
        )) => {
            state
                .nodes
                .last_mut()
                .expect("node to come before section key")
                .section_keys
                .push(SectionKey::Position(x, y));
            Expecting::HeadingOrSectionKey
        }
        Expecting::SectionKey(SectionKeyBuilder::SpriteFrames(
            SubResourceExpecting::ParenClose(with_str),
        )) => {
            state
                .nodes
                .last_mut()
                .expect("node to come before section key")
                .section_keys
                .push(SectionKey::SpriteFramesSubResource(with_str.into()));
            Expecting::HeadingOrSectionKey
        }

        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamOrDone,
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamOrDone,
        }),
        _ => {
            panic!("Unexpected paren close for {expecting:?}")
        }
    }
}
