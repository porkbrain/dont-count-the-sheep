//! Player and NPCs.

mod cutscenes;
mod player;

use bevy::ecs::event::event_update_condition;
use main_game_lib::cutscene::in_cutscene;
use top_down::actor::{emit_movement_events, movement_event_emitted};

use crate::{prelude::*, Building1PlayerFloor};

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum Building1PlayerFloorAction {
    EnterElevator,
    StartMeditation,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // the entities are spawned into the root entity
        // this means we don't need to despawn them manually as they will be
        // despawned when the scene is despawned
        // we do this to leverage ysorting

        app.add_event::<Building1PlayerFloorAction>().add_systems(
            Update,
            (
                player::start_meditation_minigame_if_near_chair,
                player::enter_the_elevator,
            )
                .run_if(event_update_condition::<Building1PlayerFloorAction>)
                .run_if(in_state(GlobalGameState::AtBuilding1PlayerFloor))
                .run_if(not(in_cutscene())),
        );

        app.add_systems(
            Update,
            player::toggle_zone_hints
                .run_if(movement_event_emitted::<Building1PlayerFloor>())
                .run_if(in_state(GlobalGameState::AtBuilding1PlayerFloor))
                .after(emit_movement_events::<Building1PlayerFloor>),
        );
    }
}
