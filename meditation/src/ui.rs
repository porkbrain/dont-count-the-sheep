//! The UI comprises menu and score text.
//!
//! TODO:
//! - face is rendered behind menu box
//! - proper reset of the game

mod menu;
mod score;

mod consts {
    use super::*;

    pub(crate) const BIG_FONT_SIZE: f32 = 45.0;
    pub(crate) const SMALL_FONT_SIZE: f32 = BIG_FONT_SIZE - 10.0;
    pub(crate) const FONT: &str = "fonts/fffforwa.ttf";
    /// Used to highlight some text.
    pub(crate) const HIGHLIGHT_COLOR: &str = "#ffea63";

    pub(crate) const SCORE_EDGE_OFFSET: f32 = 25.0;

    pub(crate) const MENU_BOX_WIDTH: f32 = 215.0 * crate::consts::PIXEL_ZOOM;
    pub(crate) const MENU_BOX_HEIGHT: f32 = 145.0 * crate::consts::PIXEL_ZOOM;

    //
    // These spacings and sizes are arbitrary.
    // They match the size of the menu box and were hand picked.
    //

    pub(crate) const FIRST_SELECTION_FACE_OFFSET: Vec2 = vec2(-80.0, 50.0);
    pub(crate) const SELECTIONS_SPACING: f32 =
        crate::weather::consts::FACE_RENDERED_SIZE + 4.0;

    pub(crate) const SELECTIONS_LEFT_OFFSET: Val = Val::Px(128.0);
    pub(crate) const SELECTIONS_TOP_OFFSET: Val = Val::Px(55.0);
    pub(crate) const SELECTIONS_PADDING_TOP: Val = Val::Px(12.0);
}

pub(crate) use menu::Selection;
pub(crate) use score::Score;

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (score::spawn, menu::spawn))
            .add_systems(Update, score::update)
            // order important bcs we simulate ESC to close
            .add_systems(
                Update,
                (menu::open, menu::select, menu::close).chain(),
            );
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
