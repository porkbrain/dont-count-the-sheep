use super::*;

pub(super) fn parse(expecting: Expecting) -> Expecting {
    match expecting {
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::StartQuote,
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::String,
        )),
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::EndQuote(with_str),
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::ParenClose(with_str),
        )),
        _ => {
            panic!("Unexpected quote for {expecting:?}")
        }
    }
}
