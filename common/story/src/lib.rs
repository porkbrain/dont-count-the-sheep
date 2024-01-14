//! Contains dialog logic and dialog texts.
//! Basically anything related to the lore is here.

#![feature(round_char_boundary)]
#![feature(trivial_bounds)]
#![deny(missing_docs)]

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*,
    render::view::RenderLayers,
};
use common_visuals::camera::{order, render_layer};

/// The main dialog type that should take away player control.
pub mod portrait_dialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
enum Character {
    /// Main playable character
    Winnie,
    /// The Princess
    Phoebe,
    /// The Widow
    Marie,
    /// The cult head
    Master,

    Redhead,
    Bolt,
    Capy,
    Cat,
    Emil,
    Pooper,
    Unnamed,
    Otter,
}

/// Marks the dialog camera.
#[derive(Component)]
pub struct DialogCamera;

/// Spawns the dialog camera which has a high order and only renders the dialog
/// entities.
pub fn spawn_camera(mut cmd: Commands) {
    cmd.spawn((
        Name::from("Dialog camera"),
        DialogCamera,
        RenderLayers::layer(render_layer::DIALOG),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::DIALOG,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
    ));
}

/// Despawns the dialog camera.
/// By despawning the camera you clean up the existing entities.
/// Some scenes don't have the dialog.
pub fn despawn_camera(
    mut cmd: Commands,
    entities: Query<Entity, With<DialogCamera>>,
) {
    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

impl Character {
    fn portrait_asset_path(self) -> &'static str {
        use common_assets::paths::portraits::*;

        match self {
            Character::Winnie => WINNIE,
            Character::Phoebe => PHOEBE,
            Character::Marie => MARIE,
            Character::Bolt => BOLT,
            Character::Capy => CAPY,
            Character::Cat => CAT,
            Character::Emil => EMIL,
            Character::Master => MASTER,
            Character::Pooper => POOPER,
            Character::Redhead => REDHEAD,
            Character::Unnamed => UNNAMED,
            Character::Otter => OTTER,
        }
    }
}
