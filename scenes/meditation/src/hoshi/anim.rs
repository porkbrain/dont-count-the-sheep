use std::{
    cmp::Ordering,
    f32::consts::{E, PI},
};

use main_game_lib::common_ext::QueryExt;

use super::{consts::*, mode, sprite, ActionEvent, HoshiBody, HoshiFace};
use crate::prelude::*;

#[derive(Component)]
pub(super) struct SparkEffect;

/// It always takes the same time to load special.
/// That's because it's a very timing critical animation.
///
/// 1. Abruptly slow down Hoshi to be still.
/// 2. Render spark's atlas first frame in place of Hoshi's body, make it a bit
///    bigger and shrink it to below its normal size.
/// 4. The time it takes to shrink is almost the same as the time it takes to
///    load special. The animation is resumed bit earlier before the special is
///    loaded.
/// 5. Hoshi is off to Mars or wherever while last few frames are playing in
///    place. That's why the effect sprite is not a child of hoshi.
pub(super) fn sprite_loading_special(
    mut cmd: Commands,
    time: Res<Time>,

    mut hoshi: Query<(&mode::LoadingSpecial, &mut Velocity, &Transform)>,
    mut set: ParamSet<(
        Query<
            (Entity, &mut Sprite, &mut TextureAtlas, &mut Transform),
            (
                With<SparkEffect>,
                Without<AtlasAnimationTimer>,
                Without<mode::LoadingSpecial>,
            ),
        >,
        Query<
            &mut TextureAtlas,
            (With<HoshiBody>, Without<mode::LoadingSpecial>),
        >,
        Query<
            &mut TextureAtlas,
            (With<HoshiFace>, Without<mode::LoadingSpecial>),
        >,
    )>,
) {
    let Some((mode, mut vel, transform)) = hoshi.get_single_mut_or_none()
    else {
        return;
    };

    let dt = time.delta_seconds();

    if let Some((
        spark_entity,
        mut spark_sprite,
        mut spark_atlas,
        mut spark_transform,
    )) = set.p0().get_single_mut_or_none()
    {
        let elapsed = mode.activated.elapsed();

        if elapsed > START_SPARK_ANIMATION_AFTER_ELAPSED {
            spark_sprite.custom_size = Some(Vec2::splat(SPARK_SIDE));
            spark_atlas.index = 1;
            cmd.entity(spark_entity).insert(AtlasAnimationTimer::new(
                SPARK_FRAME_TIME,
                TimerMode::Repeating,
            ));
        } else {
            const INITIAL_SIZE: f32 = SPARK_SIDE * 1.25;
            const END_SIZE: f32 = SPARK_SIDE * 0.5;

            let animation_elapsed = elapsed.as_secs_f32()
                / START_SPARK_ANIMATION_AFTER_ELAPSED.as_secs_f32();

            let square_side =
                INITIAL_SIZE - (INITIAL_SIZE - END_SIZE) * animation_elapsed;
            spark_sprite.custom_size = Some(Vec2::splat(square_side));

            // smoothly:
            // vec.x when animation_elapsed 0
            // ...
            // 0.5 * vec.x when 0.5
            // ...
            // 0 when 1
            let dt_prime =
                dt / WHEN_LOADING_SPECIAL_STOP_MOVEMENT_WITHIN.as_secs_f32();
            vel.x -= vel.x * dt_prime;
            vel.y -= vel.y * dt_prime;
            // make spark effect stick with Hoshi until fired
            spark_transform.translation.x = transform.translation.x;
            spark_transform.translation.y = transform.translation.y;
        }
    }

    if let Some(mut body) = set.p1().get_single_mut_or_none() {
        body.index = sprite::BodyKind::Folded.index();
    }

    if let Some(mut face) = set.p2().get_single_mut_or_none() {
        face.index = sprite::FaceKind::TryHarding.index();
    }
}

