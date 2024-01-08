use main_game_lib::{VISIBLE_HEIGHT, VISIBLE_WIDTH};

use crate::prelude::*;

/// The stage is bigger than what's shown on screen.
pub(crate) const GRAVITY_STAGE_WIDTH: f32 = VISIBLE_WIDTH * 1.25;

/// The stage is bigger than what's shown on screen.
pub(crate) const GRAVITY_STAGE_HEIGHT: f32 = VISIBLE_HEIGHT * 1.25;

pub(crate) const ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN: Duration =
    from_millis(200);

pub(crate) const ON_RESTART_FADE_LOADING_SCREEN_OUT: Duration =
    ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN;

pub(crate) const ON_EXIT_TO_APARTMENT_FADE_LOADING_SCREEN_OUT: Duration =
    from_millis(200);
