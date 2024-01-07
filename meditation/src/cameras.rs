use bevy::{
    core_pipeline::{bloom::BloomSettings, clear_color::ClearColorConfig},
    render::{camera::RenderTarget, view::RenderLayers},
};
use bevy_magic_light_2d::{
    gi::{
        compositing::{setup_post_processing_quad, CameraTargets},
        LightScene,
    },
    SceneCamera,
};
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use main_game_lib::PIXEL_ZOOM;

use crate::prelude::*;

pub(crate) const OBJ_RENDER_LAYER: u8 = 1;
pub(crate) const BG_RENDER_LAYER: u8 = 2;

/// All entities with this component are part of the background light scene.
/// Also, they will get despawned by query in this plugin.
#[derive(Component, Default, Clone, TypePath)]
pub(crate) struct BackgroundLightScene;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GlobalGameState::MeditationLoading),
            (
                BackgroundLightScene::setup_post_processing_quad_system(),
                spawn_bg_render_camera
                    .after(setup_post_processing_quad::<BackgroundLightScene>),
            ),
        );
        app.add_systems(
            OnEnter(GlobalGameState::MeditationInGame),
            spawn_cameras,
        );
        app.add_systems(OnEnter(GlobalGameState::MeditationQuitting), despawn);

        BackgroundLightScene::build(app);
    }

    fn finish(&self, app: &mut App) {
        BackgroundLightScene::finish(app);
    }
}

impl LightScene for BackgroundLightScene {
    const HANDLE_START: u128 = 23475629871623176235;

    fn render_layer_index() -> u8 {
        (RenderLayers::TOTAL_LAYERS - 2) as u8
    }
}

/// These cameras render into the game window.
/// We spawn them last after everything else is ready to avoid things just
/// popping into existence X frames later.
fn spawn_cameras(mut commands: Commands) {
    info!("Spawning cameras");

    commands.spawn((
        BackgroundLightScene,
        Camera2dBundle {
            camera: Camera {
                order: 1,
                hdr: true,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        BloomSettings {
            intensity: 0.1,
            ..default()
        },
        RenderLayers::layer(BackgroundLightScene::render_layer_index()),
        UiCameraConfig { show_ui: false },
    ));

    commands.spawn((
        BackgroundLightScene,
        PixelZoom::Fixed(PIXEL_ZOOM as i32),
        PixelViewport,
        RenderLayers::layer(OBJ_RENDER_LAYER),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: 2,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
    ));
}

/// This camera does not render into a window, but into a quad that's then
/// rendered by whichever camera renders the layer [`LightScene::render_layer`].
fn spawn_bg_render_camera(
    mut commands: Commands,
    bg_camera_targets: Res<CameraTargets<BackgroundLightScene>>,
) {
    info!("Spawning bg render camera");

    commands
        .spawn((
            BackgroundLightScene,
            SceneCamera::<BackgroundLightScene>::default(),
            PixelZoom::Fixed(PIXEL_ZOOM as i32),
            PixelViewport,
            RenderLayers::from_layers(&[0, BG_RENDER_LAYER]),
            UiCameraConfig { show_ui: false },
        ))
        .insert(Camera2dBundle {
            camera: Camera {
                hdr: true,
                target: RenderTarget::Image(
                    bg_camera_targets.floor_target.clone(),
                ),
                ..default()
            },
            projection: OrthographicProjection {
                near: -2000.0,
                ..default()
            },
            ..default()
        });
}

fn despawn(
    mut commands: Commands,
    camera: Query<Entity, With<BackgroundLightScene>>,
) {
    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
