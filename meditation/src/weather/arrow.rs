//! When weather is off screen we show a little arrow pointing to it on the edge
//! of the screen closest to the point where weather is.

use std::f32::consts::PI;

use crate::{
    consts::{HEIGHT, WIDTH},
    prelude::*,
    weather::consts::ARROW_DISTANCE_FROM_EDGE,
};

use super::Weather;

#[derive(Component)]
pub(crate) struct Arrow;

/// Renders the arrow pointing to the weather when it's off screen.
/// Hides the arrow when the weather is on screen.
pub(crate) fn point_arrow(
    weather: Query<&Transform, (With<Weather>, Without<Arrow>)>,
    mut arrow: Query<
        (&mut Transform, &mut Visibility),
        (With<Arrow>, Without<Weather>),
    >,
) {
    const HORIZONTAL_CORNER: f32 = WIDTH / 2.0 - ARROW_DISTANCE_FROM_EDGE;
    const VERTICAL_CORNER: f32 = HEIGHT / 2.0 - ARROW_DISTANCE_FROM_EDGE;

    let Ok(weather_transform) = weather.get_single() else {
        return;
    };

    let (mut transform, mut visibility) = arrow.single_mut();

    let weather_translation = weather_transform.translation;
    let is_off_x = weather_translation.x.abs() > WIDTH / 2.0;
    let is_off_y = weather_translation.y.abs() > HEIGHT / 2.0;

    let x_signum = weather_translation.x.signum();
    let y_signum = weather_translation.y.signum();

    *visibility = match (is_off_x, is_off_y) {
        // ~
        // if not off screen, hide the arrow
        // ~
        (false, false) => Visibility::Hidden,
        // ~
        // else show the arrow in the appropriate corner and rotation
        // ~
        (true, false) => {
            transform.translation = Vec3::new(
                x_signum * HORIZONTAL_CORNER,
                weather_translation
                    .y
                    .clamp(-VERTICAL_CORNER, VERTICAL_CORNER),
                zindex::WEATHER_ARROW,
            );
            transform.rotation = Quat::from_rotation_z(-PI / 2.0 * x_signum);

            Visibility::Visible
        }
        (false, true) => {
            transform.translation = Vec3::new(
                weather_translation
                    .x
                    .clamp(-HORIZONTAL_CORNER, HORIZONTAL_CORNER),
                y_signum * VERTICAL_CORNER,
                zindex::WEATHER_ARROW,
            );
            transform.rotation =
                Quat::from_rotation_z(if weather_translation.y < 0.0 {
                    PI
                } else {
                    0.0
                });

            Visibility::Visible
        }
        (true, true) => {
            transform.translation = Vec3::new(
                x_signum * HORIZONTAL_CORNER,
                y_signum * VERTICAL_CORNER,
                zindex::WEATHER_ARROW,
            );

            // We want to rotate the arrow so that it points to the weather,
            // so we change basis to the arrow to be at the origin.
            // Then we measure the angle between the new origin and weather's
            // position.
            let diff = weather_translation - transform.translation;
            let new_origin = Vec2::new(0.0, 1.0);
            let a = new_origin.angle_between(diff.truncate());
            transform.rotation = Quat::from_rotation_z(a);

            Visibility::Visible
        }
    }
}
