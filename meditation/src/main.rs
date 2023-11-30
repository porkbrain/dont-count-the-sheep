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

fn apply_acceleration(mut query: Query<(&mut Velocity, &mut Acceleration)>, time: Res<Time>) {
    let d = time.delta_seconds();

    for (mut vel, mut acc) in &mut query {
        // apply gravity
        acc.0.y -= consts::GRAVITY_PER_SECOND * d;

        // TODO
        vel.0.x -= vel.0.x * 0.75 * d;
        // acc.x *= 0.8;

        // TODO
        // clamp acceleration
        // acc.0.x = acc.0.x.clamp(-1000.0, 1000.0);
        acc.0.y = acc.0.y.clamp(-1000.0, f32::MAX);
        // // clamp velocity
        // vel.0.x = vel.0.x.clamp(-4000.0, 4000.0);
        vel.0.y = vel.0.y.clamp(-600.0, f32::MAX);

        vel.x += acc.x * d;
        vel.y += acc.y * d;
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * time.delta_seconds();
        transform.translation.y += vel.y * time.delta_seconds();
    }
}
