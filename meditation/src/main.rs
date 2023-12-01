mod camera;
mod prelude;
mod weather;

use bevy_pixel_camera::PixelCameraPlugin;
use prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "meditation=trace".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PixelCameraPlugin)
        .insert_resource(ClearColor(Color::hex("#0d0e1f").unwrap()))
        .add_event::<weather::event::LoadedSpecial>()
        .add_event::<weather::event::StartLoadingSpecial>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (apply_velocity, weather::anim::rotate, camera::twinkle),
        )
        .add_systems(
            FixedUpdate,
            (
                weather::controls::normal,
                weather::controls::loading_special,
                weather::anim::apply_bloom,
            )
                .chain(), // important for events
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    camera::spawn_main(&mut commands, &asset_server);
    weather::spawn(&mut commands, &asset_server);
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let d = time.delta_seconds();

    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * d;
        transform.translation.y += vel.y * d;
    }
}
