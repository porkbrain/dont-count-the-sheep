use crate::{
    cameras::{BackgroundLightScene, OBJ_RENDER_LAYER},
    prelude::*,
};
use bevy::{render::view::RenderLayers, time::Stopwatch, utils::HashSet};
use bevy_magic_light_2d::gi::types::LightOccluder2D;
use rand::{random, seq::SliceRandom};

use super::{
    consts::*, videos::ALL_VIDEOS, Distraction, DistractionOccluder, Video,
};

/// Manages spawning of distractions.
#[derive(Resource, Debug)]
pub(super) struct Spawner {
    /// What videos are currently active.
    /// We never spawn the same video twice.
    active: HashSet<Video>,
    /// How many videos out of active are verbal.
    out_of_those_are_verbal: usize,
    /// When choosing the next video, we iterate over this array based on the
    /// last index (the first tuple member.)
    sampler: (usize, [Video; 10]),
    /// When was the last distraction spawned.
    /// TODO: pausable
    last_spawned_at: Stopwatch,
}

/// If we don't have another video to spawn of if too crowded, then we do
/// nothing.
pub(super) fn try_spawn_next(
    mut spawner: ResMut<Spawner>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    spawner.last_spawned_at.tick(time.delta());

    if spawner.active.len() >= MAX_DISTRACTIONS {
        return;
    }
    if spawner.last_spawned_at.elapsed() < MIN_DELAY_BETWEEN_SPAWNS {
        return;
    }

    let spawn_chance_per_second =
        DISTRACTION_SPAWN_CHANCES_PER_SECOND[spawner.active.len()];
    if random::<f32>() > spawn_chance_per_second * time.delta_seconds() {
        return;
    }
    let Some(video) = spawner.sample_next() else {
        trace!("No video to spawn");
        return;
    };

    trace!("Spawning video {video:?}");

    spawner.last_spawned_at.reset();

    commands
        .spawn((Distraction::new(video), AngularVelocity::default()))
        .insert((
            RenderLayers::layer(OBJ_RENDER_LAYER),
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("textures/distractions/crack_atlas.png"),
                    vec2(DISTRACTION_SPRITE_SIZE, DISTRACTION_SPRITE_SIZE),
                    MAX_CRACKS,
                    1,
                    None,
                    None,
                )),
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    zindex::DISTRACTION_CRACK,
                )),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                RenderLayers::layer(OBJ_RENDER_LAYER),
                SpriteBundle {
                    texture: asset_server
                        .load("textures/distractions/frame.png"),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        zindex::DISTRACTION_FRAME,
                    )),
                    ..default()
                },
            ));

            // TODO: vary videos
            // TODO: sound
            video.spawn(parent, &asset_server);

            parent.spawn((
                DistractionOccluder,
                SpatialBundle { ..default() },
                BackgroundLightScene,
                LightOccluder2D {
                    h_size: Vec2::new(
                        DISTRACTION_OCCLUDER_SIZE,
                        DISTRACTION_OCCLUDER_SIZE,
                    ),
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
                let mut all_videos = ALL_VIDEOS.clone();
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
                // Already too many distractions spawned.
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
