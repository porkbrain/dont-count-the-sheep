use crate::prelude::*;

/// `#0a0d0f`
///
/// We use this color to match the background color of the rooms as they
/// gradient out to this black.
pub(crate) const CLEAR_COLOR_BG: Color = Color::srgb(0.039, 0.051, 0.059);

/// The path to the meditation entry scene.
/// This scene is always loaded first when meditation is started.
pub(crate) const ENTRY_ROOM_ASSET_PATH: &str =
    "scenes/meditation_room_entry.tscn";

pub(crate) const DEFAULT_ROOM_HEIGHT_PX: f32 = 1209.0;

/// Hoshi cannot go beyond this distance from the edge of the screen.
pub(crate) const HALF_LEVEL_WIDTH_PX: f32 = 400.0;

pub(crate) const ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN: Duration =
    from_millis(200);

pub(crate) const ON_RESTART_FADE_LOADING_SCREEN_OUT: Duration =
    ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN;

pub(crate) const ON_EXIT_FADE_LOADING_SCREEN_OUT: Duration = from_millis(200);
