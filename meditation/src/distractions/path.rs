//! Contains Bezier curve points for the various paths distractions can take.
//!
//! ```text
//! t = top
//! b = bottom
//! r = right
//! l = left
//! c = center
//! ```
//!
//! I made these by hand on an imaginary 128x72 grid.
//! That's because the size of the game is 640x360 (5 times larger).
//! Then I scale them up by 4 to get the final result.

use bevy::math::cubic_splines::CubicCurve;

use crate::prelude::*;
use LevelPath::*;

/// Scales up from an imaginary 128x72 grid to the game's 640x360 grid.
const F: f32 = 4.65;

pub(crate) enum LevelPath {
    IntroA,
    IntroB,
    Lvl1A,
    Lvl1B,
    FromLvl1AToLvl2,
    // TODO: from lvl2a to lvl2
    Lvl2A,
}

impl LevelPath {
    pub(crate) fn random_intro() -> Self {
        if rand::random() {
            IntroA
        } else {
            IntroB
        }
    }

    /// Schedules the next level path.
    /// If `should_level_up` is `true`, the next level path will be more
    /// proximate to the center of the screen.
    pub(crate) fn transition_into(&self, should_level_up: bool) -> Self {
        match self {
            IntroA => Lvl1A,
            IntroB => Lvl1B,
            Lvl1A | Lvl1B if should_level_up => FromLvl1AToLvl2,
            Lvl1A => Lvl1B,
            Lvl1B => Lvl1A,
            FromLvl1AToLvl2 => Lvl2A,
            Lvl2A if should_level_up => {
                // TODO: what happens next?
                Lvl2A
            }
            Lvl2A => Lvl2A,
        }
    }

    pub(crate) fn curve(&self) -> &'static CubicCurve<Vec2> {
        match self {
            IntroA => &intro_a::CURVE,
            IntroB => &intro_b::CURVE,
            Lvl1A => &lvl1_a::CURVE,
            Lvl1B => &lvl1_b::CURVE,
            FromLvl1AToLvl2 => &from_lvl1a_to_lvl2::CURVE,
            Lvl2A => &lvl2_a::CURVE,
        }
    }

    pub(crate) fn segments(&self) -> &'static [CubicSegment<Vec2>] {
        self.curve().segments()
    }

    pub(crate) fn segment_timing(&self) -> &'static [f32] {
        match self {
            IntroA => &intro_a::SEGMENT_TIMING,
            IntroB => &intro_b::SEGMENT_TIMING,
            Lvl1A => &lvl1_a::SEGMENT_TIMING,
            Lvl1B => &lvl1_b::SEGMENT_TIMING,
            FromLvl1AToLvl2 => &from_lvl1a_to_lvl2::SEGMENT_TIMING,
            Lvl2A => &lvl2_a::SEGMENT_TIMING,
        }
    }

    pub(crate) fn total_path_time(&self) -> f32 {
        match self {
            IntroA => intro_a::TOTAL_PATH_TIME,
            IntroB => intro_b::TOTAL_PATH_TIME,
            Lvl1A => lvl1_a::TOTAL_PATH_TIME,
            Lvl1B => lvl1_b::TOTAL_PATH_TIME,
            FromLvl1AToLvl2 => from_lvl1a_to_lvl2::TOTAL_PATH_TIME,
            Lvl2A => lvl2_a::TOTAL_PATH_TIME,
        }
    }

    /// Returns the current segment and how much of it has been traversed.
    pub(crate) fn segment(&self, elapsed: &Duration) -> (usize, f32) {
        // total path time, as path repeats once all segments have been
        // traversed
        let total_t = elapsed.as_secs_f32() % self.total_path_time();

        // now calculate how much of the current segment has been traversed by
        // 1. finding the current segment
        // 2. finding finding how much is left
        // 3. finding the length of the current segment
        // 4. dividing 2. by 3. to get the percentage of the segment that has
        //    been
        // traversed

        let timing = self.segment_timing();

        // 1.
        let (seg_index, seg_ends_at) = timing
            .iter()
            .enumerate()
            .find(|(_, seg_t)| total_t < **seg_t)
            .map(|(i, seg_t)| (i, *seg_t))
            .unwrap_or((timing.len() - 1, timing[timing.len() - 1]));
        // 2.
        let seg_remaining = seg_ends_at - total_t;
        // 3.
        let seg_length = if seg_index == 0 {
            timing[0]
        } else {
            timing[seg_index] - timing[seg_index - 1]
        };
        // 4.
        let seg_t = 1.0 - (seg_remaining / seg_length);

        (seg_index, seg_t)
    }
}

