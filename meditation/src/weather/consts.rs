use crate::prelude::*;

/// How many pixels per second pulls weather down
pub(crate) const GRAVITY: f32 = 512.0;
/// Caps gravity effect.
/// If weather is falling faster than this it slows down
pub(crate) const TERMINAL_VELOCITY: f32 = -216.0;
/// Pressing up does nothing if the last jump was less than this
pub(crate) const MIN_JUMP_DELAY: Duration = from_millis(750);
/// Pressing left/right does nothing if the last dash was less than this
pub(crate) const MIN_DASH_DELAY: Duration = from_millis(500);
/// Dashing in the opposite direction of the velocity should be available sooner
/// that dashing in the same direction
pub(crate) const MIN_DASH_AGAINST_VELOCITY_DELAY: Duration = from_millis(100);
/// Pressing down does nothing if the last dip was less than this
pub(crate) const MIN_DIP_DELAY: Duration = MIN_JUMP_DELAY;
/// Maximum amount of time weather can be selecting the angle of its special
/// before it fires
pub(crate) const SPECIAL_LOADING_TIME: Duration = from_millis(1500);
/// Cannot jump more times in a row than this before resetting
pub(crate) const MAX_JUMPS: u8 = 6;
/// When left/right is pressed while up/down then weather gets an extra kick
pub(crate) const HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP: f32 = 128.0;
/// When down is pressed, weather's vertical velocity is set to this value
pub(crate) const VERTICAL_VELOCITY_ON_DIP: f32 = -350.0;
/// When left/right is pressed, weather gets an extra kick
pub(crate) const DASH_VELOCITY_BOOST: f32 = 128.0;
/// The jump function uses this to calculate the jump strength
pub(crate) const BASIS_VELOCITY_ON_JUMP: f32 = 216.0;
/// When special is fired weather gets an extra kick in the chosen direction
pub(crate) const VELOCITY_BOOST_ON_SPECIAL: f32 = 350.0;

pub(crate) const BLOOM_FADE_OUT_ON_FIRED: Duration = from_millis(2000);
pub(crate) const BLOOM_FADE_OUT_ON_CANCELED: Duration = from_millis(250);
pub(crate) const INITIAL_BLOOM_INTENSITY: f32 = 0.1;
pub(crate) const INITIAL_BLOOM_LFB: f32 = 0.25;
pub(crate) const BLOOM_INTENSITY_INCREASE_PER_SECOND: f32 = 0.4;
pub(crate) const BLOOM_LFB_INCREASE_PER_SECOND: f32 = 0.5;

/// How fast does weather rotate towards its velocity vector
pub(crate) const ROTATION_SPEED: f32 = 2.0;

/// Show the falling sprite if appropriate at least after this long since the
/// last sprite change.
/// This is override if dipped.
pub(crate) const SHOW_FALLING_SPRITE_AFTER: Duration = from_millis(400);
pub(crate) const SHOW_DEFAULT_SPRITE_AFTER: Duration = from_millis(1000);

pub(crate) const BODY_ATLAS_ROWS: usize = 10;
pub(crate) const BODY_ATLAS_COLS: usize = 10;
pub(crate) const BODY_ATLAS_PADDING: Vec2 = Vec2::new(3.0, 3.0);
pub(crate) const BODY_WIDTH: f32 = 35.0;
pub(crate) const BODY_HEIGHT: f32 = 35.0;

pub(crate) const FACE_WIDTH: f32 = BODY_WIDTH;
pub(crate) const FACE_HEIGHT: f32 = BODY_HEIGHT;
pub(crate) const FACE_ATLAS_PADDING: Vec2 = BODY_ATLAS_PADDING;
pub(crate) const FACE_ATLAS_ROWS: usize = 5;
pub(crate) const FACE_ATLAS_COLS: usize = 5;
