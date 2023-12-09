//! When weather is off screen we show a little arrow pointing to it on the edge
//! of the screen closest to the point where weather is.

use std::f32::consts::PI;

use crate::{
    consts::{VISIBLE_HEIGHT, VISIBLE_WIDTH},
    prelude::*,
    weather::consts::ARROW_DISTANCE_FROM_EDGE,
};

use super::{consts::MAX_ARROW_PUSH_BACK, Weather};

#[derive(Component)]
pub(crate) struct Arrow;

enum OffScreen {
    Horizontal,
    Vertical,
    Both,
}

/// Renders the arrow pointing to the weather when it's off screen.
/// Hides the arrow when the weather is on screen.
pub(crate) fn point_arrow(
    weather: Query<&Transform, (With<Weather>, Without<Arrow>)>,
    mut arrow: Query<
        (&mut Transform, &mut Visibility),
        (With<Arrow>, Without<Weather>),
    >,
) {
    let Ok(weather_transform) = weather.get_single() else {
        return;
    };

    let (mut transform, mut visibility) = arrow.single_mut();

    let weather_translation = weather_transform.translation;

    let is_off_x = weather_translation.x.abs() > VISIBLE_WIDTH / 2.0;
    let is_off_y = weather_translation.y.abs() > VISIBLE_HEIGHT / 2.0;

    *visibility = match (is_off_x, is_off_y) {
        // ~
        // if not off screen, hide the arrow
        // ~
        (false, false) => Visibility::Hidden,
        // ~
        // else show the arrow in the appropriate corner and rotation
        // ~
        (true, false) => {
            update_arrow_position_and_rotation(
                OffScreen::Horizontal,
                &mut transform,
                weather_translation,
            );

            Visibility::Visible
        }
        // off screen vertically
        (false, true) => {
            update_arrow_position_and_rotation(
                OffScreen::Vertical,
                &mut transform,
                weather_translation,
            );

            Visibility::Visible
        }
        // off screen vertically and horizontally
        (true, true) => {
            update_arrow_position_and_rotation(
                OffScreen::Both,
                &mut transform,
                weather_translation,
            );

            Visibility::Visible
        }
    }
}

const HORIZONTAL_CORNER: f32 = VISIBLE_WIDTH / 2.0 - ARROW_DISTANCE_FROM_EDGE;
const VERTICAL_CORNER: f32 = VISIBLE_HEIGHT / 2.0 - ARROW_DISTANCE_FROM_EDGE;

fn update_arrow_position_and_rotation(
    offscreen: OffScreen,
    transform: &mut Transform,
    weather: Vec3,
) {
    let x_signum = weather.x.signum();
    let y_signum = weather.y.signum();

    let push_back_x = ((weather.x.abs() - VISIBLE_WIDTH / 2.0).sqrt())
        .min(MAX_ARROW_PUSH_BACK);
    let push_back_y = ((weather.y.abs() - VISIBLE_HEIGHT / 2.0).sqrt())
        .min(MAX_ARROW_PUSH_BACK);

    match offscreen {
        OffScreen::Horizontal => {
            transform.translation = Vec3::new(
                x_signum * (HORIZONTAL_CORNER + push_back_x),
                weather.y.clamp(-VERTICAL_CORNER, VERTICAL_CORNER),
                zindex::WEATHER_ARROW,
            );
            transform.rotation = Quat::from_rotation_z(-PI / 2.0 * x_signum);
        }
        OffScreen::Vertical => {
            transform.translation = Vec3::new(
                weather.x.clamp(-HORIZONTAL_CORNER, HORIZONTAL_CORNER),
                y_signum * (VERTICAL_CORNER + push_back_y),
                zindex::WEATHER_ARROW,
            );
            transform.rotation =
                Quat::from_rotation_z(if weather.y < 0.0 { PI } else { 0.0 });
        }
        OffScreen::Both => {
            transform.translation = Vec3::new(
                x_signum * (HORIZONTAL_CORNER + push_back_x),
                y_signum * (VERTICAL_CORNER + push_back_y),
                zindex::WEATHER_ARROW,
            );

            // We want to rotate the arrow so that it points to the weather,
            // so we change basis to the arrow to be at the origin.
            // Then we measure the angle between the new origin and weather's
            // position.
            let diff = weather - transform.translation;
            let new_origin = Vec2::new(0.0, 1.0);
            let a = new_origin.angle_between(diff.truncate());
            transform.rotation = Quat::from_rotation_z(a);
        }
    }
}
