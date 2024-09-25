//! Ubiquitous imports for the main game library.

pub use std::time::Duration;

pub use bevy::{math::vec2, prelude::*, time::Stopwatch};
#[cfg(feature = "devtools")]
pub use bevy_inspector_egui::prelude::*;
pub use common_action::{
    leafwing_input_manager::action_state::ActionState, ActionStateExt,
    GlobalAction, MovementAction,
};
pub use common_visuals::PRIMARY_COLOR;

pub use crate::{
    bevy_rscn,
    state::*,
    top_down::{self, Player},
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

/// Similar to bevy's
/// [`bevy::ecs::schedule::common_conditions::on_event`], but useful
/// when a specific event variation is expected.
/// Used typically with enum events.
pub fn on_event_variant<T: Event + Eq + Clone>(
    variant: T,
) -> impl FnMut(EventReader<T>) -> bool + Clone {
    // The events need to be consumed, so that there are no false positives on
    // subsequent calls of the run condition. Simply checking `is_empty`
    // would not be enough. PERF: note that `count` is efficient (not
    // actually looping/iterating), due to Bevy having a specialized
    // implementation for events.
    move |mut reader: EventReader<T>| {
        reader.read().any(|event| event == &variant)
    }
}
