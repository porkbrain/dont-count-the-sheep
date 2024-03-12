use super::*;

pub(super) fn parse(expecting: Expecting, s: &str) -> Expecting {
    match expecting {
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamValue(with_param),
        }) if with_param == "duration" => {
            assert_eq!("1.0", s, "we only support evenly spaced frames");
            Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                state,
                expecting: SingleAnimExpecting::FrameNextParamOrDone,
            })
        }
        Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
            mut state,
            expecting: SingleAnimExpecting::NextParamValue(with_param),
        }) if with_param == "speed" => {
            state.speed = Fps(s.parse().unwrap());
            Expecting::SectionKey(SectionKeyBuilder::SingleAnim {
                state,
                expecting: SingleAnimExpecting::ReadNextParamOrDone,
            })
        }

        Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::Float1,
        )) => Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::Float2(X(s.parse().unwrap())),
        )),
        Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::Float2(x),
        )) => Expecting::SectionKey(SectionKeyBuilder::Position(
            Vector2Expecting::ParenClose(x, Y(s.parse().unwrap())),
        )),

        _ => panic!("Unexpected float for {expecting:?}"),
    }
}
