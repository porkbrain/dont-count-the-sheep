//! Things that player can encounter in this scene.

use bevy::render::view::RenderLayers;
use common_store::GlobalStore;
use common_top_down::{actor::CharacterExt, layout::LAYOUT, ActorTarget};
use common_visuals::camera::render_layer;

use crate::prelude::*;

/// When the downtown is loaded, the character is spawned at this square.
const DEFAULT_INITIAL_POSITION: Vec2 = vec2(-15.0, 15.0);

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum DowntownAction {}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _: &mut App) {}
}

pub(crate) fn spawn_player(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    store: &GlobalStore,
) -> Vec<Entity> {
    use common_store::DowntownStore;

    let initial_position = store
        .position_on_load()
        .get()
        .unwrap_or(DEFAULT_INITIAL_POSITION);
    store.position_on_load().remove();

    let walking_to = store
        .walk_to_onload()
        .get()
        .map(|pos| LAYOUT.world_pos_to_square(pos))
        .map(ActorTarget::new);
    store.walk_to_onload().remove();

    let step_time = store.step_time_onload().get();
    store.step_time_onload().remove();

    let mut player =
        cmd.spawn((Player, RenderLayers::layer(render_layer::OBJ)));
    common_story::Character::Winnie
        .bundle_builder()
        .with_initial_position(initial_position)
        .with_walking_to(walking_to)
        .with_initial_step_time(step_time)
        .is_player(true)
        .insert(asset_server, &mut player);
    let player = player.id();

    vec![player]
}
