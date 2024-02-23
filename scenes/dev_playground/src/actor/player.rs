use bevy::{asset, render::view::RenderLayers};
use common_top_down::actor::CharacterExt;
use common_visuals::camera::render_layer;

use super::CharacterEntity;
use crate::{prelude::*, DevPlayground};

pub(super) fn spawn(mut cmd: Commands, asset_server: Res<asset::AssetServer>) {
    cmd.spawn((
        Player,
        CharacterEntity,
        RenderLayers::layer(render_layer::OBJ),
    ))
    .insert(
        common_story::Character::Winnie
            .bundle_builder()
            .is_player(true)
            .build::<DevPlayground>(&asset_server),
    );
}
