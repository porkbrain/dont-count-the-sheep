//! Contains dialog logic and dialog texts.
//! Basically anything related to the lore is here.

#![feature(round_char_boundary)]
#![feature(trivial_bounds)]
#![deny(missing_docs)]
#![feature(let_chains)]

mod deser;
pub mod portrait_dialog;

use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};
use common_assets::store::AssetList;
use common_visuals::camera::{order, render_layer};
use serde::{Deserialize, Serialize};
use strum::{
    AsRefStr, Display, EnumCount, EnumIter, EnumString, IntoEnumIterator,
    IntoStaticStr,
};

/// Used in conjunction with [`common_assets::AssetStore`] to keep loaded assets
/// in memory. Thanks to implementation of [`AssetList`] you can now use the
/// common systems to load and unload assets.
pub struct StoryAssets;

/// List of all the NPCs and player characters.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Reflect,
    EnumIter,
    Serialize,
    Deserialize,
    Hash,
    EnumString,
    Display,
    IntoStaticStr,
    AsRefStr,
    EnumCount,
)]
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
                clear_color: ClearColorConfig::None,
                ..default()
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

impl AssetList for StoryAssets {
    fn folders() -> &'static [&'static str] {
        &[
            common_assets::character_atlases::FOLDER,
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

    /// This handle always refers to an existing texture atlas.
    #[inline]
    pub fn sprite_atlas_layout_handle(self) -> Handle<TextureAtlasLayout> {
        const ROOT: u128 = 1684183453418242348;

        Handle::weak_from_u128(ROOT + self as u128)
    }

    /// The asset to load for the character atlas.
    pub fn sprite_atlas_texture_path(self) -> &'static str {
        use common_assets::character_atlases::*;

        match self {
            Character::Winnie => WINNIE,
            Character::Marie => MARIE,
            Character::Unnamed => UNNAMED,
            _ => unimplemented!(),
        }
    }

    /// Loads all sprite atlases and stores them to the [`Assets`].
    /// Each atlas has a unique handle.
    /// Call this once when setting up the game.
    pub fn load_all_sprite_atlas_layouts(
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    ) {
        for character in Self::iter() {
            character.insert_sprite_atlas_layout(texture_atlases);
        }
    }

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

    /// Returns arguments to [`TextureAtlasLayout::from_grid`].
    fn sprite_atlas(self) -> Option<(Vec2, usize, usize, Vec2)> {
        const STANDARD_SIZE: Vec2 = Vec2::new(25.0, 46.0);

        match self {
            Character::Winnie => Some((STANDARD_SIZE, 12, 1, default())),
            Character::Marie => Some((STANDARD_SIZE, 15, 1, default())),
            Character::Unnamed => Some((STANDARD_SIZE, 15, 1, default())),
            _ => None,
        }
    }

    #[inline]
    fn insert_sprite_atlas_layout(
        self,
        texture_atlases: &mut Assets<TextureAtlasLayout>,
    ) {
        let Some((size, cols, rows, padding)) = self.sprite_atlas() else {
            return;
        };

        let atlas = TextureAtlasLayout::from_grid(
            size,
            cols,
            rows,
            Some(padding),
            None,
        );

        texture_atlases.insert(self.sprite_atlas_layout_handle(), atlas);
    }
}
