//! The UI comprises menu and score text.
//! Open/close the menu with ESC.

mod menu;
mod score;

mod consts {
    use common_visuals::camera::PIXEL_ZOOM;

    use super::*;

    pub(crate) const BIG_FONT_SIZE: f32 = 45.0;
    pub(crate) const SMALL_FONT_SIZE: f32 = BIG_FONT_SIZE - 10.0;
    pub(crate) const FONT: &str = common_assets::fonts::PIXEL1;
    /// Used to highlight some text.
    pub(crate) const HIGHLIGHT_COLOR: &str = "#ffea63";

    pub(crate) const SCORE_EDGE_OFFSET: f32 = 25.0;

    pub(crate) const MENU_BOX_WIDTH: f32 = 215.0 * PIXEL_ZOOM as f32;
    pub(crate) const MENU_BOX_HEIGHT: f32 = 145.0 * PIXEL_ZOOM as f32;

    //
    // These spacings and sizes are arbitrary.
    // They match the size of the menu box and were hand picked.
    // #pixel_perfect
    //

    pub(crate) const SELECTIONS_LEFT_OFFSET: Val = Val::Px(128.0);
    pub(crate) const SELECTIONS_TOP_OFFSET: Val = Val::Px(54.0);
    pub(crate) const SELECTIONS_PADDING_TOP: Val = Val::Px(12.0);

    pub(crate) const SELECTION_MARKER_TOP_OFFSET_PX: f32 =
        4.5 * PIXEL_ZOOM as f32;
    pub(crate) const SELECTION_MARKER_TOP_PADDING_PX_PER_SELECTION: f32 =
        19.0 * PIXEL_ZOOM as f32;
    pub(crate) const SELECTION_MARKER_LEFT_OFFSET: Val =
        Val::Px(5.0 * PIXEL_ZOOM as f32);
}

use common_action::move_action_pressed;
use leafwing_input_manager::common_conditions::action_just_pressed;
pub(crate) use score::Score;

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::MeditationLoading),
            score::spawn,
        )
        .add_systems(OnEnter(GlobalGameState::MeditationInMenu), menu::spawn)
        .add_systems(OnExit(GlobalGameState::MeditationInMenu), menu::despawn)
        .add_systems(
            Update,
            score::update.run_if(in_state(GlobalGameState::MeditationInGame)),
        )
        .add_systems(
            Update,
            menu::open
                .run_if(in_state(GlobalGameState::MeditationInGame))
                .run_if(action_just_pressed(GlobalAction::Cancel)),
        )
        .add_systems(
            Update,
            menu::change_selection
                .run_if(in_state(GlobalGameState::MeditationInMenu))
                .run_if(move_action_pressed())
                .before(menu::select),
        )
        .add_systems(
            Update,
            // order important bcs we simulate ESC to close
            menu::select
                .run_if(in_state(GlobalGameState::MeditationInMenu))
                .run_if(action_just_pressed(GlobalAction::Interact))
                .before(menu::close),
        )
        .add_systems(
            Update,
            menu::close
                .run_if(in_state(GlobalGameState::MeditationInMenu))
                .run_if(action_just_pressed(GlobalAction::Cancel)),
        )
        .add_systems(
            OnExit(GlobalGameState::MeditationQuitting),
            score::despawn,
        );
    }
}
