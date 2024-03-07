mod export;
mod move_sprite;

use bevy_egui::EguiContexts;
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

// 1. Export scene
pub(crate) fn update_ui<T: TopDownScene>(
    mut contexts: EguiContexts,
    mut toolbar: ResMut<SceneMakerToolbar>,

    // for store procedure
    sprites: Query<
        (&SceneSpriteConfig, &Name, &Transform, &Sprite),
        With<LoadedFromSceneFile>,
    >,
) where
    T::LocalTileKind: Ord,
{
    let ctx = contexts.ctx_mut();
    bevy_egui::egui::Window::new("Scene maker")
        .vscroll(true)
        .show(ctx, |ui| {
            //
            // 1.
            //
            if ui.button("Store map").clicked() {
                export_map::<T>(&mut toolbar);
            }
        });
}
