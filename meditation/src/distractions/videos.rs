use rand::random;

use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum Video {
    Alex,
    Bunny,
    Dance,
    Fragrance,
    Knight,
    Mukbang,
    Panda,
    Puppy,
    Sandwich,
    Vampire,
}

impl Video {
    pub(super) fn spawn(
        self,
        parent: &mut ChildBuilder,
        asset_server: &Res<AssetServer>,
    ) {
        parent.spawn(bevy_webp_anim::WebpBundle {
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
        });
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
