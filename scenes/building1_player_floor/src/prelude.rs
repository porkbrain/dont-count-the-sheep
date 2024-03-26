pub(crate) use main_game_lib::prelude::*;

/// For a bit, show Winnie just doing the activity such as meditating or
/// sleeping before the loading screen appears and the next phase of the game
/// starts.
pub(crate) const START_LOADING_SCREEN_AFTER: Duration = from_millis(500);

/// This means that the meditation game will not start until the loading screen
/// has been shown for at least this long, plus it takes some time for the
/// fading to happen.
pub(crate) const WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST:
    Duration = from_millis(1500);

/// Walk down slowly otherwise it'll happen before the player even sees it.
pub(crate) const STEP_TIME_ONLOAD_FROM_MEDITATION: Duration = from_millis(750);
/// For the animation of stepping out of the elevator.
pub(crate) const STEP_TIME_ON_EXIT_ELEVATOR: Duration =
    STEP_TIME_ONLOAD_FROM_MEDITATION;
