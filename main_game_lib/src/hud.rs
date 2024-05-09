//! HUD UI can some components which are always displayed, such as [`daybar`],
//! or others that pop-up when needed, such as notifications.

pub mod daybar;

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // TODO: https://github.com/porkbrain/dont-count-the-sheep/issues/14
        app.insert_resource(daybar::DayBar::default())
            .add_event::<daybar::UpdateDayBarEvent>()
            .add_systems(
                First,
                daybar::update.run_if(on_event::<daybar::UpdateDayBarEvent>()),
            )
            .add_systems(Update, daybar::interact);
    }
}