mod intro_a {
    use lazy_static::lazy_static;

    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 1] = [10.0];
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    lazy_static! {
        pub(super) static ref CURVE: CubicCurve<Vec2> = curve();
    }

    fn curve() -> CubicCurve<Vec2> {
        let start = vec2(-128.0, 72.0) * F;
        let control1 = vec2(-112.0, 64.0) * F;
        let control2 = vec2(-88.0, 48.0) * F;
        let t_l = vec2(-64.0, 36.0) * F;

        let path = vec![[start, control1, control2, t_l]];
        debug_assert_eq!(path.len(), SEGMENT_TIMING.len());

        CubicBezier::new(path).to_curve()
    }
}

mod intro_b {
    use lazy_static::lazy_static;

    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 1] = [10.0];
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    lazy_static! {
        pub(super) static ref CURVE: CubicCurve<Vec2> = curve();
    }

    fn curve() -> CubicCurve<Vec2> {
        let start = vec2(-128.0, 0.0) * F;
        let control1 = vec2(-112.0, 12.0) * F;
        let control2 = vec2(-88.0, 24.0) * F;
        let t_l = vec2(-64.0, 36.0) * F;

        let path = vec![[start, control1, control2, t_l]];
        debug_assert_eq!(path.len(), SEGMENT_TIMING.len());

        CubicBezier::new(path).to_curve()
    }
}

mod lvl1_a {
    use lazy_static::lazy_static;

    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 6] = {
        let f = 7.0;
        [1.1 * f, 1.5 * f, 1.9 * f, 2.2 * f, 3.4 * f, 4.0 * f]
    };
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    lazy_static! {
        pub(super) static ref TOP_LEFT: Vec2 = vec2(-64.0, 36.0) * F;
        pub(super) static ref CURVE: CubicCurve<Vec2> = curve();
    }

    fn curve() -> CubicCurve<Vec2> {
        let l_control2 = vec2(-55.0, 23.0) * F;
        let t_l = *TOP_LEFT;
        let t_control1 = vec2(-24.0, 25.0) * F;

        let t_control2 = vec2(24.0, 42.0) * F;
        let t_r = vec2(64.0, 36.0) * F;
        let r_spiral_in_control1 = vec2(70.0, 16.0) * F;

        let r_spiral_in_control2 = vec2(54.0, 6.0) * F;
        let r_spiral_in = vec2(45.0, 12.0) * F;
        let r_spiral_out_control1 = vec2(56.0, 24.0) * F;

        let r_spiral_out_control2 = vec2(68.0, 0.0) * F;
        let r_spiral_out = vec2(64.0, -12.0) * F;
        let r_control1 = vec2(54.0, -20.0) * F;

        let r_control2 = vec2(72.0, -28.0) * F;
        let b_r = vec2(64.0, -36.0) * F;
        let b_control1 = vec2(24.0, -40.0) * F;

        let b_control2 = vec2(-36.0, -40.0) * F;
        let b_l = vec2(-64.0, -36.0) * F;
        let l_control1 = vec2(-68.0, -20.0) * F;

        let path = vec![
            [t_l, t_control1, t_control2, t_r],
            [t_r, r_spiral_in_control1, r_spiral_in_control2, r_spiral_in],
            [
                r_spiral_in,
                r_spiral_out_control1,
                r_spiral_out_control2,
                r_spiral_out,
            ],
            [r_spiral_out, r_control1, r_control2, b_r],
            [b_r, b_control1, b_control2, b_l],
            [b_l, l_control1, l_control2, t_l],
        ];
        debug_assert_eq!(path.len(), SEGMENT_TIMING.len());

        CubicBezier::new(path).to_curve()
    }
}

