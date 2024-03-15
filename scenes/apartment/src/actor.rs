//! Player and NPCs.

mod cutscenes;
mod npc;
mod player;

use bevy::ecs::event::event_update_condition;
use common_top_down::actor::{self, movement_event_emitted};
use main_game_lib::cutscene::in_cutscene;
pub(crate) use npc::spawn as spawn_npcs;
pub(crate) use player::spawn as spawn_player;

use crate::{prelude::*, Apartment};

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum ApartmentAction {
    EnterElevator,
    StartMeditation,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // the entities are spawned into the apartment root entity
        // this means we don't need to despawn them manually as they will be
        // despawned when the apartment is despawned
        // we do this to leverage ysorting

        app.add_event::<ApartmentAction>().add_systems(
            Update,
            (
                player::start_meditation_minigame_if_near_chair,
                player::enter_the_elevator,
            )
                .run_if(event_update_condition::<ApartmentAction>)
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(not(in_cutscene())),
        );

        app.add_systems(
            Update,
            player::load_zone_overlay
                .run_if(movement_event_emitted::<Apartment>())
                .run_if(in_state(GlobalGameState::InApartment))
                .after(actor::emit_movement_events::<Apartment>),
        );
    }
}
