//! Player and NPCs.

mod cutscenes;
mod npc;
mod player;

use bevy::ecs::event::event_update_condition;
use common_top_down::actor::{self, movement_event_emitted};
use main_game_lib::cutscene::in_cutscene;

use crate::{prelude::*, Apartment};

/// Useful for despawning entities when leaving the apartment.
#[derive(Component, Reflect)]
struct CharacterEntity;

#[derive(Event, Reflect, Clone)]
pub enum ApartmentAction {
    EnterElevator,
    StartMeditation,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::ApartmentLoading),
            (player::spawn, npc::spawn),
        )
        .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn);

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

fn despawn(
    mut cmd: Commands,
    characters: Query<Entity, With<CharacterEntity>>,
) {
    debug!("Despawning character entities");

    for entity in characters.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
