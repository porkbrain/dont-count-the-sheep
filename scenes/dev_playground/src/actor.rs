//! Player and NPCs.

mod npc;
mod player;

use bevy::ecs::event::event_update_condition;
use common_story::portrait_dialog::not_in_portrait_dialog;
use main_game_lib::{
    common_action::move_action_pressed, common_top_down::npc::PlanPathEvent,
    cutscene::not_in_cutscene,
};

use crate::{prelude::*, Test};

#[derive(Component, Reflect)]
struct CharacterEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::Blank),
            (player::spawn, npc::spawn),
        )
        .add_systems(OnExit(GlobalGameState::InDevPlayground), despawn);

        app.add_systems(
            Update,
            (common_top_down::actor::player::move_around::<Test>
                .run_if(move_action_pressed()),)
                .run_if(in_state(GlobalGameState::InDevPlayground))
                .run_if(not_in_portrait_dialog())
                .run_if(not_in_cutscene()),
        );

        app.add_systems(
            FixedUpdate,
            common_top_down::actor::animate_movement::<Test>
                .run_if(in_state(GlobalGameState::InDevPlayground)),
        );

        app.add_systems(
            Update,
            (
                common_top_down::actor::npc::drive_behavior,
                common_top_down::actor::npc::plan_path::<Test>
                    .run_if(event_update_condition::<PlanPathEvent>),
                common_top_down::actor::npc::run_path::<Test>,
            )
                .chain()
                .run_if(in_state(GlobalGameState::InDevPlayground)),
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
