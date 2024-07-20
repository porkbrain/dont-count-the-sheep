//! Different things that the top down map can have.

use bevy::{app::Update, ecs::schedule::IntoSystemConfigs};

use super::actor::{self, movement_event_emitted};
use crate::in_top_down_running_state;

pub mod door;

/// Adds systems related to the top down map's environmental objects.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            door::toggle
                .run_if(in_top_down_running_state())
                .run_if(movement_event_emitted())
                .after(actor::emit_movement_events),
        );
    }
}
