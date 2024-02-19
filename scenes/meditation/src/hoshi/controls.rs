use std::f32::consts::PI;

use bevy::time::Stopwatch;
use common_physics::PoissonsEquation;
use main_game_lib::common_ext::QueryExt;

use super::{anim::SparkEffect, consts::*, mode, ActionEvent};
use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
};

/// Controls when in normal mode.
pub(super) fn normal(
    mut cmd: Commands,
    mut broadcast: EventWriter<ActionEvent>,
    controls: Res<ActionState<GlobalAction>>,
    gravity: Res<PoissonsEquation<Gravity>>,
    time: Res<Time>,

    mut hoshi: Query<
        (Entity, &mut mode::Normal, &mut Velocity, &Transform),
        Without<SparkEffect>,
    >,
    mut spark: Query<(&mut Transform, &mut Visibility), With<SparkEffect>>,
) {
    let Some((entity, mut mode, mut vel, transform)) =
        hoshi.get_single_mut_or_none()
    else {
        return;
    };
    mode.tick(&time);

    if mode.can_use_special && controls.pressed(&GlobalAction::Interact) {
        if let Some(angle) = unit_circle_angle(&controls) {
            debug!("Send loading special");
            broadcast.send(ActionEvent::StartLoadingSpecial {
                at_translation: transform.translation.truncate(),
            });

            cmd.entity(entity).remove::<mode::Normal>();
            cmd.entity(entity).insert(mode::LoadingSpecial {
                angle,
                activated: Stopwatch::default(),
                jumps: mode.jumps,
            });

            let (mut spark_transform, mut spark_visibility) =
                spark.single_mut();
            *spark_visibility = Visibility::Visible;
            *spark_transform = *transform;
            spark_transform.translation.z = zindex::SPARK_EFFECT;

            return;
        }
    }

    let pressed_left = controls.pressed(&GlobalAction::MoveLeft)
        || controls.pressed(&GlobalAction::MoveDownLeft)
        || controls.pressed(&GlobalAction::MoveUpLeft);
    let pressed_right = controls.pressed(&GlobalAction::MoveRight)
        || controls.pressed(&GlobalAction::MoveDownRight)
        || controls.pressed(&GlobalAction::MoveUpRight);
    let pressed_down = controls.pressed(&GlobalAction::MoveDown)
        || controls.pressed(&GlobalAction::MoveDownLeft)
        || controls.pressed(&GlobalAction::MoveDownRight);
    let pressed_up = controls.pressed(&GlobalAction::MoveUp)
        || controls.pressed(&GlobalAction::MoveUpLeft)
        || controls.pressed(&GlobalAction::MoveUpRight);

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

    if mode.jumps >= MAX_JUMPS {
        debug!("God mode: Ability reset");
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
    mut cmd: Commands,
    mut broadcast: EventWriter<ActionEvent>,
    time: Res<Time>,
    controls: Res<ActionState<GlobalAction>>,

    mut hoshi: Query<(Entity, &mut mode::LoadingSpecial, &mut Velocity)>,
) {
    let Some((entity, mut mode, mut vel)) = hoshi.get_single_mut_or_none()
    else {
        return;
    };
    mode.tick(&time);

    let elapsed = mode.activated.elapsed();

    // see whether the player has changed the direction
    if let Some(angle) = unit_circle_angle(&controls) {
        mode.angle = angle;
    }

    if elapsed > SPECIAL_LOADING_TIME {
        cmd.entity(entity).remove::<mode::LoadingSpecial>();
        cmd.entity(entity).insert(mode::Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            last_dash: Stopwatch::default(),
            last_dip: Stopwatch::default(),
            can_use_special: false,
        });

        // fires Hoshi into the direction given by the angle
        vel.x = mode.angle.cos() * VELOCITY_BOOST_ON_SPECIAL;
        vel.y = mode.angle.sin() * VELOCITY_BOOST_ON_SPECIAL;

        broadcast.send(ActionEvent::FiredSpecial);
    }
}

fn unit_circle_angle(a: &ActionState<GlobalAction>) -> Option<Radians> {
    use GlobalAction::*;

    let angle = if a.pressed(&MoveLeft) {
        PI // ←
    } else if a.pressed(&MoveRight) {
        2.0 * PI // →
    } else if a.pressed(&MoveUp) {
        PI / 2.0 // ↑
    } else if a.pressed(&MoveDown) {
        3.0 * PI / 2.0 // ↓
    } else if a.pressed(&MoveUpLeft) {
        3.0 * PI / 4.0 // ↖
    } else if a.pressed(&MoveUpRight) {
        PI / 4.0 // ↗
    } else if a.pressed(&MoveDownRight) {
        7.0 * PI / 4.0 // ↘
    } else if a.pressed(&MoveDownLeft) {
        5.0 * PI / 4.0 // ↙
    } else {
        return None;
    };

    Some(Radians::new(angle))
}
