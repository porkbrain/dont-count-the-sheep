//! Player and NPCs.

mod cutscenes;
mod npc;
mod player;

use bevy::ecs::event::event_update_condition;
use common_story::portrait_dialog::not_in_portrait_dialog;
use main_game_lib::{
    common_action::{interaction_pressed, move_action_pressed},
    common_top_down::{actor, npc::PlanPathEvent, ActorMovementEvent},
    cutscene::not_in_cutscene,
};

use crate::{prelude::*, Apartment};

/// Useful for despawning entities when leaving the apartment.
#[derive(Component, Reflect)]
struct CharacterEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::ApartmentLoading),
            (player::spawn, npc::spawn),
        )
        .add_systems(OnExit(GlobalGameState::ApartmentQuitting), despawn);

        app.add_systems(
            Update,
            (
                common_top_down::actor::player::move_around::<Apartment>
                    .run_if(move_action_pressed()),
                player::start_meditation_minigame_if_near_chair
                    .run_if(interaction_pressed()),
                player::enter_the_elevator.run_if(interaction_pressed()),
            )
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(not_in_portrait_dialog())
                .run_if(not_in_cutscene()),
        );

        app.add_systems(
            Update,
            player::load_zone_overlay
                .run_if(event_update_condition::<ActorMovementEvent>)
                .run_if(in_state(GlobalGameState::InApartment))
                .after(actor::emit_movement_events::<Apartment>),
        );

        app.add_systems(
            FixedUpdate,
            common_top_down::actor::animate_movement::<Apartment>
                .run_if(in_state(GlobalGameState::InApartment)),
        );

        app.add_systems(
            Update,
            (
                common_top_down::actor::npc::drive_behavior,
                common_top_down::actor::npc::plan_path::<Apartment>
                    .run_if(event_update_condition::<PlanPathEvent>),
                common_top_down::actor::npc::run_path::<Apartment>,
            )
                .chain()
                .run_if(in_state(GlobalGameState::InApartment)),
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