mod lvl1_b {
    use lazy_static::lazy_static;

    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 5] = {
        let f = 7.0;
        [1.3 * f, 1.75 * f, 2.3 * f, 3.4 * f, 4.0 * f]
    };
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    lazy_static! {
        pub(super) static ref CURVE: CubicCurve<Vec2> = curve();
    }

    fn curve() -> CubicCurve<Vec2> {
        let l_control2 = vec2(-72.0, 23.0) * F;
        let t_l = vec2(-64.0, 36.0) * F;
        let t_control1 = vec2(-24.0, 35.0) * F;

        let t_control2 = vec2(24.0, 42.0) * F;
        let t_r = vec2(62.0, 40.0) * F;
        let c_r_in_control1 = vec2(70.0, 28.0) * F;

        let c_r_in_control2 = vec2(64.0, 6.0) * F;
        let c_r = vec2(72.0, 0.0) * F;
        let c_r_out_control1 = vec2(54.0, -6.0) * F;

        let c_r_out_control2 = vec2(72.0, -28.0) * F;
        let b_r = vec2(64.0, -36.0) * F;
        let b_control1 = vec2(24.0, -40.0) * F;

        let b_control2 = vec2(-36.0, -43.0) * F;
        let b_l = vec2(-64.0, -36.0) * F;
        let l_control1 = vec2(-68.0, -20.0) * F;

        let path = vec![
            [t_l, t_control1, t_control2, t_r],
            [t_r, c_r_in_control1, c_r_in_control2, c_r],
            [c_r, c_r_out_control1, c_r_out_control2, b_r],
            [b_r, b_control1, b_control2, b_l],
            [b_l, l_control1, l_control2, t_l],
        ];
        debug_assert_eq!(path.len(), SEGMENT_TIMING.len());

        CubicBezier::new(path).to_curve()
    }
}

mod from_lvl1a_to_lvl2 {
    use lazy_static::lazy_static;

    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 2] = {
        let f = 2.0;
        [1.0 * f, 2.0 * f]
    };
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    lazy_static! {
        pub(super) static ref CURVE: CubicCurve<Vec2> = curve();
    }

    fn curve() -> CubicCurve<Vec2> {
        let from_lvl1 = *lvl1_a::TOP_LEFT;
        let mid = (*lvl1_a::TOP_LEFT + *lvl2_a::TOP_LEFT) / 2.0;
        let to_lvl2 = *lvl2_a::TOP_LEFT;

        let from_control1 = vec2(mid.x, from_lvl1.y + 4.0 * F);
        let from_control2 = vec2(to_lvl2.x, from_lvl1.y);

        let to_control1 = vec2(from_lvl1.x - 4.0 * F, to_lvl2.y);
        let to_control2 = vec2(mid.x, to_lvl2.y - 4.0 * F);

        let path = vec![
            [from_lvl1, from_control1, from_control2, mid],
            [mid, to_control1, to_control2, to_lvl2],
        ];
        debug_assert_eq!(path.len(), SEGMENT_TIMING.len());

        CubicBezier::new(path).to_curve()
    }
}

mod lvl2_a {
    use lazy_static::lazy_static;

    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 4] = {
        let f = 5.0;
        [1.0 * f, 2.0 * f, 3.0 * f, 4.0 * f]
    };
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    lazy_static! {
        pub(super) static ref TOP_LEFT: Vec2 = vec2(-48.0, 24.0) * F;
        pub(super) static ref CURVE: CubicCurve<Vec2> = curve();
    }

    fn curve() -> CubicCurve<Vec2> {
        let l_control2 = vec2(-55.0, 8.0) * F;
        let t_l = *TOP_LEFT;
        let t_control1 = vec2(-24.0, 22.0) * F;

        let t_control2 = vec2(24.0, 20.0) * F;
        let t_r = vec2(40.0, 24.0) * F;
        let r_control1 = vec2(42.0, 8.0) * F;

        let r_control2 = vec2(48.0, -8.0) * F;
        let b_r = vec2(48.0, -24.0) * F;
        let b_control1 = vec2(24.0, -34.0) * F;

        let b_control2 = vec2(-24.0, -34.0) * F;
        let b_l = vec2(-48.0, -24.0) * F;
        let l_control1 = vec2(-52.0, -8.0) * F;

        let path = vec![
            [t_l, t_control1, t_control2, t_r],
            [t_r, r_control1, r_control2, b_r],
            [b_r, b_control1, b_control2, b_l],
            [b_l, l_control1, l_control2, t_l],
        ];
        debug_assert_eq!(path.len(), SEGMENT_TIMING.len());

        CubicBezier::new(path).to_curve()
    }
}

pub(crate) fn visualize(mut gizmos: Gizmos) {
    gizmos.linestrip(
        lvl1_a::CURVE.iter_positions(100).map(|p| p.extend(0.0)),
        Color::SEA_GREEN,
    );

    gizmos.linestrip(
        lvl1_b::CURVE.iter_positions(100).map(|p| p.extend(0.0)),
        Color::LIME_GREEN,
    );

    gizmos.linestrip(
        from_lvl1a_to_lvl2::CURVE
            .iter_positions(100)
            .map(|p| p.extend(0.0)),
        Color::VIOLET,
    );

    gizmos.linestrip(
        lvl2_a::CURVE.iter_positions(100).map(|p| p.extend(0.0)),
        Color::SALMON,
    );
}
