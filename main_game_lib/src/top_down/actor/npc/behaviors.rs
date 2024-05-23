//! Some common behavior subtrees.

use std::time::Duration;

use bevy::time::{Timer, TimerMode};
use bevy_grid_squared::Square;

use super::{BehaviorLeaf, BehaviorNode as BN};

/// Waits for a given amount of time, does nothing meanwhile.
pub struct IdlyWaiting(pub Duration);

impl From<IdlyWaiting> for BN {
    fn from(IdlyWaiting(how_long): IdlyWaiting) -> Self {
        BN::Invert(
            BN::LeafWithTimeout(
                BehaviorLeaf::Idle,
                Timer::new(how_long, TimerMode::Once),
            )
            .into_boxed(),
        )
    }
}

/// Walks a patrol in between points in order.
#[derive(Default)]
pub struct PatrolSequence {
    /// Where to go.
    pub points: Vec<Square>,
    /// How long to wait at each point.
    pub wait_at_each: Duration,
    /// If a point is unreachable, try to go to the next one.
    /// If not provided, behavior stops if a point is unreachable.
    pub timeout_if_inaccessible: Option<Duration>,
}

impl From<PatrolSequence> for BN {
    fn from(
        PatrolSequence {
            points,
            wait_at_each,
            timeout_if_inaccessible,
        }: PatrolSequence,
    ) -> Self {
        let points = points.into_iter().map(|point| {
            let goto = BehaviorLeaf::find_path_to(point);

            BN::Sequence(vec![
                if let Some(timeout) = timeout_if_inaccessible {
                    BN::Infallible(
                        BN::LeafWithTimeout(
                            goto,
                            Timer::new(timeout, TimerMode::Repeating),
                        )
                        .into_boxed(),
                    )
                } else {
                    BN::Leaf(goto)
                },
                IdlyWaiting(wait_at_each).into(),
            ])
        });

        BN::Repeat(BN::Sequence(points.collect()).into_boxed())
    }
}
