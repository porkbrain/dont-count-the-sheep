//! Contains dialog logic and dialog texts.
//! Basically anything related to the lore is here.

#![feature(round_char_boundary)]
#![feature(trivial_bounds)]
#![deny(missing_docs)]
#![feature(let_chains)]
#![allow(clippy::too_many_arguments)]

pub mod emoji;

use std::time::Duration;

use bevy::prelude::*;
use bevy_grid_squared::GridDirection;
use common_assets::{character_atlases::WINNIE_COLS, store::AssetList};
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
    Cooper,
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

        #[cfg(feature = "devtools")]
        {
            app.register_type::<Character>();
        }
    }
}

impl AssetList for StoryAssets {
    fn folders() -> &'static [&'static str] {
        &[
            common_assets::character_atlases::FOLDER,
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
            Character::Cooper => "Cooper",
            Character::Samizdat => "Samizdat",
            Character::Otter => "Mr Otter",
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
            Character::Cooper => COOPER,
            Character::Otter => OTTER,
            Character::Phoebe => PHOEBE,
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

    /// Each character has a unique portrait asset.
    pub fn portrait_asset_path(self) -> &'static str {
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
            Character::Cooper => COOPER,
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
    fn sprite_atlas(self) -> Option<(UVec2, u32, u32, UVec2)> {
        const STANDARD_SIZE: UVec2 = UVec2::new(25, 46);

        match self {
            Character::Winnie => {
                Some((STANDARD_SIZE, WINNIE_COLS, 3, default()))
            }
            Character::Bolt => Some((STANDARD_SIZE, 12, 1, default())),
            Character::Marie => Some((STANDARD_SIZE, 15, 1, default())),
            Character::Samizdat => Some((STANDARD_SIZE, 12, 2, default())),
            Character::WhiteCat => Some((UVec2::new(48, 46), 6, 1, default())),
            Character::Cooper => Some((UVec2::new(25, 29), 2, 1, default())),
            Character::Otter => Some((UVec2::new(36, 46), 7, 1, default())),
            Character::Phoebe => Some((UVec2::new(25, 46), 12, 2, default())),
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

        texture_atlases.insert(&self.sprite_atlas_layout_handle(), atlas);
    }

    /// How long does it take to move one square.
    pub fn default_step_time(self) -> Duration {
        match self {
            Character::Winnie => Duration::from_millis(35),
            Character::WhiteCat => Duration::from_millis(120),
            Character::Cooper => Duration::from_millis(150),
            _ => Duration::from_millis(50),
        }
    }

    /// How long does it take to move one square if walking slowly.
    pub fn slow_step_time(self) -> Duration {
        match self {
            Character::Winnie => Duration::from_millis(100),
            Character::WhiteCat => Duration::from_millis(180),
            Character::Cooper => Duration::from_millis(300),
            _ => Duration::from_millis(120),
        }
    }

    /// Based on the current character and direction they are facing, what's the
    /// current sprite index in the atlas if they are standing and not moving.
    ///
    /// You can also provide how long the character has been standing still.
    pub fn standing_sprite_atlas_index(
        self,
        direction: GridDirection,
        time: &Time,
        how_long: Option<Duration>,
    ) -> usize {
        use GridDirection::*;

        let how_long = how_long.unwrap_or_default();

        match (self, direction) {
            (Self::WhiteCat, Bottom | Left | TopLeft | BottomLeft) => 0,
            (Self::WhiteCat, Top | Right | TopRight | BottomRight) => 3,

            // two standing still frames
            (Self::Cooper, _) => {
                (3 * time.elapsed_wrapped().as_secs() as usize / 4) % 2
            }

            (Self::Winnie, TopRight) => WINNIE_COLS as usize + 6,
            (Self::Winnie, TopLeft) => WINNIE_COLS as usize + 9,
            // after a few seconds, winnie puts her hands in her pockets
            (Self::Winnie, Bottom) if how_long.as_secs() > 5 => {
                WINNIE_COLS as usize
            }

            (Self::Otter, TopRight | Top | TopLeft | Left) => 2,
            (Self::Otter, BottomRight | Bottom | BottomLeft | Right) => {
                (3 * time.elapsed_wrapped().as_secs() as usize / 4) % 2
            }

            // after a few seconds, samizdat puts her hands in her pockets
            (Self::Samizdat, _) if how_long.as_secs() > 2 => 12,

            (Self::Phoebe, _) if how_long.as_secs() > 2 => 13,

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

            (Self::Cooper, _) => 0,

            (Self::Winnie, TopRight) => WINNIE_COLS as usize + 7 + extra,
            (Self::Winnie, TopLeft) => WINNIE_COLS as usize + 10 + extra,

            (Self::Otter, TopRight | Right | BottomRight) => 3 + extra,
            (Self::Otter, TopLeft | Left | BottomLeft) => 5 + extra,
            (Self::Otter, Bottom) => extra,
            (Self::Otter, Top) => 2,

            // defaults
            (_, Top) => 2 + extra,
            (_, Bottom) => 4 + extra,
            (_, Right | TopRight | BottomRight) => 7 + extra,
            (_, Left | TopLeft | BottomLeft) => 10 + extra,
        }
    }
}
