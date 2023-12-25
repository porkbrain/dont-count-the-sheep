//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and move+space to activate
//! the special. The sprite should feel floaty as if you were playing Puff in
//! Smashbros.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod background;
mod climate;
mod control_mode;
mod distractions;
mod gravity;
mod path;
mod prelude;
mod ui;
mod weather;
mod zindex;

mod consts {
    use bevy::render::view::RenderLayers;

    /// What's shown on screen.
    pub(crate) const VISIBLE_WIDTH: f32 = 640.0;
    /// What's shown on screen.
    pub(crate) const VISIBLE_HEIGHT: f32 = 360.0;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_WIDTH: f32 = VISIBLE_WIDTH * 1.25;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_HEIGHT: f32 = VISIBLE_HEIGHT * 1.25;

    pub(crate) const PIXEL_ZOOM: f32 = 3.0;

    pub(crate) const BG_RENDER_LAYER: RenderLayers = RenderLayers::layer(0);
}

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    render::{camera::RenderTarget, view::RenderLayers},
    window::WindowTheme,
};
use bevy_magic_light_2d::{
    gi::{compositing::CameraTargets, BevyMagicLight2DPlugin, LightScene},
    SceneCamera,
};
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};
use prelude::*;

/// TODO: make it a resource
#[derive(Component)]
struct Game;

#[derive(Component)]
struct Paused;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                filter: "meditation=trace,meditation::weather::sprite=debug"
                    .to_string(),
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ciesin".into(),
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    mode: bevy::window::WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        BevyMagicLight2DPlugin,
        PixelCameraPlugin,
        bevy_webp_anim::Plugin,
        common_physics::Plugin,
        common_visuals::Plugin,
        ui::Plugin,
        climate::Plugin,
        distractions::Plugin,
        weather::Plugin,
    ))
    .insert_resource(ClearColor(Color::hex(background::COLOR).unwrap()))
    .insert_resource(gravity::field())
    .add_systems(Startup, (setup, background::spawn));

    BackgroundLightScene::init(&mut app);
    // ObjectsLightScene::init(&mut app);

    common_physics::poissons_equation::register::<gravity::Gravity>(&mut app);

    #[cfg(feature = "dev")]
    app.add_systems(Last, (path::visualize,));

    #[cfg(feature = "dev-poissons")]
    common_physics::poissons_equation::register_visualization::<
        gravity::Gravity,
        gravity::ChangeOfBasis,
        gravity::ChangeOfBasis,
    >(&mut app);

    app.run();
}

#[derive(Component, Default, Clone, TypePath)]
struct BackgroundLightScene; // TODO: Move

impl LightScene for BackgroundLightScene {
    const HANDLE_START: u128 = 23475629871623176235;

    fn render_layer_index() -> u8 {
        (RenderLayers::TOTAL_LAYERS - 2) as u8
    }

    fn camera_order() -> isize {
        1
    }
}

// #[derive(Component, Default, Clone, TypePath)]
// struct ObjectsLightScene; // TODO: Move

// impl LightScene for ObjectsLightScene {
//     const HANDLE_START: u128 = 23475629871623176235;

//     fn render_layer_index() -> u8 {
//         (RenderLayers::TOTAL_LAYERS - 1) as u8
//     }

//     fn camera_order() -> isize {
//         3
//     }
// }

fn setup(
    mut commands: Commands,
    bg_camera_targets: Res<CameraTargets<BackgroundLightScene>>,
    // obj_camera_targets: Res<CameraTargets<ObjectsLightScene>>,
) {
    commands.spawn(Game);

    commands.spawn((
        PixelZoom::Fixed(consts::PIXEL_ZOOM as i32),
        PixelViewport,
        RenderLayers::layer(1),
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

    // commands
    //     .spawn((
    //         SceneCamera::<ObjectsLightScene>::default(),
    //         PixelZoom::Fixed(consts::PIXEL_ZOOM as i32),
    //         PixelViewport,
    //         RenderLayers::layer(1),
    //     ))
    //     .insert(Camera2dBundle {
    //         camera: Camera {
    //             hdr: true,
    //             order: 2,
    //             target: RenderTarget::Image(
    //                 obj_camera_targets.floor_target.clone(),
    //             ),
    //             ..default()
    //         },
    //         projection: OrthographicProjection {
    //             near: -2000.0,
    //             ..default()
    //         },
    //         camera_2d: Camera2d {
    //             clear_color: ClearColorConfig::None,
    //         },
    //         ..default()
    //     });

    commands
        .spawn((
            SceneCamera::<BackgroundLightScene>::default(),
            PixelZoom::Fixed(consts::PIXEL_ZOOM as i32),
            PixelViewport,
            // RenderLayers::layer(2),
        ))
        .insert(Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: 2,
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
