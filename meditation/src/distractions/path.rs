//! Contains Bezier curve points for the various paths distractions can take.
//!
//! ```text
//! t = top
//! b = bottom
//! r = right
//! l = ?
//! ```
//!
//! I made these by hand on an imaginary 128x72 grid.
//! That's because the size of the game is 640x360 (5 times larger).
//! Then I scale them up by 4 to get the final result.

use bevy::math::cubic_splines::CubicCurve;

use crate::prelude::*;

/// Scales up from an imaginary 128x72 grid to the game's 640x360 grid.
const F: f32 = 4.0;

pub(crate) struct LevelPath {
    curve: CubicCurve<Vec2>,
    kind: LevelPathKind,
}

enum LevelPathKind {
    BroadestA,
}

impl LevelPath {
    pub(crate) fn broadest_a() -> Self {
        Self {
            kind: LevelPathKind::BroadestA,
            curve: LevelPathKind::BroadestA.curve(),
        }
    }

    pub(crate) fn segments(&self) -> &[CubicSegment<Vec2>] {
        self.curve.segments()
    }

    pub(crate) fn segment(&self, time: &Time) -> (usize, f32) {
        self.kind.segment(time)
    }
}

impl LevelPathKind {
    /// Returns the current segment and how much of it has been traversed.
    fn segment(&self, time: &Time) -> (usize, f32) {
        // total path time, as path repeats once all segments have been
        // traversed
        let total_t = time.elapsed_seconds() % self.total_path_time();

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

    fn curve(&self) -> CubicCurve<Vec2> {
        match self {
            Self::BroadestA => lvl1::curve(),
        }
    }

    fn segment_timing(&self) -> &'static [f32] {
        match self {
            Self::BroadestA => &lvl1::SEGMENT_TIMING,
        }
    }

    fn total_path_time(&self) -> f32 {
        match self {
            Self::BroadestA => lvl1::TOTAL_PATH_TIME,
        }
    }
}

mod lvl1 {
    use super::*;

    pub(super) const SEGMENT_TIMING: [f32; 6] = {
        let f = 7.0;
        [1.0 * f, 1.4 * f, 1.7 * f, 2.0 * f, 3.0 * f, 3.5 * f]
    };
    pub(super) const TOTAL_PATH_TIME: f32 =
        SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

    pub(super) fn curve() -> CubicCurve<Vec2> {
        let l_control2 = vec2(-55.0, 23.0) * F;
        let t_l = vec2(-64.0, 36.0) * F;
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

pub(crate) fn visualize(mut gizmos: Gizmos) {
    gizmos.linestrip(
        lvl1::curve().iter_positions(25).map(|p| p.extend(0.0)),
        Color::WHITE,
    );
}
