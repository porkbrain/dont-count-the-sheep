pub(crate) use main_game_lib::prelude::*;

pub(crate) use crate::{Building1PlayerFloor, Building1PlayerFloorAction};

/// This means that the meditation game will not start until the loading screen
/// has been shown for at least this long, plus it takes some time for the
/// fading to happen.
pub(crate) const WHEN_ENTERING_MEDITATION_SHOW_LOADING_IMAGE_FOR_AT_LEAST:
    Duration = from_millis(1500);
/// Hard coded to make the animation play out.
pub(crate) const WINNIE_IN_BATHROOM_TRANSITION_FOR_AT_LEAST: Duration =
    from_millis(3500);

/// Walk down slowly otherwise it'll happen before the player even sees it.
pub(crate) const STEP_TIME_ONLOAD_FROM_MEDITATION: Duration = from_millis(750);
