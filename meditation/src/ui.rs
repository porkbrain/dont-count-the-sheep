//! The UI comprises menu and score text.
//! Open/close the menu with ESC.
//!
//! TODO:
//! - render face next to the menu selection
//! - proper reset of the game

mod menu;
mod score;

mod consts {
    use crate::cameras::PIXEL_ZOOM;

    use super::*;

    pub(crate) const BIG_FONT_SIZE: f32 = 45.0;
    pub(crate) const SMALL_FONT_SIZE: f32 = BIG_FONT_SIZE - 10.0;
    pub(crate) const FONT: &str = "fonts/fffforwa.ttf";
    /// Used to highlight some text.
    pub(crate) const HIGHLIGHT_COLOR: &str = "#ffea63";

    pub(crate) const SCORE_EDGE_OFFSET: f32 = 25.0;

    pub(crate) const MENU_BOX_WIDTH: f32 = 215.0 * PIXEL_ZOOM;
    pub(crate) const MENU_BOX_HEIGHT: f32 = 145.0 * PIXEL_ZOOM;

    //
    // These spacings and sizes are arbitrary.
    // They match the size of the menu box and were hand picked.
    //

    pub(crate) const SELECTIONS_LEFT_OFFSET: Val = Val::Px(128.0);
    pub(crate) const SELECTIONS_TOP_OFFSET: Val = Val::Px(55.0);
    pub(crate) const SELECTIONS_PADDING_TOP: Val = Val::Px(12.0);
}

pub(crate) use score::Score;

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, score::spawn)
            .add_systems(
                OnEnter(GlobalGameState::MeditationInMenu),
                menu::spawn,
            )
            .add_systems(
                OnExit(GlobalGameState::MeditationInMenu),
                menu::despawn,
            )
            .add_systems(
                Update,
                score::update
                    .run_if(in_state(GlobalGameState::MeditationInGame)),
            )
            .add_systems(
                Update,
                menu::open.run_if(in_state(GlobalGameState::MeditationInGame)),
            )
            .add_systems(
                Update,
                // order important bcs we simulate ESC to close
                menu::select
                    .run_if(in_state(GlobalGameState::MeditationInMenu))
                    .before(menu::close),
            )
            .add_systems(
                Update,
                menu::close.run_if(in_state(GlobalGameState::MeditationInMenu)),
            )
            .add_systems(
                OnEnter(GlobalGameState::MeditationQuitting),
                score::despawn,
            );
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
