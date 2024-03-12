use Expecting::SectionKey as Sk;
use Rect2BuilderExpecting as Rect2B;
use SectionKeyBuilder as SkB;

use super::*;

pub(super) fn parse(expecting: Expecting, s: &str) -> Expecting {
    match expecting {
        Sk(SkB::Region(Rect2B::Int1)) => {
            Sk(SkB::Region(Rect2B::Int2(s.parse().unwrap())))
        }
        Sk(SkB::Region(Rect2B::Int2(int1))) => {
            Sk(SkB::Region(Rect2B::Int3(int1, s.parse().unwrap())))
        }
        Sk(SkB::Region(Rect2B::Int3(int1, int2))) => {
            Sk(SkB::Region(Rect2B::Int4(int1, int2, s.parse().unwrap())))
        }
        Sk(SkB::Region(Rect2B::Int4(int1, int2, int3))) => Sk(SkB::Region(
            Rect2B::ParenClose(int1, int2, int3, s.parse().unwrap()),
        )),
        _ => {
            panic!("Unexpected int {s} for {expecting:?}")
        }
    }
}
