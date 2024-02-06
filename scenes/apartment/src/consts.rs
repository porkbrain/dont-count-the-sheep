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

/// When the apartment is loaded, the character is spawned at this square.
pub(crate) const DEFAULT_INITIAL_POSITION: Vec2 = vec2(-15.0, 15.0);
/// Upon going to the meditation minigame we set this value so that once the
/// game is closed, the character is spawned next to the meditation chair.
pub(crate) const POSITION_ON_LOAD_FROM_MEDITATION: Vec2 = vec2(25.0, 60.0);
/// And it does a little animation of walking down.
pub(crate) const WALK_TO_ONLOAD_FROM_MEDITATION: Vec2 = vec2(25.0, 40.0);
/// Walk down slowly otherwise it'll happen before the player even sees it.
pub(crate) const STEP_TIME_ONLOAD_FROM_MEDITATION: Duration = from_millis(750);
/// For the animation of stepping out of the elevator.
pub(crate) const STEP_TIME_ON_EXIT_ELEVATOR: Duration =
    STEP_TIME_ONLOAD_FROM_MEDITATION;
