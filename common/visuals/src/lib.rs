#![doc = include_str!("../README.md")]

mod systems;
mod types;

use bevy::app::{App, FixedUpdate, Update};
pub use types::*;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, systems::advance_animation)
            .add_systems(
                Update,
                (systems::change_frame_at_random, systems::flicker),
            );
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
