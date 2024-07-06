//! When Hoshi is off screen we show a little arrow pointing to it on the edge
//! of the screen closest to the point where Hoshi is.

use std::f32::consts::PI;

use common_visuals::camera::{PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH};
use main_game_lib::common_ext::QueryExt;

use super::{consts::MAX_ARROW_PUSH_BACK, Hoshi};
use crate::{
    cameras::BackgroundLightScene, hoshi::consts::ARROW_DISTANCE_FROM_EDGE,
    prelude::*,
};

/// The arrow is lit by a light source.
const LIGHT_COLOR: &str = "#d9ff75";

#[derive(Component)]
pub(super) struct Arrow;

enum OffScreen {
    Horizontal,
    Vertical,
    Both,
}

/// Arrow is hidden by default and shown when Hoshi is off screen.
pub(super) fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        Arrow,
        BackgroundLightScene,
        SpriteBundle {
            texture: asset_server.load(assets::HOSHI_ARROW),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::HOSHI_ARROW,
            )),
            visibility: Visibility::Hidden,
            ..default()
        },
        OmniLightSource2D {
            intensity: 0.75,
            color: Color::hex(LIGHT_COLOR).unwrap(),
            jitter_intensity: 1.0,
            falloff: Vec3::new(3.0, 3.0, 0.05),
            ..default()
        },
    ));
}

/// Renders the arrow pointing to the Hoshi when it's off screen.
/// Hides the arrow when the Hoshi is on screen.
pub(super) fn point_arrow(
    hoshi: Query<&Transform, (With<Hoshi>, Without<Arrow>)>,
    mut arrow: Query<
        (&mut Transform, &mut Visibility),
        (With<Arrow>, Without<Hoshi>),
    >,
) {
    let Some(hoshi_transform) = hoshi.get_single_or_none() else {
        return;
    };

    let (mut transform, mut visibility) = arrow.single_mut();

    let hoshi_translation = hoshi_transform.translation;

    let is_off_x = hoshi_translation.x.abs() > PIXEL_VISIBLE_WIDTH / 2.0;
    let is_off_y = hoshi_translation.y.abs() > PIXEL_VISIBLE_HEIGHT / 2.0;

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
                hoshi_translation,
            );

            Visibility::Visible
        }
        // off screen vertically
        (false, true) => {
            update_arrow_position_and_rotation(
                OffScreen::Vertical,
                &mut transform,
                hoshi_translation,
            );

            Visibility::Visible
        }
        // off screen vertically and horizontally
        (true, true) => {
            update_arrow_position_and_rotation(
                OffScreen::Both,
                &mut transform,
                hoshi_translation,
            );

            Visibility::Visible
        }
    }
}

const HORIZONTAL_CORNER: f32 =
    PIXEL_VISIBLE_WIDTH / 2.0 - ARROW_DISTANCE_FROM_EDGE;
const VERTICAL_CORNER: f32 =
    PIXEL_VISIBLE_HEIGHT / 2.0 - ARROW_DISTANCE_FROM_EDGE;

fn update_arrow_position_and_rotation(
    offscreen: OffScreen,
    transform: &mut Transform,
    hoshi: Vec3,
) {
    let x_signum = hoshi.x.signum();
    let y_signum = hoshi.y.signum();

    let push_back_x = ((hoshi.x.abs() - PIXEL_VISIBLE_WIDTH / 2.0).sqrt())
        .min(MAX_ARROW_PUSH_BACK);
    let push_back_y = ((hoshi.y.abs() - PIXEL_VISIBLE_HEIGHT / 2.0).sqrt())
        .min(MAX_ARROW_PUSH_BACK);

    match offscreen {
        OffScreen::Horizontal => {
            transform.translation = Vec3::new(
                x_signum * (HORIZONTAL_CORNER + push_back_x),
                hoshi.y.clamp(-VERTICAL_CORNER, VERTICAL_CORNER),
                zindex::HOSHI_ARROW,
            );
            transform.rotation = Quat::from_rotation_z(-PI / 2.0 * x_signum);
        }
        OffScreen::Vertical => {
            transform.translation = Vec3::new(
                hoshi.x.clamp(-HORIZONTAL_CORNER, HORIZONTAL_CORNER),
                y_signum * (VERTICAL_CORNER + push_back_y),
                zindex::HOSHI_ARROW,
            );
            transform.rotation =
                Quat::from_rotation_z(if hoshi.y < 0.0 { PI } else { 0.0 });
        }
        OffScreen::Both => {
            transform.translation = Vec3::new(
                x_signum * (HORIZONTAL_CORNER + push_back_x),
                y_signum * (VERTICAL_CORNER + push_back_y),
                zindex::HOSHI_ARROW,
            );

            // We want to rotate the arrow so that it points to the Hoshi,
            // so we change basis to the arrow to be at the origin.
            // Then we measure the angle between the new origin and Hoshi's
            // position.
            let diff = hoshi - transform.translation;
            let new_origin = vec2(0.0, 1.0);
            let a = new_origin.angle_between(diff.truncate());
            transform.rotation = Quat::from_rotation_z(a);
        }
    }
}
