use super::*;

pub(super) fn parse(expecting: Expecting) -> Expecting {
    match expecting {
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::NextParamColon(with_param),
        }) if with_param == "frames" => {
            Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                state,
                expecting: SingleAnimExpecting::FramesStartSquareBracket,
            })
        }
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::NextParamColon(with_param),
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::NextParamValue(with_param),
        }),
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamColon(with_param),
        }) => Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamValue(with_param),
        }),
        _ => panic!("Unexpected colon for {expecting:?}"),
    }
}
