pub use std::time::Duration;

pub use bevy::{math::vec2, prelude::*, time::Stopwatch};
pub use common_action::{
    leafwing_input_manager::action_state::ActionState, GlobalAction,
};

pub use crate::GlobalGameState;

pub type Pos2 = Vec2;

pub const fn from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

pub fn stopwatch_at(duration: Duration) -> Stopwatch {
    let mut s = Stopwatch::new();
    s.tick(duration);
    s
}
