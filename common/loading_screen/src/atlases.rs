//! Shaperka created bunch of animated loading screens we store in sprite
//! atlases.
//! This module enumerates all of them and provides a way to load them.

use bevy::{
    math::uvec2, reflect::Reflect, sprite::TextureAtlasLayout, utils::default,
};
use common_visuals::{AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer};

/// All available loading screens animations.
#[allow(missing_docs)]
#[derive(Debug, Clone, Reflect, Default, Copy)]
pub enum LoadingScreenAtlas {
    #[default]
    Bunny,
    Space,
    Hedgehog,
    LoFiGirl,

    /// Special loading screen for after sleep transition.
    WinnieInBathroom,
}

impl LoadingScreenAtlas {
    /// Picks a random loading screen.
    pub fn random() -> Self {
        let variants =
            &[Self::Hedgehog, Self::Bunny, Self::Space, Self::LoFiGirl];

        variants[rand::random::<usize>() % variants.len()]
    }

    pub(crate) fn asset_path(self) -> &'static str {
        match self {
            Self::Bunny => common_assets::misc::LOADING_SCREEN_BUNNY_ATLAS,
            Self::Space => common_assets::misc::LOADING_SCREEN_SPACE_ATLAS,
            Self::WinnieInBathroom => {
                common_assets::misc::LOADING_SCREEN_WINNIE_IN_BATHROOM_ATLAS
            }
            Self::Hedgehog => {
                common_assets::misc::LOADING_SCREEN_HEDGEHOG_ATLAS
            }
            Self::LoFiGirl => common_assets::misc::LOADING_SCREEN_LOFI_GIRL,
        }
    }

    pub(crate) fn thingies(
        self,
    ) -> (TextureAtlasLayout, AtlasAnimation, AtlasAnimationTimer) {
        let (columns, tile_size, fps, on_last_frame) = match self {
            Self::Bunny => (
                26,
                uvec2(210, 135),
                7.0,
                AtlasAnimationEnd::LoopIndefinitely,
            ),
            Self::Space => (
                12,
                uvec2(350, 190),
                7.0,
                AtlasAnimationEnd::LoopIndefinitely,
            ),
            Self::WinnieInBathroom => {
                (11, uvec2(122, 121), 3.0, AtlasAnimationEnd::RemoveTimer)
            }
            Self::Hedgehog => {
                (8, uvec2(122, 206), 2.0, AtlasAnimationEnd::LoopIndefinitely)
            }
            Self::LoFiGirl => {
                (7, uvec2(202, 116), 3.0, AtlasAnimationEnd::LoopIndefinitely)
            }
        };

        (
            TextureAtlasLayout::from_grid(tile_size, columns, 1, None, None),
            AtlasAnimation {
                last: columns as usize - 1,
                on_last_frame,
                ..default()
            },
            AtlasAnimationTimer::new_fps(fps),
        )
    }
}
