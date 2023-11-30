//! Illustrates bloom post-processing in 2d.

mod consts;
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
        .add_systems(
            FixedUpdate,
            (
                weather::control_normal,
                weather::control_loading_special,
                apply_acceleration,
                apply_velocity,
            )
                .chain(),
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

fn apply_acceleration(
    mut query: Query<(&mut Velocity, &Acceleration)>,
    time: Res<Time>,
) {
    let d = time.delta_seconds();

    for (mut vel, acc) in &mut query {
        vel.x += acc.x * d;
        vel.y += acc.y * d;
    }
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
