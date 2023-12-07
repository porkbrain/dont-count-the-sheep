//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and space to activate special.
//! The sprite should feel floaty as if you were playing Puff in Smashbros.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod background;
mod control_mode;
mod distractions;
mod generic;
mod menu;
mod prelude;
mod weather;
mod zindex;

mod consts {
    pub(crate) const WIDTH: f32 = 630.0;
    pub(crate) const HEIGHT: f32 = 360.0;
}

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
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((PixelCameraPlugin,))
        .add_plugins((distractions::WebPAnimationPlugin,))
        .insert_resource(ClearColor(Color::hex("#0d0e1f").unwrap()))
        .add_event::<weather::ActionEvent>()
        .add_event::<distractions::DistractionDestroyedEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (apply_velocity, advance_animation, weather::anim::rotate),
        )
        .add_systems(
            Update,
            (
                weather::arrow::point_arrow,
                weather::anim::sprite_loading_special,
                weather::anim::sprite_normal,
                change_frame_at_random,
                background::twinkle,
                background::shooting_star,
            ),
        )
        .add_systems(
            Update,
            (
                menu::open,
                menu::select, // order important bcs we simulate ESC to close
                menu::close,
                weather::controls::normal,
                weather::controls::loading_special,
                // must be after controls bcs events dependency
                weather::anim::update_camera_on_special,
                distractions::xd,
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
        PixelZoom::Fixed(3),
        PixelViewport,
    ));

    background::spawn(&mut commands, &asset_server, &mut texture_atlases);
    weather::spawn(&mut commands, &asset_server, &mut texture_atlases);
    menu::spawn(&mut commands, &asset_server);
    distractions::spawn(&mut commands, &asset_server, &mut texture_atlases);
}
