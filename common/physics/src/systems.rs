use bevy::prelude::*;

use crate::Velocity;

/// You need to register this system in your app.
pub fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * dt;
        transform.translation.y += vel.y * dt;
    }
}
