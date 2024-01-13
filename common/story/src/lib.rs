#![feature(round_char_boundary)]
#![feature(trivial_bounds)]

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*,
    render::view::RenderLayers,
};

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

#[derive(Component)]
pub struct DialogCamera;

pub fn spawn(mut cmd: Commands) {
    cmd.spawn((
        Name::from("Dialog camera"),
        DialogCamera,
        RenderLayers::layer(25), // TODO
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: 12, // TODO
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
    ));
}

pub fn despawn(mut cmd: Commands, entities: Query<Entity, With<DialogCamera>>) {
    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

impl Character {
    fn portrait_asset_path(&self) -> &'static str {
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
