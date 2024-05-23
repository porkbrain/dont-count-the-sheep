//! Contains dialog logic and dialog texts.
//! Basically anything related to the lore is here.

#![feature(round_char_boundary)]
#![feature(trivial_bounds)]
#![deny(missing_docs)]
#![feature(let_chains)]
#![allow(clippy::too_many_arguments)]

pub mod dialog;
pub mod emoji;

use std::time::Duration;

use bevy::prelude::*;
use bevy_grid_squared::GridDirection;
use common_assets::store::AssetList;
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
    /// Sits in the mall at the Good Water stand.
    GingerCat,
    /// Walks around the mall with a pushcart.
    WhiteCat,
    /// A character.
    Emil,
    /// A character.
    Pooper,
    /// A character.
    Samizdat,
    /// A character.
    Otter,
}

/// Registers necessary assets and asset loaders.
#[derive(Default)]
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(emoji::Plugin);

        app.add_plugins(dialog::fe::portrait::Plugin)
            .init_asset_loader::<dialog::loader::Loader>()
            .init_asset::<dialog::DialogGraph>();

        app.add_systems(
            Update,
            dialog::wait_for_assets_then_spawn_dialog
                .run_if(resource_exists::<dialog::StartDialogWhenLoaded>)
                .run_if(not(resource_exists::<dialog::Dialog>)),
        );

        #[cfg(feature = "devtools")]
        {
            app.register_type::<dialog::DialogGraph>()
                .register_type::<dialog::Dialog>()
                .register_type::<Character>();
        }
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
            Character::GingerCat => "Rolo",
            Character::WhiteCat => "Fluffy",
            Character::Emil => "Emil",
            Character::Pooper => "Pooper",
            Character::Samizdat => "Samizdat",
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
            Character::Samizdat => SAMIZDAT,
            Character::WhiteCat => WHITE_CAT,
            Character::Bolt => BOLT,
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
            Character::GingerCat => GINGER_CAT,
            Character::WhiteCat => WHITE_CAT,
            Character::Emil => EMIL,
            Character::Master => MASTER,
            Character::Pooper => POOPER,
            Character::Redhead => REDHEAD,
            Character::Samizdat => SAMIZDAT,
            Character::Otter => OTTER,
        }
    }

    /// Returns arguments to [`TextureAtlasLayout::from_grid`]:
    ///
    /// * `tile_size` - Each layout grid cell size
    /// * `columns` - Grid column count
    /// * `rows` - Grid row count
    /// * `padding` - Optional padding between cells
    #[inline]
    fn sprite_atlas(self) -> Option<(Vec2, usize, usize, Vec2)> {
        const STANDARD_SIZE: Vec2 = Vec2::new(25.0, 46.0);

        match self {
            Character::Winnie => Some((STANDARD_SIZE, 12, 1, default())),
            Character::Bolt => Some((STANDARD_SIZE, 12, 1, default())),
            Character::Marie => Some((STANDARD_SIZE, 15, 1, default())),
            Character::Samizdat => Some((STANDARD_SIZE, 15, 1, default())),
            Character::WhiteCat => {
                Some((Vec2::new(48.0, 46.0), 6, 1, default()))
            }
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

    /// How long does it take to move one square.
    pub fn default_step_time(self) -> Duration {
        match self {
            Character::Winnie => Duration::from_millis(35),
            Character::WhiteCat => Duration::from_millis(120),
            _ => Duration::from_millis(50),
        }
    }

    /// How long does it take to move one square if walking slowly.
    pub fn slow_step_time(self) -> Duration {
        match self {
            Character::Winnie => Duration::from_millis(100),
            _ => Duration::from_millis(120),
        }
    }

    /// Based on the current character and direction they are facing, what's the
    /// current sprite index in the atlas if they are standing and not moving.
    pub fn standing_sprite_atlas_index(
        self,
        direction: GridDirection,
    ) -> usize {
        use GridDirection::*;

        match (self, direction) {
            (Self::WhiteCat, Bottom | Left | TopLeft | BottomLeft) => 0,
            (Self::WhiteCat, Top | Right | TopRight | BottomRight) => 3,

            (_, Bottom) => 0,
            (_, Top) => 1,
            (_, Right | TopRight | BottomRight) => 6,
            (_, Left | TopLeft | BottomLeft) => 9,
        }
    }

    /// Based on the current character, direction they are walking,
    /// how much time has elapsed since the beginning and how fast they
    /// walk, returns the index of the sprite in the atlas.
    #[inline]
    pub fn walking_sprite_atlas_index(
        self,
        direction: GridDirection,
        time: &Time,
        step_time: Duration,
    ) -> usize {
        use GridDirection::*;

        // How often we change walking frame based on how fast we're walking
        // from square to square.
        let step_secs = step_time.as_secs_f32();
        let animation_step_time = match direction {
            Top | Bottom => step_secs * 5.0,
            _ => step_secs * 3.5,
        }
        .clamp(0.2, 0.5);

        // right now, each sprite has 3 walking frames
        let extra = (time.elapsed_seconds_wrapped() / animation_step_time)
            .floor() as usize
            % 2;

        match (self, direction) {
            (Self::WhiteCat, Bottom | Left | TopLeft | BottomLeft) => 1 + extra,
            (Self::WhiteCat, Top | Right | TopRight | BottomRight) => 4 + extra,

            // defaults
            (_, Top) => 2 + extra,
            (_, Bottom) => 4 + extra,
            (_, Right | TopRight | BottomRight) => 7 + extra,
            (_, Left | TopLeft | BottomLeft) => 10 + extra,
        }
    }
}
