pub use crate::GlobalGameState;
pub use bevy::math::vec2;
pub use bevy::prelude::*;
pub use bevy::time::Stopwatch;
pub use std::time::Duration;

pub type Pos2 = Vec2;

pub const fn from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

pub fn stopwatch_at(duration: Duration) -> Stopwatch {
    let mut s = Stopwatch::new();
    s.tick(duration);
    s
}
