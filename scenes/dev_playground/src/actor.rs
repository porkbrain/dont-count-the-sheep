//! Player and NPCs.

mod npc;
mod player;

use crate::prelude::*;

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
