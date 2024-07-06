use bevy::render::view::RenderLayers;
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_loading_screen::LoadingScreenState;
use common_visuals::camera::{order, render_layer, PIXEL_ZOOM};

use crate::prelude::*;

/// All entities with this component are part of the background light scene.
/// Also, they will get despawned by query in this plugin.
#[derive(Component, Default, Clone, TypePath)]
pub(crate) struct BackgroundLightScene;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(LoadingScreenState::WaitForSignalToFinish),
            spawn_cameras.run_if(in_state(GlobalGameState::LoadingMeditation)),
        );
        app.add_systems(OnExit(GlobalGameState::QuittingMeditation), despawn);
    }
}

/// These cameras render into the game window.
/// We spawn them last after everything else is ready to avoid things just
/// popping into existence X frames later.
fn spawn_cameras(mut cmd: Commands) {
    debug!("Spawning cameras");

    cmd.spawn((
        BackgroundLightScene,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::layer(render_layer::OBJ),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::LIGHT,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
    ));
}

fn despawn(
    mut cmd: Commands,
    entities: Query<Entity, With<BackgroundLightScene>>,
) {
    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
