//! HUD UI can some components which are always displayed, such as [`daybar`],
//! or others that pop-up when needed, such as notifications.

pub mod daybar;

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // TODO: https://github.com/porkbrain/dont-count-the-sheep/issues/14
        app.insert_resource(daybar::DayBar { progress: 0.0 })
            .add_event::<daybar::IncreaseDayBarEvent>()
            .add_systems(
                First,
                daybar::increase
                    .run_if(on_event::<daybar::IncreaseDayBarEvent>()),
            );

        #[cfg(feature = "devtools")]
        {
            app.register_type::<daybar::DayBar>()
                .add_systems(Update, daybar::change_progress);
        }
    }
}
