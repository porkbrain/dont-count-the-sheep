use bevy::{render::view::RenderLayers, time::Stopwatch, utils::HashSet};
use bevy_magic_light_2d::gi::types::LightOccluder2D;
use bevy_webp_anim::WebpAnimator;
use rand::{random, seq::SliceRandom};

use super::{consts::*, videos::ALL_VIDEOS, Polpo, PolpoOccluder, Video};
use crate::{
    cameras::{BackgroundLightScene, OBJ_RENDER_LAYER},
    polpos::PolpoEntity,
    prelude::*,
};

/// Manages spawning of Polpos.
#[derive(Resource, Debug)]
pub(super) struct Spawner {
    /// What videos are currently active.
    /// We never spawn the same video twice.
    active: HashSet<Video>,
    /// How many videos out of active are verbal.
    /// We don't want to play too many of those at once.
    out_of_those_are_verbal: usize,
    /// When choosing the next video, we iterate over this array based on the
    /// last index (the first tuple member.)
    sampler: (usize, [Video; 10]),
    /// When was the last Polpo spawned.
    last_spawned_at: Stopwatch,
}

/// If we don't have another video to spawn of if too crowded, then we do
/// nothing.
pub(super) fn try_spawn_next(
    mut spawner: ResMut<Spawner>,
    mut webp: ResMut<WebpAnimator>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    spawner.last_spawned_at.tick(time.delta());

    if spawner.active.len() >= MAX_POLPOS {
        return;
    }
    if spawner.last_spawned_at.elapsed() < MIN_DELAY_BETWEEN_SPAWNS {
        return;
    }

    let spawn_chance_per_second =
        SPAWN_CHANCES_PER_SECOND[spawner.active.len()];
    if random::<f32>() > spawn_chance_per_second * time.delta_seconds() {
        return;
    }
    let Some(video) = spawner.sample_next() else {
        trace!("No video to spawn");
        return;
    };

    trace!("Spawning video {video:?}");

    spawner.last_spawned_at.reset();

    let polpo = Polpo::new(video);

    let translation = {
        let (seg_index, seg_t) = polpo.path_segment();
        let seg = &polpo.path.segments()[seg_index];
        seg.position(seg_t).extend(zindex::POLPO_CRACK)
    };

    commands
        .spawn((polpo, PolpoEntity))
        .insert((
            RenderLayers::layer(OBJ_RENDER_LAYER),
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load(assets::CRACK_ATLAS),
                    vec2(POLPO_SPRITE_SIZE, POLPO_SPRITE_SIZE),
                    MAX_CRACKS,
                    1,
                    None,
                    None,
                )),
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_translation(translation),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                RenderLayers::layer(OBJ_RENDER_LAYER),
                SpriteBundle {
                    texture: asset_server.load(assets::POLPO_FRAME),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        zindex::POLPO_FRAME,
                    )),
                    ..default()
                },
            ));

            parent.spawn((
                RenderLayers::layer(OBJ_RENDER_LAYER),
                SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(
                        TextureAtlas::from_grid(
                            asset_server.load(assets::TENTACLE_ATLAS),
                            vec2(POLPO_SPRITE_SIZE, POLPO_SPRITE_SIZE),
                            TENTACLE_ATLAS_COLS,
                            1,
                            None,
                            None,
                        ),
                    ),
                    sprite: TextureAtlasSprite::new(
                        random::<usize>() % TENTACLE_ATLAS_COLS,
                    ),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        zindex::POLPO_TENTACLES,
                    )),
                    ..default()
                },
            ));

            // TODO: add sound
            video.spawn(parent, &mut webp, &asset_server);

            parent.spawn((
                PolpoOccluder,
                BackgroundLightScene,
                SpatialBundle { ..default() },
                LightOccluder2D {
                    h_size: Vec2::new(OCCLUDER_SIZE, OCCLUDER_SIZE),
                },
            ));
        });
}

impl Spawner {
    pub(super) fn new() -> Self {
        Self {
            active: HashSet::default(),
            out_of_those_are_verbal: 0,
            sampler: (0, {
                let mut all_videos = ALL_VIDEOS;
                all_videos.shuffle(&mut rand::thread_rng());
                all_videos
            }),
            last_spawned_at: stopwatch_at(MIN_DELAY_BETWEEN_SPAWNS),
        }
    }

    pub(super) fn despawn(&mut self, video: Video) {
        let did_remove = self.active.remove(&video);
        debug_assert!(did_remove);

        if video.is_verbal() {
            self.out_of_those_are_verbal -= 1;
        }
    }

    /// Picks next suitable video to spawn.
    /// Mutates self in the process.
    fn sample_next(&mut self) -> Option<Video> {
        let too_many_verbals =
            self.out_of_those_are_verbal >= MAX_VERBAL_VIDEOS_AT_ONCE;

        let started_at_index = self.sampler.0;

        let (last_index, videos) = &mut self.sampler;
        loop {
            let next_index = (*last_index + 1) % videos.len();
            if started_at_index == next_index {
                // We've iterated over all videos and nothing was suitable.
                // Already too many polpos spawned.
                return None;
            }

            let next_video = videos[next_index];
            *last_index = next_index;

            if too_many_verbals && next_video.is_verbal() {
                continue;
            }

            let did_unique_insert = self.active.insert(next_video);

            if !did_unique_insert {
                continue;
            }

            if next_video.is_verbal() {
                self.out_of_those_are_verbal += 1;
            }

            break Some(next_video);
        }
    }
}
