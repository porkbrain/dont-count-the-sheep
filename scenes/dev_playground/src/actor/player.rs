use bevy::render::view::RenderLayers;
use common_visuals::camera::render_layer;
use main_game_lib::common_top_down::actor::CharacterExt;

use super::CharacterEntity;
use crate::{prelude::*, DevPlayground};

pub(super) fn spawn(mut cmd: Commands) {
    cmd.spawn((
        Player,
        CharacterEntity,
        RenderLayers::layer(render_layer::OBJ),
    ))
    .insert(
        common_story::Character::Winnie
            .bundle_builder()
            .is_player(true)
            .build::<DevPlayground>(),
    );
}
