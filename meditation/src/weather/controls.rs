use super::{anim::SparkEffect, consts::*, ActionEvent};
use crate::{
    control_mode,
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
};
use bevy::time::Stopwatch;
use common_physics::PoissonsEquation;
use std::f32::consts::PI;

/// Controls when in normal mode.
pub(super) fn normal(
    game: Query<&Game, Without<Paused>>,
    mut broadcast: EventWriter<ActionEvent>,
    mut weather: Query<
        (Entity, &mut control_mode::Normal, &mut Velocity, &Transform),
        Without<SparkEffect>, // to make bevy be sure there won't be conflicts
    >,
    mut spark: Query<(&mut Transform, &mut Visibility), With<SparkEffect>>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    gravity: Res<PoissonsEquation<Gravity>>,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

    let Ok((entity, mut mode, mut vel, transform)) = weather.get_single_mut()
    else {
        return;
    };
    mode.tick(&time);

    let pressed_space = keyboard.pressed(KeyCode::Space);
    let pressed_left =
        keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
    let pressed_right =
        keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);
    let pressed_down =
        keyboard.pressed(KeyCode::Down) || keyboard.pressed(KeyCode::S);
    let pressed_up =
        keyboard.pressed(KeyCode::Up) || keyboard.pressed(KeyCode::W);

    if mode.can_use_special && pressed_space {
        if let Some(angle) = unit_circle_angle(&keyboard) {
            debug!("Send loading special");
            broadcast.send(ActionEvent::StartLoadingSpecial {
                at_translation: transform.translation.truncate(),
            });

            commands.entity(entity).remove::<control_mode::Normal>();
            commands
                .entity(entity)
                .insert(control_mode::LoadingSpecial {
                    angle,
                    activated: Stopwatch::default(),
                    jumps: mode.jumps,
                    god_mode: mode.god_mode,
                });

            let (mut spark_transform, mut spark_visibility) =
                spark.single_mut();
            *spark_visibility = Visibility::Visible;
            *spark_transform = *transform;
            spark_transform.translation.z = zindex::SPARK_EFFECT;

            return;
        }
    }

    let dt = time.delta_seconds();
    let gvec = gravity.gradient_at(ChangeOfBasis::from(*transform))
        * GRAVITY_MULTIPLIER;

    let mut update_horizontal = |dir: MotionDirection| {
        let is_moving_in_opposite_direction = !dir.is_aligned(vel.x);

        if mode.last_dash.elapsed() > MIN_DASH_DELAY
            || (is_moving_in_opposite_direction
                && mode.last_dash.elapsed() > MIN_DASH_AGAINST_VELOCITY_DELAY)
        {
            mode.last_dash = Stopwatch::new();

            // velocity boost is even stronger when moving in opposite direction
            let directional_boost = if is_moving_in_opposite_direction {
                2.0
            } else {
                1.0
            };

            let vel_cap = match dir {
                MotionDirection::Left => vel.x.min(0.0),
                MotionDirection::Right => vel.x.max(0.0),
                MotionDirection::None => unreachable!(),
            };

            vel.x =
                vel_cap + dir.sign() * directional_boost * DASH_VELOCITY_BOOST;

            // if moving back and forth, fall slower
            if is_moving_in_opposite_direction && vel.y < 0.0 {
                vel.y /= 2.0;
                broadcast
                    .send(ActionEvent::DashedAgainstVelocity { towards: dir });
            }
        }
    };

    if pressed_left && !pressed_right {
        update_horizontal(MotionDirection::Left);
    }
    if pressed_right && !pressed_left {
        update_horizontal(MotionDirection::Right);
    }

    if pressed_down && mode.last_dip.elapsed() > MIN_DIP_DELAY {
        // dip

        mode.last_dip = Stopwatch::new();
        broadcast.send(ActionEvent::Dipped);

        // the downward movement is stabilized
        vel.y = VERTICAL_VELOCITY_ON_DIP;

        if pressed_left {
            vel.x -= HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
        if pressed_right {
            vel.x += HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
    } else if vel.y < TERMINAL_VELOCITY {
        // slow down to terminal velocity

        vel.y += {
            debug_assert!(TERMINAL_VELOCITY < 0.0); // => vel.y < 0.0

            let diff = -vel.y + TERMINAL_VELOCITY;
            // always slow down at least 1 pixel per second to avoid
            // infinite approach
            (diff * dt * SLOWDOWN_TO_TERMINAL_VELOCITY_FACTOR).max(1.0)
        };
    } else {
        // apply gravity

        vel.y = (vel.y + gvec.y * dt).max(TERMINAL_VELOCITY);
    }

    if mode.god_mode && mode.jumps >= MAX_JUMPS {
        debug!("Ability reset");
        mode.jumps = 0;
        mode.can_use_special = true;
    }

    if pressed_up
        && mode.jumps < MAX_JUMPS
        && mode.last_jump.elapsed() > MIN_JUMP_DELAY
    {
        mode.jumps += 1;
        mode.last_jump = Stopwatch::new();
        broadcast.send(ActionEvent::Jumped);

        vel.y = VELOCITY_ON_JUMP[mode.jumps - 1];

        if pressed_left {
            vel.x -= HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
        if pressed_right {
            vel.x += HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
    }

    // apply gravity to the horizontal movement
    vel.x += gvec.x * dt;
    // apply friction to the horizontal movement
    vel.x -= vel.x * dt;
}

/// Controls while loading special.
pub(crate) fn loading_special(
    game: Query<&Game, Without<Paused>>,
    mut broadcast: EventWriter<ActionEvent>,
    mut weather: Query<(
        Entity,
        &mut control_mode::LoadingSpecial,
        &mut Velocity,
    )>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

    let Ok((entity, mut mode, mut vel)) = weather.get_single_mut() else {
        return;
    };
    mode.tick(&time);

    let elapsed = mode.activated.elapsed();

    // see whether the player has changed the direction
    if let Some(angle) = unit_circle_angle(&keyboard) {
        mode.angle = angle;
    }

    if elapsed > SPECIAL_LOADING_TIME {
        commands
            .entity(entity)
            .remove::<control_mode::LoadingSpecial>();
        commands.entity(entity).insert(control_mode::Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            last_dash: Stopwatch::default(),
            last_dip: Stopwatch::default(),
            can_use_special: false,
            god_mode: mode.god_mode,
        });

        // fires weather into the direction given by the angle
        vel.x = mode.angle.cos() * VELOCITY_BOOST_ON_SPECIAL;
        vel.y = mode.angle.sin() * VELOCITY_BOOST_ON_SPECIAL;

        broadcast.send(ActionEvent::FiredSpecial);
    }
}

fn unit_circle_angle(key: &Input<KeyCode>) -> Option<Radians> {
    use KeyCode::*;
    let pressed_left = key.pressed(Left)
        || key.just_released(Left)
        || key.pressed(A)
        || key.just_released(A);
    let pressed_right = key.pressed(Right)
        || key.just_released(Right)
        || key.pressed(D)
        || key.just_released(D);
    let pressed_down = key.pressed(Down)
        || key.just_released(Down)
        || key.pressed(S)
        || key.just_released(S);
    let pressed_up = key.pressed(Up)
        || key.just_released(Up)
        || key.pressed(W)
        || key.just_released(W);

    let angle = if pressed_left && !pressed_right {
        if pressed_up && !pressed_down {
            3.0 * PI / 4.0 // ←↑ = ↖
        } else if pressed_down && !pressed_up {
            5.0 * PI / 4.0 // ←↓ = ↙
        } else {
            PI // ←
        }
    } else if pressed_right && !pressed_left {
        if pressed_up && !pressed_down {
            PI / 4.0 // ↑→ = ↗
        } else if pressed_down && !pressed_up {
            7.0 * PI / 4.0 // ↓→ = ↘
        } else {
            2.0 * PI // →
        }
    } else if pressed_down && !pressed_up {
        3.0 * PI / 2.0 // ↓
    } else if pressed_up && !pressed_down {
        PI / 2.0 // ↑ (default)
    } else {
        return None;
    };

    Some(Radians::new(angle))
}
