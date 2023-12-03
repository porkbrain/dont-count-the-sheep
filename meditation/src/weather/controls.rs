use super::{anim::SparkEffect, consts, ActionEvent};
use crate::prelude::*;
use bevy::time::Stopwatch;
use std::f32::consts::PI;

#[derive(Component)]
pub(crate) struct Normal {
    /// weather has a limited number of jumps before it must reset
    /// via the [`Climate`]
    pub(crate) jumps: u8,
    /// there's a minimum delay between jumps
    pub(crate) last_jump: Stopwatch,
    /// there's a minimum delay between dashes
    pub(crate) last_dash: Stopwatch,
    /// there's a minimum delay between dips
    pub(crate) last_dip: Stopwatch,
    /// weather can only use its special ability once per reset
    pub(crate) can_use_special: bool,
}

#[derive(Component, Default)]
pub(crate) struct LoadingSpecial {
    /// Angle is given by the combination of keys pressed.
    /// See [`unit_circle_angle`].
    pub(crate) angle: Radians,
    /// special mode has a set duration after which it fires
    pub(crate) activated: Stopwatch,
    /// once special is fired, weather can only do the same amount of jumps
    /// as it had before
    pub(crate) jumps: u8,
}

/// Controls when in normal mode.
pub(crate) fn normal(
    mut broadcast: EventWriter<ActionEvent>,
    mut weather: Query<
        (Entity, &mut Normal, &mut Velocity, &Transform),
        Without<SparkEffect>, // to make bevy be sure there won't be conflicts
    >,
    mut spark: Query<(&mut Transform, &mut Visibility), With<SparkEffect>>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
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
            broadcast.send(ActionEvent::StartLoadingSpecial);

            commands.entity(entity).remove::<Normal>();
            commands.entity(entity).insert(LoadingSpecial {
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

    let dt = time.delta_seconds();

    let mut update_horizontal = |dir: MotionDirection| {
        let is_moving_in_opposite_direction = !dir.is_aligned(vel.x);

        if mode.last_dash.elapsed() > consts::MIN_DASH_DELAY
            || (is_moving_in_opposite_direction
                && mode.last_dash.elapsed()
                    > consts::MIN_DASH_AGAINST_VELOCITY_DELAY)
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

            vel.x = vel_cap
                + dir.sign() * directional_boost * consts::DASH_VELOCITY_BOOST;

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

    if pressed_down && mode.last_dip.elapsed() > consts::MIN_DIP_DELAY {
        // dip

        mode.last_dip = Stopwatch::new();
        broadcast.send(ActionEvent::Dipped);

        // the downward movement is stabilized
        vel.y = consts::VERTICAL_VELOCITY_ON_DIP;

        if pressed_left {
            vel.x -= consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
        if pressed_right {
            vel.x += consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
    } else if vel.y < consts::TERMINAL_VELOCITY {
        // slow down to terminal velocity

        vel.y += {
            debug_assert!(consts::TERMINAL_VELOCITY < 0.0);

            let diff = -vel.y + consts::TERMINAL_VELOCITY;
            // always slow down at least 1 pixel per second to avoid
            // infinite approach
            (diff * dt).max(1.0)
        };
    } else {
        // apply gravity

        vel.y = (vel.y - consts::GRAVITY * dt).max(consts::TERMINAL_VELOCITY);
    }

    if pressed_up
        && mode.jumps < consts::MAX_JUMPS
        && mode.last_jump.elapsed() > consts::MIN_JUMP_DELAY
    {
        mode.jumps += 1;
        mode.last_jump = Stopwatch::new();

        // each jump is less and less strong until reset
        let jump_boost = (consts::MAX_JUMPS + 1 - mode.jumps) as f32
            / consts::MAX_JUMPS as f32;

        vel.y = consts::BASIS_VELOCITY_ON_JUMP
            + consts::BASIS_VELOCITY_ON_JUMP * jump_boost;

        if pressed_left {
            vel.x -= consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
        if pressed_right {
            vel.x += consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
    }

    // apply friction to the horizontal movement
    vel.x -= vel.x * dt;
}

/// Controls while loading special.
///
/// TODO: hold in place and no cancel
pub(crate) fn loading_special(
    mut broadcast: EventWriter<ActionEvent>,
    mut weather: Query<(Entity, &mut LoadingSpecial, &mut Velocity)>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, mut mode, mut vel)) = weather.get_single_mut() else {
        return;
    };
    mode.tick(&time);

    *vel = Velocity::default();

    let elapsed = mode.activated.elapsed();

    // see whether the player has changed the direction
    if let Some(angle) = unit_circle_angle(&keyboard) {
        mode.angle = angle;
    }

    if elapsed > consts::SPECIAL_LOADING_TIME {
        commands.entity(entity).remove::<LoadingSpecial>();
        commands.entity(entity).insert(Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            last_dash: Stopwatch::default(),
            last_dip: Stopwatch::default(),
            can_use_special: false,
        });

        // fires weather into the direction given by the angle
        vel.x = mode.angle.cos() * consts::VELOCITY_BOOST_ON_SPECIAL;
        vel.y = mode.angle.sin() * consts::VELOCITY_BOOST_ON_SPECIAL;

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

impl Normal {
    fn tick(&mut self, time: &Time) {
        self.last_jump.tick(time.delta());
        self.last_dash.tick(time.delta());
        self.last_dip.tick(time.delta());
    }
}

impl LoadingSpecial {
    fn tick(&mut self, time: &Time) {
        self.activated.tick(time.delta());
    }
}

impl Default for Normal {
    fn default() -> Self {
        Self {
            jumps: 0,
            last_dash: Stopwatch::default(),
            last_jump: Stopwatch::default(),
            last_dip: Stopwatch::default(),
            can_use_special: true,
        }
    }
}
