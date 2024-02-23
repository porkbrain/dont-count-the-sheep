//! Player and NPCs.

mod cutscenes;
mod npc;
mod player;

use common_action::interaction_pressed;
use common_story::portrait_dialog::not_in_portrait_dialog;
use common_top_down::actor::{self, movement_event_emitted};
use main_game_lib::cutscene::not_in_cutscene;

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
                player::start_meditation_minigame_if_near_chair,
                player::enter_the_elevator,
            )
                .run_if(interaction_pressed())
                .run_if(in_state(GlobalGameState::InApartment))
                .run_if(not_in_portrait_dialog())
                .run_if(not_in_cutscene()),
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
