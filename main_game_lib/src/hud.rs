//! HUD UI can some components which are always displayed, such as [`daybar`],
//! or others that pop-up when needed, such as notifications.

pub mod daybar;
pub mod notification;

use crate::prelude::*;

const MARGIN_TOP_PX: f32 = 10.0;
const MARGIN_LEFT_PX: f32 = 10.0;

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

        app.init_resource::<notification::NotificationFifo>();

        #[cfg(feature = "devtools")]
        {
            app.register_type::<daybar::DayBar>()
                .register_type::<daybar::Beats>();

            app.register_type::<notification::Notification>()
                .register_type::<notification::NotificationFifo>()
                .register_type::<notification::FifoElement>();
        }
    }
}
