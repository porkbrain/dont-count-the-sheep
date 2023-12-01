//! Illustrates bloom post-processing in 2d.

mod prelude;
mod weather;

use prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::WARN,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, apply_velocity)
        .add_systems(
            FixedUpdate,
            (weather::control_normal, weather::control_loading_special),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle { ..default() });

    weather::spawn(&mut commands, &mut meshes, &mut materials);
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
