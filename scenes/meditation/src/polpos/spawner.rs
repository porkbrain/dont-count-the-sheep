use bevy::{render::view::RenderLayers, utils::HashSet};
use bevy_webp_anim::WebpAnimator;
use common_visuals::camera::render_layer;
use rand::{random, seq::SliceRandom};

use super::{consts::*, videos::ALL_VIDEOS, Polpo, Video};
use crate::{polpos::PolpoEntity, prelude::*};

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
    mut cmd: Commands,
    mut spawner: ResMut<Spawner>,
    mut webp: ResMut<WebpAnimator>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
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

    cmd.spawn((polpo, PolpoEntity))
        .insert((
            RenderLayers::layer(render_layer::OBJ),
            SpriteBundle {
                texture: asset_server.load(assets::CRACK_ATLAS),
                transform: Transform::from_translation(translation),
                ..default()
            },
            TextureAtlas {
                index: 0,
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(POLPO_SPRITE_SIZE as u32),
                    MAX_CRACKS as u32,
                    1,
                    None,
                    None,
                )),
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                RenderLayers::layer(render_layer::OBJ),
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
                RenderLayers::layer(render_layer::OBJ),
                SpriteBundle {
                    texture: asset_server.load(assets::TENTACLE_ATLAS),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        zindex::POLPO_TENTACLES,
                    )),
                    ..default()
                },
                TextureAtlas {
                    index: random::<usize>() % TENTACLE_ATLAS_COLS,
                    layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                        UVec2::splat(POLPO_SPRITE_SIZE as u32),
                        TENTACLE_ATLAS_COLS as u32,
                        1,
                        None,
                        None,
                    )),
                },
            ));

            // TODO: add sound
            video.spawn(parent, &mut webp, &asset_server);
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
