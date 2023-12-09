//! Creates a field that satisfies the poisson equation.
//! The field must be at least 3x3, and the edges are set to 0.

use crate::prelude::*;
use common_physics::{GridCoords, PoissonsEquation, WorldDimensions};

use crate::consts::{STAGE_HEIGHT, STAGE_WIDTH};

pub(crate) const OPTIMAL_OVERCORRECTION_FACTOR: f32 = 1.7490273;

/// Trying to preserve the aspect ratio of the world: 630x360
pub(crate) const GRAVITY_FIELD_WIDTH: usize = 105;
pub(crate) const GRAVITY_FIELD_HEIGHT: usize = 60;

pub(crate) struct Gravity;

pub(crate) struct ChangeOfBasis(Vec2);

pub(crate) fn field() -> PoissonsEquation<Gravity> {
    field_(OPTIMAL_OVERCORRECTION_FACTOR)
}

fn field_(overcorrection_factor: f32) -> PoissonsEquation<Gravity> {
    common_physics::PoissonsEquation::new(
        GRAVITY_FIELD_WIDTH,
        GRAVITY_FIELD_HEIGHT,
    )
    .with_downward_attraction()
    .with_overcorrection_factor(overcorrection_factor)
    .with_initial_smoothing(32)
}

impl ChangeOfBasis {
    #[inline]
    pub(crate) fn new(translation: Vec2) -> Self {
        Self(translation)
    }
}

impl From<Transform> for ChangeOfBasis {
    #[inline]
    fn from(Transform { translation, .. }: Transform) -> Self {
        Self(translation.truncate())
    }
}

impl From<ChangeOfBasis> for GridCoords {
    #[inline]
    fn from(ChangeOfBasis(Vec2 { x, y }): ChangeOfBasis) -> Self {
        let field_width = GRAVITY_FIELD_WIDTH as f32;
        let field_height = GRAVITY_FIELD_HEIGHT as f32;

        GridCoords {
            // 0 is the leftmost column
            // so the more positive x the higher the column
            x: ((STAGE_WIDTH / 2.0 + x) / STAGE_WIDTH * field_width)
                .round()
                .clamp(0.0, field_width - 1.0) as usize,
            // 0 is the topmost row
            // so the more positive y the higher the row
            y: ((STAGE_HEIGHT / 2.0 - y) / STAGE_HEIGHT * field_height)
                .round()
                .clamp(0.0, field_height - 1.0) as usize,
        }
    }
}

impl WorldDimensions for ChangeOfBasis {
    #[inline]
    fn width() -> f32 {
        STAGE_WIDTH
    }

    #[inline]
    fn height() -> f32 {
        STAGE_HEIGHT
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::{STAGE_HEIGHT, STAGE_WIDTH};

    use super::*;

    #[test]
    fn it_translates_from_canvas_to_grid() {
        // top left
        assert_eq!(
            GridCoords { x: 0, y: 0 },
            ChangeOfBasis(Vec2::new(-STAGE_WIDTH / 2.0, STAGE_HEIGHT / 2.0))
                .into(),
        );
        // bottom right
        assert_eq!(
            GridCoords {
                x: GRAVITY_FIELD_WIDTH - 1,
                y: GRAVITY_FIELD_HEIGHT - 1,
            },
            ChangeOfBasis(Vec2::new(STAGE_WIDTH / 2.0, -STAGE_HEIGHT / 2.0))
                .into(),
        );
        // center left
        assert_eq!(
            GridCoords {
                x: 0,
                y: (GRAVITY_FIELD_HEIGHT as f32 / 2.0).round() as usize,
            },
            ChangeOfBasis(Vec2::new(-STAGE_WIDTH / 2.0 + 0.001, 0.0)).into(),
        );
        // top center
        assert_eq!(
            GridCoords {
                x: (GRAVITY_FIELD_WIDTH as f32 / 2.0).round() as usize,
                y: 0,
            },
            ChangeOfBasis(Vec2::new(0.0, STAGE_HEIGHT / 2.0 + 0.001)).into(),
        );
        // center
        assert_eq!(
            GridCoords {
                x: (GRAVITY_FIELD_WIDTH as f32 / 2.0).round() as usize,
                y: (GRAVITY_FIELD_HEIGHT as f32 / 2.0).round() as usize,
            },
            ChangeOfBasis(Vec2::new(0.0, 0.0)).into(),
        );
    }

    /// Binary searches optimal factor which is something between 1.5 and 2.0.
    #[test]
    fn it_finds_optimal_overcorrection_factor() {
        const MIN: f32 = 1.5;
        const MAX: f32 = 2.0;

        let calc_correction = |f| field_(f).last_smoothing_correction;

        let mut min = (MIN, calc_correction(MIN));
        let mut max = (MAX, calc_correction(MAX));
        let mut best = min;

        for _ in 0..64 {
            let mid = (min.0 + max.0) / 2.0;
            let correction = calc_correction(mid);

            if correction < best.1 {
                best = (mid, correction);
            }
            if correction == 0.0 {
                break;
            }

            if min.1 < max.1 {
                max = (mid, correction);
            } else {
                min = (mid, correction);
            }
        }

        assert_ne!(best.0, MIN, "to readjust the min bound");
        assert_ne!(best.0, MAX, "to readjust the max bound");

        assert!(
            (OPTIMAL_OVERCORRECTION_FACTOR - best.0).abs() < 0.001,
            "Replace optimal factor with {}",
            best.0
        );
    }
}
