pub(crate) use crate::zindex;
pub(crate) use bevy::prelude::*;
use bevy::time::Stopwatch;
pub(crate) use common_physics::{
    AngularVelocity, MotionDirection, Radians, Velocity,
};
pub(crate) use common_visuals::{
    Animation, AnimationEnd, AnimationTimer, BeginAnimationAtRandom, Flicker,
};
pub(crate) use std::time::Duration;

pub(crate) const fn from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

pub(crate) fn stopwatch_at(duration: Duration) -> Stopwatch {
    let mut s = Stopwatch::new();
    s.tick(duration);
    s
}
