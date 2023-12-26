pub(crate) use crate::{assets, zindex};
pub(crate) use bevy::math::vec2;
pub(crate) use bevy::prelude::*;
use bevy::time::Stopwatch;
pub(crate) use common_physics::{
    AngularVelocity, MotionDirection, Radians, Velocity,
};
pub(crate) use common_visuals::{
    Animation, AnimationEnd, AnimationTimer, BeginAnimationAtRandom, Flicker,
};
pub(crate) use main_game_lib::GlobalGameState;
pub(crate) use std::time::Duration;

pub(crate) const fn from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

pub(crate) fn stopwatch_at(duration: Duration) -> Stopwatch {
    let mut s = Stopwatch::new();
    s.tick(duration);
    s
}

pub(crate) type Pos2 = Vec2;
