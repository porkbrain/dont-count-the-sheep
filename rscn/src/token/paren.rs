use super::*;

pub(super) fn parse_open(expecting: Expecting) -> Expecting {
    match expecting {
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::StartQuote,
        )),
        Expecting::SectionKey(SectionKeyBuilder::Region(
            Rect2BuilderExpecting::ParenOpen,
        )) => Expecting::SectionKey(SectionKeyBuilder::Region(
            Rect2BuilderExpecting::Int1,
        )),
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
            ExtResourceBuilderExpecting::ParenClose(with_str),
        )) => {
            state
                .sub_resources
                .last_mut()
                .expect("sub resource to come before section key")
                .section_keys
                .push(SectionKey::AtlasExtResource(with_str));
            Expecting::HeadingOrSectionKey
        }
        Expecting::SectionKey(SectionKeyBuilder::Region(
            Rect2BuilderExpecting::ParenClose(int1, int2, int3, int4),
        )) => {
            state
                .sub_resources
                .last_mut()
                .expect("sub resource to come before section key")
                .section_keys
                .push(SectionKey::RegionRect2(int1, int2, int3, int4));
            Expecting::HeadingOrSectionKey
        }
        _ => {
            panic!("Unexpected paren close for {expecting:?}")
        }
    }
}
