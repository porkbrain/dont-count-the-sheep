use crate::prelude::*;

/// How long to wait between spawning Polpos.
pub(crate) const MIN_DELAY_BETWEEN_SPAWNS: Duration = from_millis(2000);
/// How many Polpos can be active at once.
pub(crate) const MAX_POLPOS: usize = 6;
/// Spawn chance based on how many Polpos are already active.
pub(crate) const SPAWN_CHANCES_PER_SECOND: [f32; MAX_POLPOS] =
    [1.0, 0.9, 0.75, 0.5, 0.3, 0.25];
/// Since some videos are verbal, we don't want to play too many of them at
/// once.
/// Play more of the non-verbal ones to not overwhelm the player.
pub(crate) const MAX_VERBAL_VIDEOS_AT_ONCE: usize = 1;
/// If special is casted within this distance of the Polpo, then destroy
/// it.
pub(crate) const HOSHI_SPECIAL_HITBOX_RADIUS: f32 = 35.0;
/// The actual pixel size of the image.
pub(crate) const POLPO_SPRITE_SIZE: f32 = 100.0;
/// Each video is the same square.
pub(crate) const VIDEO_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub(crate) const VIDEO_FPS: f32 = 30.0;
/// As more light is shone, more cracks appear on the Polpo.
pub(crate) const MAX_CRACKS: usize = 5;
/// Plays static as on old TVs.
pub(crate) const STATIC_ATLAS_FRAMES: usize = 5;
/// How long each frame of static is shown.
pub(crate) const STATIC_ATLAS_FRAME_TIME: Duration = from_millis(50);
pub(crate) const TENTACLE_ATLAS_COLS: usize = 6;

/// Black hole affects the poissons equation by being a source this strong.
pub(crate) const BLACK_HOLE_GRAVITY: f32 = 1.5;
pub(crate) const BLACK_HOLE_SPRITE_SIZE: f32 = 100.0;
pub(crate) const BLACK_HOLE_ATLAS_FRAMES: usize = 5;
/// Every now and then the black hole flickers with some lights
pub(crate) const BLACK_HOLE_FLICKER_CHANCE_PER_SECOND: f32 = 0.5;
pub(crate) const BLACK_HOLE_FLICKER_DURATION: Duration = from_millis(100);
pub(crate) const BLACK_HOLE_DESPAWN_CHANCE_PER_SECOND: f32 = 0.1;
/// When despawning, the black hole collapses into itself.
pub(crate) const BLACK_HOLE_FRAME_TIME: Duration = from_millis(40);
/// Otherwise it looks odd when the black hole disappears instantly.
pub(crate) const BLACK_HOLE_MIN_LIFE: Duration = from_millis(1000);

/// How many pixels the Polpo jitters when hit by the Hoshi special.
pub(crate) const JITTER_ON_HIT_INTENSITY: f32 = 4.0;
/// In 1/x seconds the jitter will be gone.
pub(crate) const JITTER_ON_HIT_TIME_PENALTY: f32 = 4.0;
