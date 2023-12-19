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
    /// What's shown on screen.
    pub(crate) const VISIBLE_WIDTH: f32 = 640.0;
    /// What's shown on screen.
    pub(crate) const VISIBLE_HEIGHT: f32 = 360.0;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_WIDTH: f32 = VISIBLE_WIDTH * 1.25;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_HEIGHT: f32 = VISIBLE_HEIGHT * 1.25;

    pub(crate) const PIXEL_ZOOM: f32 = 3.0;
}

use bevy::{render::camera::RenderTarget, window::WindowTheme};
use bevy_magic_light_2d::{
    gi::{compositing::CameraTargets, BevyMagicLight2DPlugin},
    FloorCamera, ObjectsCamera, WallsCamera,
};
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};
use prelude::*;

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
    ))
    .insert_resource(ClearColor(Color::hex(background::COLOR).unwrap()))
    .insert_resource(gravity::field())
    .add_event::<weather::ActionEvent>()
    .add_event::<distractions::DistractionDestroyedEvent>()
    .add_systems(
        Startup,
        (
            setup,
            background::spawn,
            distractions::spawn,
            weather::spawn,
        ),
    )
    .add_systems(FixedUpdate, (weather::anim::rotate, distractions::rotate))
    .add_systems(
        Update,
        (
            weather::arrow::point_arrow,
            weather::anim::sprite_loading_special,
            distractions::follow_curve,
        ),
    )
    .add_systems(
        Update,
        (
            weather::controls::normal,
            weather::controls::loading_special,
            // must be after controls bcs events dependency
            weather::anim::update_camera_on_special,
            weather::anim::sprite_normal,
            distractions::react_to_weather,
            // must be after react_to_weather bcs events dependency
            distractions::destroyed,
        )
            .chain(),
    );
    // TODO: dev feature
    // .add_systems(
    //     Last,
    //     (
    //         path::visualize,
    //     )
    // );

    common_physics::poissons_equation::register::<gravity::Gravity>(&mut app);
    #[cfg(feature = "dev")]
    common_physics::poissons_equation::register_visualization::<
        gravity::Gravity,
        gravity::ChangeOfBasis,
        gravity::ChangeOfBasis,
    >(&mut app);

    app.run();
}

fn setup(mut commands: Commands, camera_targets: Res<CameraTargets>) {
    let zoom = PixelZoom::Fixed(consts::PIXEL_ZOOM as i32);

    commands
        .spawn(Name::new("camera_main"))
        .insert(Camera2dBundle::default())
        .insert(zoom.clone())
        .insert(PixelViewport)
        .insert(RenderLayers::layer(3)); // TODO

    commands
        .spawn(Name::new("camera_light"))
        .insert(Camera2dBundle {
            camera: Camera {
                hdr: true,
                target: RenderTarget::Image(
                    camera_targets.objects_target.clone(),
                ),
                ..default()
            },
            projection: OrthographicProjection {
                near: -2000.0,
                ..default()
            },
            ..default()
        })
        .insert(zoom.clone())
        .insert(PixelViewport)
        .insert(ObjectsCamera)
        .insert(WallsCamera)
        .insert(FloorCamera)
        .insert(RenderLayers::layer(0)) // TODO
        .insert(UiCameraConfig { show_ui: false });
}
