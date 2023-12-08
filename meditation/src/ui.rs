//! TODO:
//! - face is rendered behind menu box
//! - clean up consts and ugly code

mod menu;
mod score;

mod consts {
    use super::*;

    pub(crate) const BIG_FONT_SIZE: f32 = 45.0;
    pub(crate) const SMALL_FONT_SIZE: f32 = BIG_FONT_SIZE - 10.0;
    pub(crate) const FONT: &str = "fonts/fffforwa.ttf";
    pub(crate) const FIRST_SELECTION_FACE_OFFSET: Vec2 = Vec2::new(-80.0, 50.0);
    pub(crate) const SELECTIONS_SPACING: f32 =
        crate::weather::consts::FACE_RENDERED_SIZE + 4.0;
    pub(crate) const HIGHLIGHT_COLOR: &str = "#ffea63";
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
