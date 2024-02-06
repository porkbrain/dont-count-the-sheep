use bevy::render::view::RenderLayers;
use bevy_webp_anim::WebpAnimator;
use common_visuals::camera::render_layer;
use rand::random;

use super::consts::{VIDEO_FPS, VIDEO_SIZE};
use crate::prelude::*;

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
    Knight,

    // Group 2 comprises videos with sound/music only.
    Bunny,
    Dance,
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
        webp: &mut ResMut<WebpAnimator>,
        asset_server: &Res<AssetServer>,
    ) {
        parent.spawn((
            RenderLayers::layer(render_layer::OBJ),
            bevy_webp_anim::WebpBundle {
                remote_control: webp.add_and_wait_for_asset_load(
                    asset_server.load(self.asset_path()),
                    VIDEO_FPS,
                ),
                transform: {
                    let mut t = Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        // add some randomness to the z-index for deterministic
                        // ordering of multiple Polpos
                        zindex::POLPO_VIDEO + random::<f32>() * 0.1,
                    ));

                    // there's a bug where the video every so slightly goes out
                    // of the frame sometimes
                    t.scale = Vec3::splat(1.0 - 0.01);

                    t
                },
                sprite: Sprite {
                    // Because the handle is 1x1 when created and the rendering
                    // pipeline doesn't update the size when the actual video
                    // frames are being loaded into the handle, we inform the
                    // pipeline about the size.
                    // Otherwise, if the center of the video goes off screen,
                    // it won't be rendered at all.
                    custom_size: Some(VIDEO_SIZE),
                    ..default()
                },
                ..default()
            },
        ));
    }

    fn asset_path(self) -> &'static str {
        use assets::*;

        match self {
            Self::Alex => VIDEO_ALEX,
            Self::Bunny => VIDEO_BUNNY,
            Self::Dance => VIDEO_DANCE,
            Self::Fragrance => VIDEO_FRAGRANCE,
            Self::Knight => VIDEO_KNIGHT,
            Self::Mukbang => VIDEO_MUKBANG,
            Self::Panda => VIDEO_PANDA,
            Self::Puppy => VIDEO_PUPPY,
            Self::Sandwich => VIDEO_SANDWICH,
            Self::Vampire => VIDEO_VAMPIRE,
        }
    }
}
