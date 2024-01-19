use bevy::render::view::RenderLayers;
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_visuals::camera::{
    order, render_layer, PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH, PIXEL_ZOOM,
};
use lazy_static::lazy_static;
use main_game_lib::common_visuals::EASE_IN_OUT;

use crate::prelude::*;

lazy_static! {
    /// If the player leaves this bounding box, the camera follows her.
    /// The box is centered at camera position.
    static ref BOUNDING_BOX_SIZE: Vec2 = {

        1.5 * vec2(PIXEL_VISIBLE_WIDTH, PIXEL_VISIBLE_HEIGHT)
    / //------------------------------------------------------
                            3.0

    };
}

const SYNCING_DURATION: Duration = from_millis(2000);

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::DowntownLoading), spawn)
            .add_systems(OnExit(GlobalGameState::DowntownQuitting), despawn);

        app.add_systems(
            Update,
            (
                check_whether_needed_to_sync_with_player,
                sync_with_player,
                stick_to_player_until_she_stops,
            )
                .run_if(in_state(GlobalGameState::AtDowntown))
                .chain(),
        );
    }
}

#[derive(Component)]
struct CameraEntity;

fn spawn(mut cmd: Commands) {
    debug!("Spawning camera");

    cmd.spawn((
        Name::from("Downtown camera"),
        CameraEntity,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::from_layers(&[render_layer::BG, render_layer::OBJ]),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::DEFAULT,
                ..default()
            },
            ..default()
        },
    ));

    #[cfg(feature = "dev")]
    cmd.spawn((
        Name::from("Downtown debug camera"),
        CameraEntity,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::from_layers(&[
            0, // for FPS and other debug tools
        ]),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::DEV,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color:
                    bevy::core_pipeline::clear_color::ClearColorConfig::None,
            },
            ..default()
        },
    ));
}

fn despawn(mut cmd: Commands, bg: Query<Entity, With<CameraEntity>>) {
    debug!("Despawning camera");

    for entity in bg.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

/// If the player leaves a bounding box defined with [`BOUNDING_BOX_SIZE`],
/// this state is inserted with the [`CameraState::SyncWithPlayer`] variant.
#[derive(Component)]
enum CameraState {
    /// While camera state is this, cameras are translated towards the player
    /// over the course of [`SYNCING_DURATION`].
    ///
    /// Then, the state is changed to [`CameraState::StickToPlayer`].
    SyncWithPlayer {
        initial_position: Pos2,
        animation_timer: Stopwatch,
    },
    /// While camera state is this, cameras are translated to the player's
    /// position every frame.
    /// This ends when the player stops moving.
    StickToPlayer,
}

fn check_whether_needed_to_sync_with_player(
    mut cmd: Commands,

    state: Query<&CameraState>, // must be empty
    player: Query<&Transform, (Without<CameraEntity>, With<Player>)>,
    cameras: Query<&Transform, (With<CameraEntity>, Without<Player>)>,
) {
    if !state.is_empty() {
        return;
    }

    let Some(some_camera) = cameras.iter().next() else {
        return;
    };

    let player = player.single();

    let bounding_box = Rect::from_center_size(
        some_camera.translation.truncate(),
        *BOUNDING_BOX_SIZE,
    );

    if bounding_box.contains(player.translation.truncate()) {
        return;
    }

    trace!("Player left the bounding box, camera follows her");
    cmd.spawn(CameraState::SyncWithPlayer {
        initial_position: some_camera.translation.truncate(),
        animation_timer: Stopwatch::new(),
    });
}

fn sync_with_player(
    time: Res<Time>,

    mut state: Query<&mut CameraState>,
    player: Query<&Transform, (Without<CameraEntity>, With<Player>)>,
    mut cameras: Query<&mut Transform, (With<CameraEntity>, Without<Player>)>,
) {
    let Ok(mut state) = state.get_single_mut() else {
        return;
    };

    let CameraState::SyncWithPlayer {
        initial_position,
        animation_timer,
    } = &mut *state
    else {
        return;
    };
    animation_timer.tick(time.delta());

    let lerp_factor = EASE_IN_OUT.ease(
        animation_timer.elapsed().as_secs_f32()
            / SYNCING_DURATION.as_secs_f32(),
    );

    if lerp_factor >= 1.0 - f32::EPSILON {
        trace!("Camera is now synced with player");
        *state = CameraState::StickToPlayer;
        return;
    }

    let player = player.single();

    let new_translation = initial_position
        .lerp(player.translation.truncate(), lerp_factor)
        .extend(0.0);

    for mut camera in cameras.iter_mut() {
        camera.translation = new_translation;
    }
}

fn stick_to_player_until_she_stops(
    mut cmd: Commands,
    controls: Res<ActionState<GlobalAction>>,

    state: Query<(Entity, &CameraState)>,
    player: Query<&Transform, (Without<CameraEntity>, With<Player>)>,
    mut cameras: Query<&mut Transform, (With<CameraEntity>, Without<Player>)>,
) {
    let Ok((state_entity, CameraState::StickToPlayer)) = state.get_single()
    else {
        return;
    };

    let player = player.single();

    let any_movement = controls
        .get_pressed()
        .into_iter()
        .any(|a| !matches!(a, GlobalAction::Cancel | GlobalAction::Interact));

    if !any_movement {
        trace!("Player stopped moving, camera stops following her");
        cmd.entity(state_entity).despawn();
    } else {
        for mut camera in cameras.iter_mut() {
            camera.translation = player.translation.truncate().extend(0.0);
        }
    }
}
