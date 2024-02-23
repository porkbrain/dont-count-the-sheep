//! Things that player can encounter in this scene.

use bevy::render::view::RenderLayers;
use common_store::GlobalStore;
use common_top_down::{actor::CharacterExt, ActorTarget, TopDownScene};
use common_visuals::camera::render_layer;

use crate::{prelude::*, Downtown};

/// When the downtown is loaded, the character is spawned at this square.
const DEFAULT_INITIAL_POSITION: Vec2 = vec2(-15.0, 15.0);

/// Useful for despawning entities when leaving the downtown.
#[derive(Component, Reflect)]
struct CharacterEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::DowntownLoading), spawn)
            .add_systems(OnExit(GlobalGameState::DowntownQuitting), despawn);
    }
}

fn spawn(
    mut cmd: Commands,
    store: Res<GlobalStore>,
    asset_server: Res<AssetServer>,
) {
    use common_store::DowntownStore;

    let initial_position = store
        .position_on_load()
        .get()
        .unwrap_or(DEFAULT_INITIAL_POSITION);
    store.position_on_load().remove();

    let walking_to = store
        .walk_to_onload()
        .get()
        .map(|pos| Downtown::layout().world_pos_to_square(pos))
        .map(ActorTarget::new);
    store.walk_to_onload().remove();

    let step_time = store.step_time_onload().get();
    store.step_time_onload().remove();

    cmd.spawn((
        Player,
        CharacterEntity,
        RenderLayers::layer(render_layer::OBJ),
    ))
    .insert(
        common_story::Character::Winnie
            .bundle_builder()
            .with_initial_position(initial_position)
            .with_walking_to(walking_to)
            .with_initial_step_time(step_time)
            .is_player(true)
            .build::<Downtown>(&asset_server),
    );
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
