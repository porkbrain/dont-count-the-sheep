//! Ubiquitous imports for the main game library.

pub use std::time::Duration;

pub use bevy::{math::vec2, prelude::*, time::Stopwatch};
pub use common_action::{
    leafwing_input_manager::action_state::ActionState, GlobalAction,
};
pub use common_visuals::PRIMARY_COLOR;

pub use crate::{
    rscn,
    state::*,
    top_down::{self, Player, TopDownScene},
};

/// A convenience function to create a [`Duration`] from milliseconds.
pub const fn from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

/// A convenience function to create a [`Stopwatch`] from [`Duration`].
pub fn stopwatch_at(duration: Duration) -> Stopwatch {
    let mut s = Stopwatch::new();
    s.tick(duration);
    s
}
