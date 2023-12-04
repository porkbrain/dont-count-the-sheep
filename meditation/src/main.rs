//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and space to activate special.
//! The sprite should feel floaty as if you were playing Puff in Smashbros.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod background;
mod control_mode;
mod generic;
mod menu;
mod prelude;
mod weather;
mod zindex;

use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};
use prelude::*;

#[derive(Resource)]
struct NalUnits(usize, Vec<Vec<u8>>);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "meditation=trace".to_string(),
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins((PixelCameraPlugin,))
        .insert_resource(ClearColor(Color::hex("#0d0e1f").unwrap()))
        .add_event::<weather::ActionEvent>()
        .add_systems(Startup, setup)
        .add_systems(First, reset_weather)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                advance_animation,
                weather::anim::rotate,
                weather::anim::apply_bloom,
            ),
        )
        .add_systems(
            Update,
            (
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
}

/// If weather transform translation is within 100.0 of the origin, reset
/// jumps and using special.
/// This is a temp solution.
/// TODO: restore one jump per reset. climate pulsates - only if we are within
/// it in the pulse time do you get one jump.
/// every 4 pulses you get the special back.
fn reset_weather(mut query: Query<(&mut control_mode::Normal, &Transform)>) {
    let Ok((mut controls, transform)) = query.get_single_mut() else {
        return;
    };

    if transform.translation.length() < 100.0 {
        controls.jumps = 0;
        controls.can_use_special = true;
    }
}
