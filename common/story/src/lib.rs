//! Contains dialog logic and dialog texts.
//! Basically anything related to the lore is here.

#![feature(round_char_boundary)]
#![feature(trivial_bounds)]
#![deny(missing_docs)]

use std::time::Duration;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*,
    render::view::RenderLayers,
};
use common_visuals::camera::{order, render_layer};

/// The main dialog type that should take away player control.
pub mod portrait_dialog;

/// List of all the NPCs and player characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Character {
    /// Main playable character
    Winnie,
    /// The Princess
    Phoebe,
    /// The Widow
    Marie,
    /// The cult head
    Master,

    /// A character.
    Redhead,
    /// A character.
    Bolt,
    /// A character.
    Capy,
    /// A character.
    Cat,
    /// A character.
    Emil,
    /// A character.
    Pooper,
    /// A character.
    Unnamed,
    /// A character.
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
    /// How long does it take to move one square.
    pub fn default_step_time(self) -> Duration {
        Duration::from_millis(50)
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
