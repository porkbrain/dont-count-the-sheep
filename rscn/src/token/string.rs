use super::*;

pub(super) fn parse(expecting: Expecting, s: &str) -> Expecting {
    match expecting {
        Expecting::HeadingOrSectionKey => match s {
            "atlas" => Expecting::SectionKey(SectionKeyBuilder::Atlas(
                ExtResourceBuilderExpecting::ExtResource,
            )),
            "region" => Expecting::SectionKey(SectionKeyBuilder::Region(
                Rect2BuilderExpecting::Rect2,
            )),
            _ => {
                panic!("Unknown section key: {s}")
            }
        },
        Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::String,
        )) => Expecting::SectionKey(SectionKeyBuilder::Atlas(
            ExtResourceBuilderExpecting::EndQuote(s.to_string()),
        )),
        _ => {
            panic!("Unexpected string {s} for {expecting:?}")
        }
    }
}