/// Deciding on what sprite to use is a bit complicated.
/// The sprite is changed based on the last action and the current velocity.
/// Additionally there's a cooldown on the sprite change.
pub(super) fn sprite(
    mut broadcast: EventReader<ActionEvent>,
    mut hoshi: Query<
        (
            &Velocity,
            &AngularVelocity,
            &Transform,
            &mut sprite::Transition,
        ),
        With<mode::Normal>,
    >,
    mut body: Query<&mut TextureAtlas, (With<HoshiBody>, Without<HoshiFace>)>,
    mut face: Query<
        (&mut Visibility, &mut TextureAtlas),
        (With<HoshiFace>, Without<HoshiBody>),
    >,
) {
    let Ok((vel, angvel, transform, mut transition)) = hoshi.get_single_mut()
    else {
        return;
    };
    let mut body = body.single_mut();
    let (mut face_visibility, mut face) = face.single_mut();

    let is_rot_and_vel_aligned =
        is_rotation_aligned_with_velocity(transform, *vel, *angvel, PI / 6.0);
    let is_rot_and_vel_inverse_aligned = true; // TODO

    let latest_action = broadcast.read().last();
    match latest_action {
        Some(ActionEvent::Dipped) => {
            // force? if dips twice in a row, reset timer
            if is_rot_and_vel_inverse_aligned {
                transition.force_update_body(sprite::BodyKind::Plunging);
            }

            // but don't reset face cause you catch me once shame on you
            // catch me twice shame on me
            transition.update_face(sprite::FaceKind::Surprised);
        }
        // TODO: if is_rot_and_vel_aligned (unreliable)
        Some(ActionEvent::DashedAgainstVelocity { towards }) => {
            // I want the booty dance to be shown only if the direction changes
            // fast from right to left and vice versa, ie. player is spamming
            // left and right.
            // * 2 gives the player some time to change direction
            let max_delay = MIN_DASH_AGAINST_VELOCITY_DELAY * 2;
            if let Some(ActionEvent::DashedAgainstVelocity {
                towards: last_towards,
            }) = transition.last_action_within(max_delay)
            {
                if *towards != last_towards {
                    match towards {
                        MotionDirection::Left => {
                            transition
                                .update_body(sprite::BodyKind::BootyDanceLeft);
                        }
                        MotionDirection::Right => {
                            transition
                                .update_body(sprite::BodyKind::BootyDanceRight);
                        }
                        MotionDirection::None => {}
                    }
                }
            }
        }
        // nothing imminent to do, so check the environment
        _ => sprite_under_no_latest_action_of_interest(
            vel,
            &mut transition,
            is_rot_and_vel_inverse_aligned,
            is_rot_and_vel_aligned,
        ),
    }

    if let Some(latest_action) = latest_action {
        transition.update_action(*latest_action);
    }

    body.index = transition.current_body_index();
    face.index = transition.current_face_index();

    *face_visibility = if transition.current_body().should_hide_face() {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}

/// The player did not take an which would warrant some sprite changes.
/// Check the environment and update the sprite accordingly.
fn sprite_under_no_latest_action_of_interest(
    vel: &Velocity,
    transition: &mut sprite::Transition,
    is_rot_and_vel_inverse_aligned: bool,
    is_rot_and_vel_aligned: bool,
) {
    match transition.current_body() {
        sprite::BodyKind::SpearingTowards => {
            let should_be_slowing_down = vel.y
                < VELOCITY_ON_JUMP[MAX_JUMPS - 1]
                && transition.has_elapsed_since_body_change(
                    SHOW_SPEARING_BODY_TOWARDS_FOR,
                );
            if should_be_slowing_down {
                transition
                    .update_body(sprite::BodyKind::SlowingSpearingTowards);
            }
        }
        current_sprite => {
            // some small tolerance, plus because ter. vel. is negative
            let should_be_falling = vel.y <= TERMINAL_VELOCITY + 5.0;

            let should_be_spearing_towards = vel.y
                >= VELOCITY_ON_JUMP[MAX_JUMPS - 1]
                && transition.has_elapsed_since_body_change(
                    SHOW_SPEARING_BODY_TOWARDS_IF_NO_CHANGE_FOR,
                );

            if should_be_falling {
                let min_wait_for_body = match current_sprite {
                    sprite::BodyKind::Default | sprite::BodyKind::Plunging => {
                        SHOW_FALLING_BODY_AFTER / 2
                    }
                    _ => SHOW_FALLING_BODY_AFTER,
                };
                if transition.has_elapsed_since_body_change(min_wait_for_body) {
                    if is_rot_and_vel_inverse_aligned {
                        transition.update_body(sprite::BodyKind::Falling);
                    }

                    let min_wait_for_face = match current_sprite {
                        sprite::BodyKind::Plunging => {
                            SHOW_FALLING_FACE_AFTER / 2
                        }
                        _ => SHOW_FALLING_FACE_AFTER,
                    };

                    if transition
                        .has_elapsed_since_face_change(min_wait_for_face)
                    {
                        transition.update_face(sprite::FaceKind::Intense);
                    }
                }
            } else if should_be_spearing_towards && is_rot_and_vel_aligned {
                transition.update_body(sprite::BodyKind::SpearingTowards);
                transition.update_face(sprite::FaceKind::Happy);
            } else {
                if transition.has_elapsed_since_body_change(
                    SHOW_DEFAULT_BODY_AFTER_IF_NO_CHANGE,
                ) {
                    transition.update_body(sprite::BodyKind::default());
                }
                if transition.has_elapsed_since_body_change(
                    SHOW_DEFAULT_FACE_AFTER_IF_NO_BODY_CHANGE,
                ) {
                    transition.update_face(sprite::FaceKind::default());
                }
            }
        }
    };
}

pub(super) fn rotate(
    mut hoshi: Query<
        (&Velocity, &mut AngularVelocity, &mut Transform),
        With<mode::Normal>,
    >,
    time: Res<Time>,
) {
    let Some((Velocity(vel), mut angvel, mut transform)) =
        hoshi.get_single_mut_or_none()
    else {
        return;
    };

    if vel == &Vec2::ZERO {
        // we call normalize on velocity which is undefined for zero vector
        return;
    }

    const UPRIGHT_ROTATION: Quat = Quat::from_xyzw(0.0, 0.0, 0.0, 1.0);

    let dt = time.delta_seconds();
    let rot = transform.rotation.normalize();

    // the sprite is attracted towards being upright
    let angle_towards_up = {
        // the [`Quat::angle_between`] function returns the angle between
        // two rotations in the range [0, PI], ie. we don't know if we should
        // rotate clockwise or counterclockwise
        // the method we are using here:
        // nudge the rot by the angle and if it's closer to upright then
        // we're going the right way, otherwise we're going the wrong way
        // that's probably an inefficient way to do it but look mum 4 LoC

        let a = rot.angle_between(UPRIGHT_ROTATION);
        let da = (rot * Quat::from_rotation_z(a * dt))
            .angle_between(UPRIGHT_ROTATION);

        let signum = if a < da { -1.0 } else { 1.0 };

        a * signum
    };

    let mut dangvel = angle_towards_up;

    // 0 or +-PI means movement straight up or down
    let a = vel.normalize().angle_between(vec2(0.0, 1.0));
    if a.is_finite() {
        // if a positive ~ 0 => alpha is zero (no rot)
        // if a negative ~ 0 => alpha is zero
        // if a positive ~ PI => alpha is zero
        // if a negative ~ -PI => alpha is zero
        // if a positive ~ PI/2 => alpha is PI/2
        // if a negative ~ -PI/2 => alpha is PI/2
        let unsign_alpha = PI / 2.0 - (a.abs() - PI / 2.0).abs();
        let alpha = unsign_alpha * -a.signum();

        let magnitude_factor = (vel.length() / TERMINAL_VELOCITY).powf(2.0);
        let velocity_boost = alpha * magnitude_factor * ROTATION_SPEED;

        dangvel += velocity_boost;
    }

    // add boost from velocity and attraction towards up
    angvel.0 += dangvel * dt;

    if angvel.0.abs() < 0.075 && UPRIGHT_ROTATION.angle_between(rot) < 0.05 {
        // set upright if it's close enough
        transform.rotation = UPRIGHT_ROTATION;
    } else {
        // e.g. rotating from straight up by PI/2 points to the left
        transform.rotate_z(angvel.0 * dt);
    }

    // slow down rotation over time
    angvel.0 -= angvel.0 * 0.75 * dt;
}

/// Given current rotation as quat (where's the sprite pointing?), current
/// velocity vector and angular velocity which affects the rotation over time,
/// return whether the rotation is approximately aligned with the velocity
/// vector.
///
/// The tolerance is higher if angular velocity is in the direction of the
/// velocity vector.
/// If angular velocity through the roof, then it's not aligned.
/// [This graph](https://www.desmos.com/calculator/14mphdyzxr) shows how we
/// calculate the factor derived from the angular velocity that ultimately
/// scales the tolerance.
///
/// TODO: revisit cause it's flaky
fn is_rotation_aligned_with_velocity(
    transform: &Transform,
    vel: Velocity,
    angvel: AngularVelocity,
    angle_tolerance_basis: f32,
) -> bool {
    fn rotate_by_90deg(quaternion: Quat) -> Quat {
        quaternion * Quat::from_axis_angle(Vec3::Z, PI / 2.0)
    }

    // we need to rotate by axis because the default state is facing right
    // but the sprite is facing up
    let rot = rotate_by_90deg(transform.rotation);
    let direction_vector = rot.mul_vec3(Vec2::X.extend(0.0)).truncate();

    // Rotate by this much counter clock wise to to get from direction to
    // velocity.
    //
    // This would be the optimal angvel.
    let angle = direction_vector.angle_between(*vel);
    if !angle.is_finite() {
        return false;
    }

    // The x coordinate in represents angular velocity.
    // It dictates how fast will the sprite rotate as time goes.
    let x = *angvel;
    // This represents angle between rotation of the sprite and the velocity
    // vector (not to be confused with angular velocity).
    let a = angle;

    // If the angular velocity goes to zero then the sprite won't rotate much so
    // we want to return 1 close to zero. On the other hand as the angular
    // velocity goes in the other direction against the angle x, we again won't
    // be aligned soon.
    //
    // Undefined if a == 0
    fn one_towards_zero(x: f32, a: f32) -> f32 {
        2.0f32.powf((-a + x) / a)
    }
    // If the angular velocity if high, then we can't really speak of an
    // alignment because the sprite will rotate.
    fn steep(x: f32, a: f32) -> f32 {
        // how much do we care about high angular velocity
        const STEEPNESS: f32 = 0.5;
        E.powf(-STEEPNESS * (x - a))
    }

    let factor = 2.0
        * match x.partial_cmp(&a) {
            // it's defined but always 1
            Some(Ordering::Equal) => 1.0,
            // x > a && a > 0
            Some(Ordering::Greater) if a >= 0.0 => steep(x, a),
            // x > a && a < 0
            Some(Ordering::Greater) => one_towards_zero(x, a),
            // x < a && a > 0
            Some(Ordering::Less) if a > 0.0 => one_towards_zero(x, a),
            // x < a && a < 0
            Some(Ordering::Less) => 1.0 / steep(x, a),
            None => return false,
        };
    let adjusted_tolerance = factor * angle_tolerance_basis;

    angle.abs() <= adjusted_tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXACTLY_LEFT: Vec2 = vec2(-1.0, 0.0);
    const EXACTLY_RIGHT: Vec2 = vec2(1.0, 0.0);
    const EXACTLY_DOWN: Vec2 = vec2(0.0, -1.0);

    #[test]
    fn it_is_aligned_if_rotation_exactly_matches_velocity_and_no_angvel() {
        let mut transform = Transform::default();
        transform.rotate_z(PI / 2.0);
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::default(),
            f32::EPSILON
        ));

        let mut transform = Transform::default();
        transform.rotate_z(PI);
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_DOWN.into(),
            AngularVelocity::default(),
            f32::EPSILON
        ));
    }

    #[test]
    fn it_is_aligned_if_rotation_closely_matches_vel_and_no_angvel() {
        let mut transform = Transform::default();

        transform.rotate_z(PI / 2.0 + PI / 12.0); // 105deg
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::default(),
            PI / 6.0 // 30deg tolerance
        ));

        transform.rotate_z(-PI / 6.0); // 105deg - 30deg = 75deg
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::default(),
            PI / 6.0 // 30deg tolerance
        ));

        let mut transform = Transform::default();
        transform.rotate_z(PI + PI / 12.0); // 195deg
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_DOWN.into(),
            AngularVelocity::default(),
            PI / 12.0 * 1.01 // slightly above 15deg tolerance
        ));
    }

    #[test]
    fn it_is_not_aligned_if_rotation_exactly_opposite_velocity_and_no_angvel() {
        let mut transform = Transform::default();
        transform.rotate_z(-PI / 2.0);
        assert!(!is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::default(),
            f32::EPSILON
        ));

        assert!(!is_rotation_aligned_with_velocity(
            &Transform::default(),
            EXACTLY_DOWN.into(),
            AngularVelocity::default(),
            f32::EPSILON
        ));
    }

    #[test]
    fn it_is_aligned_if_angvel_brings_rotation_towards_velocity() {
        let mut transform = Transform::default();
        transform.rotate_z(PI / 2.0 - PI / 12.0); // 75deg left
        assert!(!is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::default(),
            PI / 12.0 * 0.99 // slightly below 15deg tolerance
        ));
        // now we add andvel in the direction of velocity
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::new(PI / 24.0), // 7.5deg
            PI / 12.0 * 0.99
        ));

        let mut transform = Transform::default();
        transform.rotate_z(-PI / 2.0 + PI / 12.0); // 75deg right
        assert!(!is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_RIGHT.into(),
            AngularVelocity::default(),
            PI / 12.0 * 0.99 // slightly below 15deg tolerance
        ));
        // now we add andvel in the direction of velocity
        assert!(is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_RIGHT.into(),
            AngularVelocity::new(-PI / 24.0), // 7.5deg
            PI / 12.0 * 0.99
        ));
    }

    #[test]
    fn it_is_not_aligned_if_angular_velocity_through_the_root() {
        let mut transform = Transform::default();
        transform.rotate_z(PI / 2.0 + 0.001);
        assert!(!is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_LEFT.into(),
            AngularVelocity::new(4.0 * PI),
            PI / 12.0 * 0.99
        ));

        let mut transform = Transform::default();
        transform.rotate_z(-PI / 2.0 + 0.001);
        assert!(!is_rotation_aligned_with_velocity(
            &transform,
            EXACTLY_RIGHT.into(),
            AngularVelocity::new(-4.0 * PI),
            PI / 12.0 * 0.99
        ));
    }
}
