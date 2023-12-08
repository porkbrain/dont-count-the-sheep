use bevy::prelude::*;

use crate::Velocity;

pub(crate) fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut transform, vel) in &mut query {
        // TODO
        transform.translation.x += vel.x * dt / 2.0;
        transform.translation.y += vel.y * dt / 2.0;
    }
}
