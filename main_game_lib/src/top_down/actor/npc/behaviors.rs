//! Some common behavior subtrees.

use std::time::Duration;

use bevy::time::{Timer, TimerMode};

use super::{BehaviorLeaf, BehaviorNode};

/// Waits for a given amount of time, does nothing meanwhile.
pub struct IdlyWaiting(pub Duration);

impl From<IdlyWaiting> for BehaviorNode {
    fn from(IdlyWaiting(how_long): IdlyWaiting) -> Self {
        BehaviorNode::Invert(
            BehaviorNode::LeafWithTimeout(
                BehaviorLeaf::Idle,
                Timer::new(how_long, TimerMode::Once),
            )
            .into_boxed(),
        )
    }
}
