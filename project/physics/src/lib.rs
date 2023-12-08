#![doc = include_str!("../README.md")]

mod systems;
mod types;

use bevy::prelude::*;

pub use types::*;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, systems::apply_velocity);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
