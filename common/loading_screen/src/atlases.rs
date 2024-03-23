//! Shaperka created bunch of animated loading screens we store in sprite
//! atlases.
//! This module enumerates all of them and provides a way to load them.

// TODO; old loading screen

use bevy::{
    math::vec2, reflect::Reflect, sprite::TextureAtlasLayout, utils::default,
};
use common_visuals::{AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer};

/// All available loading screens animations.
#[allow(missing_docs)]
#[derive(Debug, Clone, Reflect, Default, Copy, strum::VariantArray)]
pub enum LoadingScreenAtlas {
    #[default]
    Bunny,
    Space,
}

impl LoadingScreenAtlas {
    /// Picks a random loading screen.
    pub fn random() -> Self {
        use strum::VariantArray;

        Self::VARIANTS[rand::random::<usize>() % Self::VARIANTS.len()]
    }

    pub(crate) fn asset_path(self) -> &'static str {
        match self {
            Self::Bunny => common_assets::misc::LOADING_SCREEN_BUNNY_ATLAS,
            Self::Space => common_assets::misc::LOADING_SCREEN_SPACE_ATLAS,
        }
    }

    pub(crate) fn thingies(
        self,
    ) -> (TextureAtlasLayout, AtlasAnimation, AtlasAnimationTimer) {
        let (columns, tile_size, fps) = match self {
            Self::Bunny => (26, vec2(210.0, 135.0), 7.0),
            Self::Space => (12, vec2(350.0, 190.0), 7.0),
        };

        (
            TextureAtlasLayout::from_grid(tile_size, columns, 1, None, None),
            AtlasAnimation {
                last: columns - 1,
                on_last_frame: AtlasAnimationEnd::Loop,
                ..default()
            },
            AtlasAnimationTimer::new_fps(fps),
        )
    }
}
