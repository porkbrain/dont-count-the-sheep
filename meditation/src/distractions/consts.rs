use crate::prelude::*;

/// How long to wait between spawning distractions.
pub(crate) const MIN_DELAY_BETWEEN_SPAWNS: Duration = from_millis(2000);
/// How many distractions can be active at once.
pub(crate) const MAX_DISTRACTIONS: usize = 6;
/// Spawn chance based on how many distractions are already active.
pub(crate) const DISTRACTION_SPAWN_CHANCES_PER_SECOND: [f32; MAX_DISTRACTIONS] =
    [1.0, 0.9, 0.75, 0.5, 0.3, 0.25];
/// Since some videos are verbal, we don't want to play too many of them at
/// once.
/// Play more of the non-verbal ones to not overwhelm the player.
pub(crate) const MAX_VERBAL_VIDEOS_AT_ONCE: usize = 1;
/// If special is casted within this distance of the distraction, then destroy
/// it.
pub(crate) const WEATHER_SPECIAL_HITBOX_RADIUS: f32 = 35.0;
/// The actual pixel size of the image.
pub(crate) const DISTRACTION_SPRITE_SIZE: f32 = 100.0;
/// As more light is shone, more cracks appear on the distraction.
pub(crate) const MAX_CRACKS: usize = 5;
/// By default, occluder is pushed towards the climate.
pub(crate) const PUSH_BACK_FORCE_AT_REST: f32 = -20.0;
pub(crate) const PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS: f32 = 40.0;
/// At this distance, the occulder is pushed back by half of
/// [`PUSH_BACK_FORCE_WEATHER_DISTANCE`].
pub(crate) const HALF_OF_WEATHER_PUSH_BACK_FORCE_AT_DISTANCE: f32 = 150.0;
pub(crate) const PUSH_BACK_FORCE_WEATHER_DISTANCE: f32 = 80.0;
/// If light is shone on the distraction, it has a chance to crack.
pub(crate) const CRACK_CHANCE_PER_SECOND: f32 = 1.0;
pub(crate) const DISTRACTION_OCCLUDER_SIZE: f32 = 4.5;
/// Plays static as on old TVs.
pub(crate) const STATIC_ATLAS_FRAMES: usize = 5;
/// How long each frame of static is shown.
pub(crate) const STATIC_ATLAS_FRAME_TIME: Duration = from_millis(50);

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