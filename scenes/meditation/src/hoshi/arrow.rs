//! When Hoshi is off screen we show a little arrow pointing to it on the edge
//! of the screen closest to the point where Hoshi is.
//!
//! Only horizontal off-screen is supported, vertical off-screen cannot happen
//! as we follow Hoshi vertically with the camera.

use std::f32::consts::PI;

use common_visuals::camera::{PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH};
use main_game_lib::common_ext::QueryExt;

use super::{consts::MAX_ARROW_PUSH_BACK, Hoshi};
use crate::{hoshi::consts::ARROW_DISTANCE_FROM_EDGE, prelude::*};

#[derive(Component)]
pub(super) struct Arrow;

/// Arrow is hidden by default and shown when Hoshi is off screen.
pub(super) fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        Arrow,
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
    ));
}

pub(super) fn despawn(mut cmd: Commands, arrow: Query<Entity, With<Arrow>>) {
    if let Some(entity) = arrow.get_single_or_none() {
        cmd.entity(entity).despawn_recursive();
    }
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

    *visibility = if is_off_x {
        update_arrow_position_and_rotation(&mut transform, hoshi_translation);

        Visibility::Visible
    } else {
        // on screen, hide the arrow
        Visibility::Hidden
    };
}

const HORIZONTAL_CORNER: f32 =
    PIXEL_VISIBLE_WIDTH / 2.0 - ARROW_DISTANCE_FROM_EDGE;
const VERTICAL_CORNER: f32 =
    PIXEL_VISIBLE_HEIGHT / 2.0 - ARROW_DISTANCE_FROM_EDGE;

/// When Hoshi is horizontally off-screen (vertically the camera follows him),
/// we show the arrow on the edge of the screen closest to Hoshi.
fn update_arrow_position_and_rotation(transform: &mut Transform, hoshi: Vec3) {
    let x_signum = hoshi.x.signum();

    let push_back_x = ((hoshi.x.abs() - PIXEL_VISIBLE_WIDTH / 2.0).sqrt())
        .min(MAX_ARROW_PUSH_BACK);

    transform.translation = Vec3::new(
        x_signum * (HORIZONTAL_CORNER + push_back_x),
        hoshi.y.clamp(-VERTICAL_CORNER, VERTICAL_CORNER),
        zindex::HOSHI_ARROW,
    );
    transform.rotation = Quat::from_rotation_z(-PI / 2.0 * x_signum);
}
