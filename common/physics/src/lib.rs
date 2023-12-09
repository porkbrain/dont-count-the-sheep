#![doc = include_str!("../README.md")]

pub mod poissons_equation;
mod systems;
mod types;

use bevy::prelude::*;

pub use types::*;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<types::PoissonsEquationUpdateEvent>()
            .add_systems(FixedUpdate, systems::apply_velocity)
            .add_systems(Last, systems::update_poissons_equation);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
