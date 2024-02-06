//! Contains dialog logic and dialog texts.
//! Basically anything related to the lore is here.

#![feature(round_char_boundary)]
#![feature(trivial_bounds)]
#![deny(missing_docs)]

pub mod portrait_dialog;

use std::time::Duration;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*,
    render::view::RenderLayers,
};
use common_assets::store::AssetList;
use common_visuals::camera::{order, render_layer};
use strum::{EnumIter, IntoEnumIterator};

/// Used in conjunction with [`common_assets::AssetStore`] to keep loaded assets
/// in memory. Thanks to implementation of [`AssetList`] you can now use the
/// common systems to load and unload assets.
pub struct DialogAssets;

/// List of all the NPCs and player characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, EnumIter)]
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

impl AssetList for DialogAssets {
    fn folders() -> &'static [&'static str] {
        &[
            common_assets::dialog::FOLDER,
            common_assets::portraits::FOLDER,
        ]
    }
}

impl Character {
    /// How long does it take to move one square.
    pub fn default_step_time(self) -> Duration {
        match self {
            Character::Winnie => Duration::from_millis(35),
            _ => Duration::from_millis(50),
        }
    }

    /// Static str name of the character.
    pub fn name(self) -> &'static str {
        match self {
            Character::Winnie => "Winnie",
            Character::Phoebe => "Phoebe",
            Character::Marie => "Marie",
            Character::Master => "Master",
            Character::Redhead => "Redhead",
            Character::Bolt => "Bolt",
            Character::Capy => "Capy",
            Character::Cat => "Cat",
            Character::Emil => "Emil",
            Character::Pooper => "Pooper",
            Character::Unnamed => "Unnamed",
            Character::Otter => "Otter",
        }
    }
}

impl Character {
    fn portrait_asset_path(self) -> &'static str {
        use common_assets::portraits::*;

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

    /// Returns arguments to [`TextureAtlas::from_grid`].
    fn sprite_atlas(self) -> Option<(&'static str, Vec2, usize, usize, Vec2)> {
        use common_assets::character_atlases::*;

        const STANDARD_SIZE: Vec2 = Vec2::new(25.0, 46.0);

        match self {
            Character::Winnie => {
                Some((WINNIE, STANDARD_SIZE, 12, 1, default()))
            }
            Character::Marie => Some((MARIE, STANDARD_SIZE, 15, 1, default())),
            _ => None,
        }
    }

    /// This handle always refers to an existing texture atlas.
    #[inline]
    pub fn sprite_atlas_handle(self) -> Handle<TextureAtlas> {
        const ROOT: u128 = 1684183453418242348;

        Handle::weak_from_u128(ROOT + self as u128)
    }

    /// Loads all sprite atlases and stores them to the [`Assets`].
    /// Each atlas has a unique handle.
    pub fn load_all_sprite_atlases(
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) {
        for character in Self::iter() {
            character.insert_sprite_atlas(asset_server, texture_atlases);
        }
    }

    #[inline]
    fn insert_sprite_atlas(
        self,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) {
        let Some((path, size, cols, rows, padding)) = self.sprite_atlas()
        else {
            return;
        };

        let atlas = TextureAtlas::from_grid(
            asset_server.load(path),
            size,
            cols,
            rows,
            Some(padding),
            None,
        );

        texture_atlases.insert(self.sprite_atlas_handle(), atlas);
    }
}
