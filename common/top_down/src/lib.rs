#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod actor;
pub mod layout;

pub use actor::{player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::app::App;
pub use layout::{IntoMap, Map, SquareKind};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActorMovementEvent>()
            .register_type::<ActorMovementEvent>();

        app.register_type::<Actor>().register_type::<ActorTarget>();
    }
}
