use bevy::render::view::RenderLayers;
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_loading_screen::LoadingScreenState;
use common_visuals::camera::{order, render_layer, PIXEL_ZOOM};

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(LoadingScreenState::WaitForSignalToFinish),
            spawn.run_if(in_state(GlobalGameState::LoadingMeditation)),
        );
        app.add_systems(OnExit(GlobalGameState::QuittingMeditation), despawn);
    }
}

#[derive(Component)]
pub(crate) struct MeditationCamera;

fn spawn(mut cmd: Commands) {
    debug!("Spawning cameras");

    cmd.spawn((
        MeditationCamera,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::from_layers(&[0, render_layer::OBJ, render_layer::BG]),
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

fn despawn(mut cmd: Commands, camera: Query<Entity, With<MeditationCamera>>) {
    cmd.entity(camera.single()).despawn_recursive();
}
