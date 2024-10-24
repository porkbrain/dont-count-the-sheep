//! When Hoshi is off screen we show a little arrow pointing to it on the edge
//! of the screen closest to the point where Hoshi is.
//!
//! While the game is a vertical scroller, an arrow is displayed whenever Hoshi
//! goes out of player's view.

use std::f32::consts::PI;

use common_visuals::camera::{MainCamera, PIXEL_ZOOM};
use main_game_lib::{common_ext::QueryExt, vec2_ext::Vec2Ext};

use super::{consts::MAX_ARROW_PUSH_BACK, Hoshi};
use crate::{hoshi::consts::ARROW_DISTANCE_FROM_EDGE, prelude::*};

#[derive(Component)]
pub(super) struct Arrow;

/// Arrow is hidden by default and shown when Hoshi is off screen.
pub(super) fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        Arrow,
        Name::new("Hoshi Arrow"),
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

/// The arrow should be shown horizontally.
#[derive(Debug)]
enum HrOff {
    /// The arrow should be shown to the left of the screen, and the x is the
    /// leftmost pixel of the screen.
    Left(f32),
    /// Same but for the right side.
    Right(f32),
}

/// The arrow should be shown vertically.
#[derive(Debug)]
enum VrOff {
    /// The arrow should be shown to the top of the screen, and the y is the
    /// topmost pixel of the screen.
    Above(f32),
    /// Same but for the bottom side.
    Below(f32),
}

/// Renders the arrow pointing to the Hoshi when it's off screen.
/// Hides the arrow when the Hoshi is on screen.
pub(super) fn point_arrow(
    windows: Query<&Window>,
    camera: Query<&GlobalTransform, With<MainCamera>>,
    hoshi: Query<&GlobalTransform, With<Hoshi>>,
    mut arrow: Query<(&mut Transform, &mut Visibility), With<Arrow>>,
) {
    let Some(hoshi_transform) = hoshi.get_single_or_none() else {
        return;
    };
    let Some(window) = windows.get_single_or_none() else {
        return;
    };
    let Some(camera_transform) = camera.get_single_or_none() else {
        return;
    };

    // window size with respect to the current camera position
    let window_size = window.resolution.size() / PIXEL_ZOOM as f32;
    let camera_pos = camera_transform.translation().truncate();
    // Window rectangle is centered at the camera position and as big as the
    // visible window size.
    // /-------------------------\
    // |                         |
    // |                         |
    // |            C            |
    // |                         |
    // |                         |
    // \-------------------------/
    let window_rect = Rect::from_center_size(camera_pos, window_size);
    let hoshi_translation = hoshi_transform.translation().truncate();

    let to_left = window_rect.min.x > hoshi_translation.x;
    let to_right = window_rect.max.x < hoshi_translation.x;
    let above = window_rect.max.y < hoshi_translation.y;
    let below = window_rect.min.y > hoshi_translation.y;

    let (mut transform, mut visibility) = arrow.single_mut();

    let (hr, vr) = match (to_left, to_right, above, below) {
        (true, false, false, false) => {
            (Some(HrOff::Left(window_rect.min.x)), None)
        }
        (false, true, false, false) => {
            (Some(HrOff::Right(window_rect.max.x)), None)
        }
        (false, false, true, false) => {
            (None, Some(VrOff::Above(window_rect.max.y)))
        }
        (false, false, false, true) => {
            (None, Some(VrOff::Below(window_rect.min.y)))
        }
        (true, false, true, false) => (
            Some(HrOff::Left(window_rect.min.x)),
            Some(VrOff::Above(window_rect.max.y)),
        ),
        (true, false, false, true) => (
            Some(HrOff::Left(window_rect.min.x)),
            Some(VrOff::Below(window_rect.min.y)),
        ),
        (false, true, true, false) => (
            Some(HrOff::Right(window_rect.max.x)),
            Some(VrOff::Above(window_rect.max.y)),
        ),
        (false, true, false, true) => (
            Some(HrOff::Right(window_rect.max.x)),
            Some(VrOff::Below(window_rect.min.y)),
        ),
        _ => {
            // hoshi is on screen
            *visibility = Visibility::Hidden;
            return;
        }
    };

    *visibility = Visibility::Visible;
    update_arrow_position_and_rotation(
        &mut transform,
        window_rect,
        hoshi_translation,
        hr,
        vr,
    );
}

fn update_arrow_position_and_rotation(
    transform: &mut Transform,
    window_rect: Rect,
    hoshi: Vec2,
    horizontal_offscreen: Option<HrOff>,
    vertical_offscreen: Option<VrOff>,
) {
    // trace!("Hoshi is off screen, showing arrow in {offscreen:?}");
    let window_size = window_rect.size();

    let push_back = (hoshi.abs() - window_size / 2.0)
        .sqrt()
        .min(Vec2::splat(MAX_ARROW_PUSH_BACK));

    match (horizontal_offscreen, vertical_offscreen) {
        (Some(HrOff::Left(x)), None) => {
            transform.translation = Vec3::new(
                x - push_back.x,
                hoshi.y.clamp(window_rect.min.y, window_rect.max.y),
                zindex::HOSHI_ARROW,
            );
            transform.rotation = Quat::from_rotation_z(PI);
        }
    }

    // match offscreen {
    //     OffScreen::Horizontal => {
    //         transform.translation = Vec3::new(
    //             x_signum * (horizontal_corner + push_back_x),
    //             hoshi.y.clamp(-vertical_corner, vertical_corner),
    //             zindex::HOSHI_ARROW,
    //         );
    //         transform.rotation = Quat::from_rotation_z(-PI / 2.0 * x_signum);
    //     }
    //     OffScreen::Vertical => {
    //         transform.translation = Vec3::new(
    //             hoshi.x.clamp(-horizontal_corner, horizontal_corner),
    //             y_signum * (vertical_corner + push_back_y),
    //             zindex::HOSHI_ARROW,
    //         );
    //         transform.rotation =
    //             Quat::from_rotation_z(if hoshi.y < 0.0 { PI } else { 0.0 });
    //     }
    //     OffScreen::Both => {
    //         transform.translation = Vec3::new(
    //             x_signum * (horizontal_corner + push_back_x),
    //             y_signum * (vertical_corner + push_back_y),
    //             zindex::HOSHI_ARROW,
    //         );

    //         // We want to rotate the arrow so that it points to the Hoshi,
    //         // so we change basis to the arrow to be at the origin.
    //         // Then we measure the angle between the new origin and Hoshi's
    //         // position.
    //         let diff = hoshi - transform.translation.truncate();
    //         let new_origin = vec2(0.0, 1.0);
    //         let a = new_origin.angle_between(diff);
    //         transform.rotation = Quat::from_rotation_z(a);
    //     }
    // }
}
