use crate::{cameras::OBJ_RENDER_LAYER, prelude::*};
use bevy::render::view::RenderLayers;
use rand::random;

/// This gets shuffled so order doesn't matter.
pub(crate) const ALL_VIDEOS: [Video; 10] = [
    Video::Alex,
    Video::Bunny,
    Video::Dance,
    Video::Fragrance,
    Video::Knight,
    Video::Mukbang,
    Video::Panda,
    Video::Puppy,
    Video::Sandwich,
    Video::Vampire,
];

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(super) enum Video {
    // Group 1 comprises verbal videos.
    Alex,
    Fragrance,

    // Group 2 comprises videos with sound/music only.
    Bunny,
    Dance,
    Knight,
    Mukbang,
    Panda,
    Puppy,
    Sandwich,
    Vampire,
}

impl Video {
    pub(super) fn is_verbal(self) -> bool {
        matches!(self, Self::Alex | Self::Fragrance)
    }

    pub(super) fn spawn(
        self,
        parent: &mut ChildBuilder,
        asset_server: &Res<AssetServer>,
    ) {
        parent.spawn((
            RenderLayers::layer(OBJ_RENDER_LAYER),
            bevy_webp_anim::WebpBundle {
                animation: asset_server.load(self.asset_path()),
                frame_rate: bevy_webp_anim::FrameRate::new(2),
                sprite: Sprite { ..default() },
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    // add some randomness to the z-index for deterministic
                    // ordering of multiple distractions
                    zindex::DISTRACTION_VIDEO + random::<f32>() * 0.1,
                )),
                ..default()
            },
        ));
    }

    fn asset_path(self) -> &'static str {
        match self {
            Self::Alex => "textures/distractions/videos/alex.webp",
            Self::Bunny => "textures/distractions/videos/bunny.webp",
            Self::Dance => "textures/distractions/videos/dance.webp",
            Self::Fragrance => "textures/distractions/videos/fragrance.webp",
            Self::Knight => "textures/distractions/videos/knight.webp",
            Self::Mukbang => "textures/distractions/videos/mukbang.webp",
            Self::Panda => "textures/distractions/videos/panda.webp",
            Self::Puppy => "textures/distractions/videos/puppy.webp",
            Self::Sandwich => "textures/distractions/videos/sandwich.webp",
            Self::Vampire => "textures/distractions/videos/vampire.webp",
        }
    }
}
