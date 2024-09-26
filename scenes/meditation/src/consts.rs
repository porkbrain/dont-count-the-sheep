use crate::prelude::*;

/// Hoshi cannot go beyond this distance from the edge of the screen.
pub(crate) const HALF_LEVEL_WIDTH_PX: f32 = 400.0;

pub(crate) const ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN: Duration =
    from_millis(200);

pub(crate) const ON_RESTART_FADE_LOADING_SCREEN_OUT: Duration =
    ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN;

pub(crate) const ON_EXIT_FADE_LOADING_SCREEN_OUT: Duration = from_millis(200);
