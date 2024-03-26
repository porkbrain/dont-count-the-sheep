pub(crate) use main_game_lib::prelude::*;

pub(crate) use crate::{
    Building1Basement1, Building1Basement1Action, Building1Basement1TileKind,
};

/// For a bit, show Winnie just doing the activity such as meditating or
/// sleeping before the loading screen appears and the next phase of the game
/// starts.
pub(crate) const START_LOADING_SCREEN_AFTER: Duration = from_millis(500);
