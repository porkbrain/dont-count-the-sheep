#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

pub mod actor;
pub mod layout;

pub use actor::{npc, player::Player, Actor, ActorMovementEvent, ActorTarget};
use bevy::app::App;
pub use layout::{IntoMap, SquareKind, TileMap};

/// Does not add any systems, only registers generic-less types.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActorMovementEvent>()
            .register_type::<ActorMovementEvent>()
            .register_type::<Actor>()
            .register_type::<ActorTarget>();

        app.add_event::<npc::PlanPathEvent>()
            .register_type::<npc::NpcInTheMap>()
            .register_type::<npc::PlanPathEvent>()
            .register_type::<npc::BehaviorLeaf>()
            .register_type::<npc::BehaviorPaused>();
    }
}
