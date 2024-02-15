//! Camera tracking systems for top-down games.

use std::time::Duration;

use bevy::{math::vec2, prelude::*, time::Stopwatch};
use common_action::{ActionState, GlobalAction};
use common_ext::QueryExt;
use common_visuals::{
    camera::{MainCamera, PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH},
    EASE_IN_OUT,
};
use lazy_static::lazy_static;

use crate::Player;

lazy_static! {
    /// If the player leaves this bounding box, the camera follows her.
    /// The box is centered at camera position.
    static ref BOUNDING_BOX_SIZE: Vec2 = {

        1.5 * vec2(PIXEL_VISIBLE_WIDTH, PIXEL_VISIBLE_HEIGHT)
    / //------------------------------------------------------
                            3.0

    };
}

const SYNCING_DURATION: Duration = Duration::from_millis(2000);

/// If the player leaves a bounding box defined with [`BOUNDING_BOX_SIZE`],
/// this state is inserted with the [`CameraState::SyncWithPlayer`] variant.
#[derive(Component)]
pub enum CameraState {
    /// While camera state is this, cameras are translated towards the player
    /// over the course of [`SYNCING_DURATION`].
    ///
    /// Then, the state is changed to [`CameraState::StickToPlayer`].
    SyncWithPlayer {
        /// The initial position of the camera when the state is inserted.
        initial_position: Vec2,
        /// Timer to keep track of the animation.
        animation_timer: Stopwatch,
    },
    /// While camera state is this, cameras are translated to the player's
    /// position every frame.
    /// This ends when the player stops moving.
    StickToPlayer,
}

/// If the player leaves a bounding box defined with [`BOUNDING_BOX_SIZE`],
/// [`MainCamera`] entity gets a new component [`CameraState::SyncWithPlayer`]
/// variant.
///
/// If the camera catches up with the player, the state is changed to
/// [`CameraState::StickToPlayer`].
///
/// If the player stops moving, the state is removed.
pub fn track_player_with_main_camera(
    cmd: Commands,
    controls: Res<ActionState<GlobalAction>>,
    time: Res<Time>,

    player: Query<&Transform, (Without<MainCamera>, With<Player>)>,
    camera: Query<
        (Entity, &mut Transform, Option<&mut CameraState>),
        (With<MainCamera>, Without<Player>),
    >,
) {
    track_player::<MainCamera>(cmd, controls, time, player, camera);
}

fn track_player<C: Component>(
    mut cmd: Commands,
    controls: Res<ActionState<GlobalAction>>,
    time: Res<Time>,

    player: Query<&Transform, (Without<C>, With<Player>)>,
    mut camera: Query<
        (Entity, &mut Transform, Option<&mut CameraState>),
        (With<C>, Without<Player>),
    >,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };

    let Some((camera_entity, mut camera, mut state)) =
        camera.get_single_mut_or_none()
    else {
        return;
    };

    match state.as_deref_mut() {
        // check whether the camera needs to start syncing with the player
        None => {
            let bounding_box = Rect::from_center_size(
                camera.translation.truncate(),
                *BOUNDING_BOX_SIZE,
            );

            if !bounding_box.contains(player.translation.truncate()) {
                trace!("Player left the bounding box, camera follows her");
                cmd.entity(camera_entity)
                    .insert(CameraState::SyncWithPlayer {
                        initial_position: camera.translation.truncate(),
                        animation_timer: Stopwatch::new(),
                    });
            }
        }
        Some(CameraState::SyncWithPlayer {
            initial_position,
            animation_timer,
        }) => {
            animation_timer.tick(time.delta());

            let lerp_factor = EASE_IN_OUT.ease(
                animation_timer.elapsed().as_secs_f32()
                    / SYNCING_DURATION.as_secs_f32(),
            );

            if lerp_factor >= 1.0 - f32::EPSILON {
                trace!("Camera is now synced with player");
                cmd.entity(camera_entity).insert(CameraState::StickToPlayer);
            } else {
                let new_translation = initial_position
                    .lerp(player.translation.truncate(), lerp_factor)
                    .extend(camera.translation.z);
                camera.translation = new_translation;
            }
        }
        // keep on player until they stop moving
        Some(CameraState::StickToPlayer) => {
            let any_movement = controls.get_pressed().into_iter().any(|a| {
                !matches!(a, GlobalAction::Cancel | GlobalAction::Interact)
            });

            if !any_movement {
                // TODO: a short movement in the direction of the player
                trace!("Player stopped moving, camera stops following her");
                cmd.entity(camera_entity).remove::<CameraState>();
            } else {
                let z = camera.translation.z;
                camera.translation = player.translation.truncate().extend(z);
            }
        }
    };
}
