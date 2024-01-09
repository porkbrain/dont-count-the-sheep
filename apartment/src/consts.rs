use crate::prelude::*;

/// For a bit, show Winnie just doing the activity such as meditating or
/// sleeping before the loading screen appears and the next phase of the game
/// starts.
pub(crate) const START_LOADING_SCREEN_AFTER: Duration = from_millis(500);

/// This means that the meditation game will not start until the loading screen
/// has been shown for at least this long, plus it takes some time for the
/// fading to happen.
pub(crate) const WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST:
    Duration = from_millis(1500);

pub(crate) const CLOUD_FRAMES: usize = 34;
pub(crate) const CLOUD_HEIGHT: f32 = 11.0;
pub(crate) const CLOUD_WIDTH: f32 = 35.0;
pub(crate) const CLOUD_PADDING: f32 = 2.0;
pub(crate) const CLOUD_ATLAS_FRAME_TIME: Duration = from_millis(500);
