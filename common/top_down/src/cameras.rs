//! Camera tracking systems for top-down games.

use std::time::Duration;

use bevy::{math::vec2, prelude::*, time::Stopwatch};
use common_ext::QueryExt;
use common_visuals::{
    camera::{
        MainCamera, PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH, PIXEL_ZOOM,
    },
    EASE_IN_OUT,
};
use lazy_static::lazy_static;

use crate::Player;

lazy_static! {
    /// If the player leaves this bounding box, the camera follows her.
    /// The box is centered at camera position.
    pub static ref BOUNDING_BOX_SIZE: Vec2 = {

        2.0 * vec2(PIXEL_VISIBLE_WIDTH, PIXEL_VISIBLE_HEIGHT)
    / //------------------------------------------------------
                            3.0

    };
}

/// How smooth is the transition of the camera from wherever it is to the
/// player's position.
pub const SYNCING_DURATION: Duration = Duration::from_millis(2000);

/// If the player leaves a bounding box defined with
/// [`static@BOUNDING_BOX_SIZE`], this component is attached.
///
/// While camera has this it is translated towards the player
/// over the course of [`SYNCING_DURATION`].
#[derive(Component)]
pub struct SyncWithPlayer {
    /// The initial position of the camera when the state is inserted.
    initial_position: Vec2,
    /// This will be set to the position of the player in the moment the
    /// syncing begins.
    /// We don't sync to player's current position because that creates
    /// hurdles such as player moving in the opposite direction, which the
    /// code would have to take into account.
    ///
    /// Instead, the make the transition pretty fast so the player has no
    /// time to move far away.
    final_position: Vec2,
    /// Timer to keep track of the animation.
    animation_timer: Stopwatch,
}

/// Recommended to run after the player's movement animation:
///
/// ```rust,ignore
/// track_player_with_main_camera.after(
///     common_top_down::actor::animate_movement::<MyScene>,
/// )
/// ```
///
/// Also don't run this in portrait dialog or cutscenes.
pub fn track_player_with_main_camera(
    cmd: Commands,
    time: Res<Time>,

    player: Query<&GlobalTransform, With<Player>>,
    camera: Query<
        (Entity, &mut Transform, Option<&mut SyncWithPlayer>),
        With<MainCamera>,
    >,
) {
    track_player::<MainCamera>(cmd, time, player, camera);
}

fn track_player<C: Component>(
    mut cmd: Commands,
    time: Res<Time>,

    player: Query<&GlobalTransform, With<Player>>,
    mut camera: Query<
        (Entity, &mut Transform, Option<&mut SyncWithPlayer>),
        With<C>,
    >,
) {
    let Some(player_pos) = player.get_single_or_none() else {
        return;
    };

    let Some((camera_entity, mut camera, mut state)) =
        camera.get_single_mut_or_none()
    else {
        return;
    };

    if let Some(SyncWithPlayer {
        initial_position,
        final_position,
        animation_timer,
    }) = state.as_deref_mut()
    {
        animation_timer.tick(time.delta());

        let lerp_factor = EASE_IN_OUT.ease(
            animation_timer.elapsed().as_secs_f32()
                / SYNCING_DURATION.as_secs_f32(),
        );

        if lerp_factor >= 1.0 - f32::EPSILON {
            trace!("Camera is now synced with player");

            cmd.entity(camera_entity).remove::<SyncWithPlayer>();
        } else {
            let precise = initial_position.lerp(*final_position, lerp_factor);
            // prevents fractions that jitter other objects
            let rounded =
                (precise * PIXEL_ZOOM as f32).round() / PIXEL_ZOOM as f32;

            let new_translation = rounded.extend(camera.translation.z);
            camera.translation = new_translation;
        }
    } else {
        // check whether the camera needs to start syncing with the player

        let bounding_box = Rect::from_center_size(
            camera.translation.truncate(),
            *BOUNDING_BOX_SIZE,
        );

        if !bounding_box.contains(player_pos.translation().truncate()) {
            trace!("Player left the bounding box, camera follows her");
            cmd.entity(camera_entity).insert(SyncWithPlayer {
                initial_position: camera.translation.truncate(),
                final_position: player_pos.translation().truncate(),
                animation_timer: Stopwatch::new(),
            });
        }
    }
}
