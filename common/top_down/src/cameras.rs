//! Camera tracking systems for top-down games.

use std::time::Duration;

use bevy::{math::vec2, prelude::*, time::Stopwatch};
use common_action::{ActionState, GlobalAction};
use common_ext::QueryExt;
use common_visuals::{
    camera::{MainCamera, PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH},
    BeginInterpolationEvent, EASE_IN_OUT,
};
use lazy_static::lazy_static;

use crate::{Actor, Player};

lazy_static! {
    /// If the player leaves this bounding box, the camera follows her.
    /// The box is centered at camera position.
    pub static ref BOUNDING_BOX_SIZE: Vec2 = {

        1.5 * vec2(PIXEL_VISIBLE_WIDTH, PIXEL_VISIBLE_HEIGHT)
    / //------------------------------------------------------
                            3.0

    };
}

/// When the player stops moving, nudge the camera towards the
/// player's direction by a few pixels.
const NUDGE_CAMERA_TOWARDS_PLAYER_DIRECTION_BY_PXS: f32 = 20.0;

/// How smooth is the transition of the camera from wherever it is to the
/// player's position.
pub const SYNCING_DURATION: Duration = Duration::from_millis(2000);

/// If the player leaves a bounding box defined with
/// [`static@BOUNDING_BOX_SIZE`], this state is inserted with the
/// [`CameraState::SyncWithPlayer`] variant.
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

/// If the player leaves a bounding box defined with
/// [`static@BOUNDING_BOX_SIZE`], [`MainCamera`] entity gets a new component
/// [`CameraState::SyncWithPlayer`] variant.
///
/// If the camera catches up with the player, the state is changed to
/// [`CameraState::StickToPlayer`].
///
/// If the player stops moving, the state is removed.
///
/// Recommended to run after the player's movement animation:
///
/// ```rust,ignore
/// track_player_with_main_camera.after(
///     common_top_down::actor::animate_movement::<MyScene>,
/// )
/// ```
pub fn track_player_with_main_camera(
    cmd: Commands,
    controls: Res<ActionState<GlobalAction>>,
    time: Res<Time>,
    events: EventWriter<BeginInterpolationEvent>,

    player: Query<
        (&Actor, &GlobalTransform),
        (Without<MainCamera>, With<Player>),
    >,
    camera: Query<
        (Entity, &mut Transform, Option<&mut CameraState>),
        (With<MainCamera>, Without<Player>),
    >,
) {
    track_player::<MainCamera>(cmd, controls, time, events, player, camera);
}

fn track_player<C: Component>(
    mut cmd: Commands,
    controls: Res<ActionState<GlobalAction>>,
    time: Res<Time>,
    mut events: EventWriter<BeginInterpolationEvent>,

    player: Query<(&Actor, &GlobalTransform), (Without<C>, With<Player>)>,
    mut camera: Query<
        (Entity, &mut Transform, Option<&mut CameraState>),
        (With<C>, Without<Player>),
    >,
) {
    let Some((actor, player_pos)) = player.get_single_or_none() else {
        return;
    };

    let Some((camera_entity, mut camera, mut state)) =
        camera.get_single_mut_or_none()
    else {
        return;
    };

    let any_movement = || {
        controls.get_pressed().into_iter().any(|a| {
            !matches!(a, GlobalAction::Cancel | GlobalAction::Interact)
        })
    };

    match state.as_deref_mut() {
        // check whether the camera needs to start syncing with the player
        None => {
            let bounding_box = Rect::from_center_size(
                camera.translation.truncate(),
                *BOUNDING_BOX_SIZE,
            );

            if !bounding_box.contains(player_pos.translation().truncate()) {
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

                if any_movement() {
                    cmd.entity(camera_entity)
                        .insert(CameraState::StickToPlayer);
                } else {
                    cmd.entity(camera_entity).remove::<CameraState>();
                }
            } else {
                let new_translation = initial_position
                    .lerp(player_pos.translation().truncate(), lerp_factor)
                    .extend(camera.translation.z);
                camera.translation = new_translation;
            }
        }
        // keep on player until they stop moving
        Some(CameraState::StickToPlayer) if any_movement() => {
            let z = camera.translation.z;
            camera.translation = player_pos.translation().truncate().extend(z);
        }
        Some(CameraState::StickToPlayer) => {
            trace!("Player stopped moving, camera stops following her");
            cmd.entity(camera_entity).remove::<CameraState>();

            let cam_pos = camera.translation.truncate();

            let move_in_direction = Vec2::from(actor.direction)
                * NUDGE_CAMERA_TOWARDS_PLAYER_DIRECTION_BY_PXS;

            events.send(BeginInterpolationEvent::of_translation(
                camera_entity,
                None,
                cam_pos + move_in_direction,
            ));
        }
    };
}
