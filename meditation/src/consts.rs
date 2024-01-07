use main_game_lib::{VISIBLE_HEIGHT, VISIBLE_WIDTH};

use crate::prelude::*;

/// The stage is bigger than what's shown on screen.
pub(crate) const GRAVITY_STAGE_WIDTH: f32 = VISIBLE_WIDTH * 1.25;

/// The stage is bigger than what's shown on screen.
pub(crate) const GRAVITY_STAGE_HEIGHT: f32 = VISIBLE_HEIGHT * 1.25;

pub(crate) const WAIT_FOR_LOADING_AT_LEAST: Duration = from_millis(500);
