use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    render::{camera::RenderTarget, view::RenderLayers},
};
use bevy_magic_light_2d::{
    gi::{compositing::CameraTargets, LightScene},
    SceneCamera,
};
use bevy_pixel_camera::{PixelViewport, PixelZoom};

use crate::prelude::*;

pub(crate) const PIXEL_ZOOM: f32 = 3.0;

pub(crate) const OBJ_RENDER_LAYER: u8 = 1;
pub(crate) const BG_RENDER_LAYER: u8 = 2;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        BackgroundLightScene::build(app);
    }

    fn finish(&self, app: &mut App) {
        BackgroundLightScene::finish(app);
    }
}

#[derive(Component, Default, Clone, TypePath)]
pub(crate) struct BackgroundLightScene;

impl LightScene for BackgroundLightScene {
    const HANDLE_START: u128 = 23475629871623176235;

    fn render_layer_index() -> u8 {
        (RenderLayers::TOTAL_LAYERS - 2) as u8
    }

    fn camera_order() -> isize {
        1
    }
}

fn spawn(
    mut commands: Commands,
    bg_camera_targets: Res<CameraTargets<BackgroundLightScene>>,
) {
    commands.spawn((
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

    commands
        .spawn((
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
