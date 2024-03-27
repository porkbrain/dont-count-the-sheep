use Expecting::SectionKey as Sk;
use Rect2Expecting as Rect2B;
use SectionKeyBuilder as SkB;

use super::*;

pub(super) fn parse(
    state: &mut State,
    expecting: Expecting,
    s: &str,
) -> Expecting {
    match expecting {
        Sk(SkB::Region(Rect2B::X1)) => {
            Sk(SkB::Region(Rect2B::Y1(X(s.parse().unwrap()))))
        }
        Sk(SkB::Region(Rect2B::Y1(x1))) => {
            Sk(SkB::Region(Rect2B::X2(x1, Y(s.parse().unwrap()))))
        }
        Sk(SkB::Region(Rect2B::X2(x1, y1))) => {
            Sk(SkB::Region(Rect2B::Y2(x1, y1, X(s.parse().unwrap()))))
        }
        Sk(SkB::Region(Rect2B::Y2(x1, y1, x2))) => Sk(SkB::Region(
            Rect2B::ParenClose(x1, y1, x2, Y(s.parse().unwrap())),
        )),

        Sk(SkB::ZIndex) => {
            state
                .nodes
                .last_mut()
                .expect("z_index assigned to a node")
                .section_keys
                .push(SectionKey::ZIndex(Number(s.parse().unwrap())));
            Expecting::HeadingOrSectionKey
        }

        Sk(SkB::SingleAnim {
            state,
            expecting: SingleAnimExpecting::FrameNextParamValue(with_param),
        }) if with_param == "duration" => {
            assert_eq!("1.0", s, "we only support evenly spaced frames");
            Sk(SkB::SingleAnim {
                state,
                expecting: SingleAnimExpecting::FrameNextParamOrDone,
            })
        }
        Sk(SkB::SingleAnim {
            mut state,
            expecting: SingleAnimExpecting::NextParamValue(with_param),
        }) if with_param == "speed" => {
            state.speed = Number(s.parse().unwrap());
            Sk(SkB::SingleAnim {
                state,
                expecting: SingleAnimExpecting::ReadNextParamOrDone,
            })
        }

        Sk(SkB::Position(Vector2Expecting::X)) => {
            Sk(SkB::Position(Vector2Expecting::Y(X(s.parse().unwrap()))))
        }
        Sk(SkB::Position(Vector2Expecting::Y(x))) => Sk(SkB::Position(
            Vector2Expecting::ParenClose(x, Y(s.parse().unwrap())),
        )),

        Sk(SkB::FrameIndex) => {
            state
                .nodes
                .last_mut()
                .unwrap()
                .section_keys
                .push(SectionKey::FrameIndex(s.parse().unwrap()));

            Expecting::HeadingOrSectionKey
        }
        // we don't care about this value
        Sk(SkB::FrameProgress) => Expecting::HeadingOrSectionKey,

        Sk(SkB::SelfModulate(ColorExpecting::R)) => Sk(SkB::SelfModulate(
            ColorExpecting::G(Number(s.parse().unwrap())),
        )),
        Sk(SkB::SelfModulate(ColorExpecting::G(r))) => Sk(SkB::SelfModulate(
            ColorExpecting::B(r, Number(s.parse().unwrap())),
        )),
        Sk(SkB::SelfModulate(ColorExpecting::B(r, g))) => {
            Sk(SkB::SelfModulate(ColorExpecting::A(
                r,
                g,
                Number(s.parse().unwrap()),
            )))
        }
        Sk(SkB::SelfModulate(ColorExpecting::A(r, g, b))) => {
            Sk(SkB::SelfModulate(ColorExpecting::ParenClose(
                r,
                g,
                b,
                Number(s.parse().unwrap()),
            )))
        }

        _ => {
            panic!("Unexpected number {s} for {expecting:?}")
        }
    }
}
