mod move_sprite;

use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
pub(super) use move_sprite::system as move_sprite_system;

use self::move_sprite::MoveSprite;
use crate::prelude::*;

#[derive(Resource, Reflect, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct SceneMakerToolbar {
    /// Whether the scene maker is active.
    is_active: bool,
    #[reflect(ignore)]
    selected_sprite: Option<MoveSprite>,
}

pub(super) fn spawn(mut cmd: Commands) {
    cmd.init_resource::<SceneMakerToolbar>();
}

pub(super) fn despawn(mut cmd: Commands) {
    cmd.remove_resource::<SceneMakerToolbar>();
}
