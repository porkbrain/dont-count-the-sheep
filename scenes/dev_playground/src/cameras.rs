use bevy::render::view::RenderLayers;
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_visuals::camera::{order, render_layer, PIXEL_ZOOM};
use main_game_lib::common_visuals::camera::MainCamera;

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::Blank), spawn)
            .add_systems(OnExit(GlobalGameState::InDevPlayground), despawn);

        app.add_systems(
            Update,
            common_top_down::cameras::track_player_with_main_camera
                .run_if(in_state(GlobalGameState::InDevPlayground)),
        );
    }
}

#[derive(Component)]
pub(crate) struct CameraEntity;

fn spawn(mut cmd: Commands) {
    debug!("Spawning camera");

    cmd.spawn((
        Name::from("Dev plaground camera"),
        MainCamera,
        CameraEntity,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::from_layers(&[
            0, // for FPS and other debug tools
            render_layer::BG,
            render_layer::OBJ,
        ]),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::DEFAULT,
                ..default()
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
