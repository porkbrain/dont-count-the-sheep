use bevy::render::view::RenderLayers;
use main_game_lib::{
    common_top_down::actor::CharacterExt, common_visuals::camera::render_layer,
};

use super::CharacterEntity;
use crate::{layout::HallwayEntity, prelude::*, Apartment};

pub(super) fn spawn(mut cmd: Commands) {
    cmd.spawn((
        CharacterEntity,
        HallwayEntity,
        RenderLayers::layer(render_layer::OBJ),
    ))
    .insert(
        common_story::Character::Marie
            .bundle_builder()
            .with_initial_position(vec2(-80.0, -95.0))
            .build::<Apartment>(),
    );
}
