//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and space to activate special.
//! The sprite should feel floaty as if you were playing Puff in Smashbros.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod background;
mod control_mode;
mod distractions;
mod prelude;
mod ui;
mod weather;
mod zindex;

mod consts {
    pub(crate) const WIDTH: f32 = 630.0;
    pub(crate) const HEIGHT: f32 = 360.0;
    pub(crate) const PIXEL_ZOOM: f32 = 3.0;
}

use bevy::window::WindowTheme;
use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};
use prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter:
                        "meditation=trace,meditation::weather::sprite=debug"
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
            PixelCameraPlugin,
            bevy_webp_anim::Plugin,
            common_physics::Plugin,
            common_visuals::Plugin,
            ui::Plugin,
        ))
        .insert_resource(ClearColor(Color::hex("#0d0e1f").unwrap()))
        .add_event::<weather::ActionEvent>()
        .add_event::<distractions::DistractionDestroyedEvent>()
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, weather::anim::rotate)
        .add_systems(
            Update,
            (
                weather::arrow::point_arrow,
                weather::anim::sprite_loading_special,
                weather::anim::sprite_normal,
                background::shooting_star,
            ),
        )
        .add_systems(
            Update,
            (
                weather::controls::normal,
                weather::controls::loading_special,
                // must be after controls bcs events dependency
                weather::anim::update_camera_on_special,
                // must be after controls bcs events dependency
                distractions::react_to_weather,
                // must be after react_to_weather bcs events dependency
                distractions::destroyed,
            )
                .chain(),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn((
        Camera2dBundle::default(),
        weather::anim::CameraState::default(),
        PixelZoom::Fixed(consts::PIXEL_ZOOM as i32),
        PixelViewport,
    ));

    background::spawn(&mut commands, &asset_server, &mut texture_atlases);
    weather::spawn(&mut commands, &asset_server, &mut texture_atlases);
    distractions::spawn(&mut commands, &asset_server, &mut texture_atlases);
}
