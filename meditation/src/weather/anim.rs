use super::{consts, controls, sprite, ActionEvent, WeatherBody, WeatherFace};
use crate::prelude::*;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    utils::Instant,
};
use std::f32::consts::PI;

#[derive(Component, Default, Clone, Copy)]
pub(crate) enum CameraState {
    #[default]
    Normal,
    BloomGoingDown {
        until: Instant,
    },
    BloomGoingUp,
}

/// Deciding on what sprite to use is a bit complicated.
/// The sprite is changed based on the last action and the current velocity.
/// Additionally there's a cooldown on the sprite change.
///
/// TODO: condition on rotation and direction of motion
pub(crate) fn sprite(
    mut broadcast: EventReader<ActionEvent>,
    mut weather: Query<
        (&Velocity, &mut sprite::Transition),
        With<controls::Normal>,
    >,
    mut body: Query<
        &mut TextureAtlasSprite,
        (With<WeatherBody>, Without<WeatherFace>),
    >,
    mut face: Query<
        (&mut Visibility, &mut TextureAtlasSprite),
        (With<WeatherFace>, Without<WeatherBody>),
    >,
) {
    let Ok((vel, mut transition)) = weather.get_single_mut() else {
        return;
    };
    let Ok(mut body) = body.get_single_mut() else {
        return;
    };
    let Ok((mut face_visibility, mut face)) = face.get_single_mut() else {
        return;
    };

    let latest_action = broadcast.read().last();
    match latest_action {
        Some(ActionEvent::Dipped) => {
            // force? if dips twice in a row, reset timer
            transition.force_update_body(sprite::BodyKind::Plunging);
            // but don't reset face cause you catch me once shame on you
            // catch me twice shame on me
            transition.update_face(sprite::FaceKind::Surprised);
        }
        Some(ActionEvent::DashedAgainstVelocity { towards }) => {
            // I want the booty dance to be shown only if the direction changes
            // fast from right to left and vice versa, ie. player is spamming
            // left and right.
            // * 2 gives the player some time to change direction
            let max_delay = consts::MIN_DASH_AGAINST_VELOCITY_DELAY * 2;
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
        _ => {
            match transition.current_body() {
                sprite::BodyKind::SpearingTowards => {
                    let should_be_slowing_down = vel.y
                        < consts::BASIS_VELOCITY_ON_JUMP
                        && transition.has_elapsed_since_body_change(
                            std::time::Duration::from_millis(500), // TODO
                        );
                    if should_be_slowing_down {
                        transition.update_body(
                            sprite::BodyKind::SlowingSpearingTowards,
                        );
                    }
                }
                current_sprite => {
                    let should_be_falling =
                        vel.y <= consts::TERMINAL_VELOCITY + 5.0; // some tolerance
                    let should_be_spearing_towards = vel.y
                        >= consts::BASIS_VELOCITY_ON_JUMP
                        && transition.has_elapsed_since_body_change(
                            std::time::Duration::from_millis(250), // TODO
                        );

                    if should_be_falling {
                        let min_wait_for_body = match current_sprite {
                            sprite::BodyKind::Default
                            | sprite::BodyKind::Plunging => {
                                consts::SHOW_FALLING_BODY_AFTER / 2
                            }
                            _ => consts::SHOW_FALLING_BODY_AFTER,
                        };
                        if transition
                            .has_elapsed_since_body_change(min_wait_for_body)
                        {
                            transition.update_body(sprite::BodyKind::Falling);

                            let min_wait_for_face = match current_sprite {
                                sprite::BodyKind::Plunging => {
                                    consts::SHOW_FALLING_FACE_AFTER / 2
                                }
                                _ => consts::SHOW_FALLING_FACE_AFTER,
                            };

                            if transition.has_elapsed_since_face_change(
                                min_wait_for_face,
                            ) {
                                transition
                                    .update_face(sprite::FaceKind::Intense);
                            }
                        }
                    } else if should_be_spearing_towards {
                        transition
                            .update_body(sprite::BodyKind::SpearingTowards);
                        transition.update_face(sprite::FaceKind::Happy);
                    } else {
                        if transition.has_elapsed_since_body_change(
                            consts::SHOW_DEFAULT_BODY_AFTER,
                        ) {
                            transition.update_body(sprite::BodyKind::default());
                        }
                        if transition.has_elapsed_since_body_change(
                            consts::SHOW_DEFAULT_FACE_AFTER,
                        ) {
                            transition.update_face(sprite::FaceKind::default());
                        }
                    }
                }
            };
        }
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

pub(crate) fn rotate(
    mut weather: Query<
        (&Velocity, &mut AngularVelocity, &mut Transform),
        With<controls::Normal>,
    >,
    time: Res<Time>,
) {
    let Ok((vel, mut angvel, mut transform)) = weather.get_single_mut() else {
        return;
    };

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
    let a = vel.normalize().angle_between(Vec2::new(0.0, 1.0));
    if a.is_finite() {
        // if a positive ~ 0 => alpha is zero (no rot)
        // if a negative ~ 0 => alpha is zero
        // if a positive ~ PI => alpha is zero
        // if a negative ~ -PI => alpha is zero
        // if a positive ~ PI/2 => alpha is PI/2
        // if a negative ~ -PI/2 => alpha is PI/2
        let unsign_alpha = PI / 2.0 - (a.abs() - PI / 2.0).abs();
        let alpha = unsign_alpha * -a.signum();

        let magnitude_factor =
            (vel.length() / consts::TERMINAL_VELOCITY).powf(2.0);
        let velocity_boost = alpha * magnitude_factor * consts::ROTATION_SPEED;

        dangvel += velocity_boost;
    }

    // add boost from velocity and attraction towards up
    angvel.0 += dangvel * dt;

    if angvel.0.abs() < 0.075 && UPRIGHT_ROTATION.angle_between(rot) < 0.05 {
        // set upright if it's close enough
        transform.rotation = UPRIGHT_ROTATION;
    } else {
        transform.rotate_z(angvel.0 * dt);

        // slow down rotation over time
        angvel.0 -= angvel.0 * 0.75 * dt;
    }
}

/// If the special is loading then bloom effect is applied.
/// It's smoothly animated in and out.
pub(crate) fn apply_bloom(
    mut action: EventReader<ActionEvent>,
    mut camera: Query<(
        Entity,
        &mut Camera,
        &mut CameraState,
        &mut Tonemapping,
        Option<&mut BloomSettings>,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let (entity, mut camera, mut state, mut tonemapping, settings) =
        camera.single_mut();

    let mut just_started_loading = false;
    for event in action.read() {
        match event {
            ActionEvent::StartLoadingSpecial
                if !matches!(*state, CameraState::BloomGoingUp) =>
            {
                debug!("Special started loading");
                *state = CameraState::BloomGoingUp;

                camera.hdr = true;
                *tonemapping = Tonemapping::TonyMcMapface;
                commands.entity(entity).insert(BloomSettings {
                    intensity: consts::INITIAL_BLOOM_INTENSITY,
                    low_frequency_boost: consts::INITIAL_BLOOM_LFB,
                    ..default()
                });
                just_started_loading = true;
            }
            ActionEvent::LoadedSpecial { fired } => {
                debug!("Special finished loading. Fired? {fired}");

                if matches!(*state, CameraState::BloomGoingUp) {
                    *state = CameraState::BloomGoingDown {
                        until: Instant::now()
                            + if *fired {
                                consts::BLOOM_FADE_OUT_ON_FIRED
                            } else {
                                consts::BLOOM_FADE_OUT_ON_CANCELED
                            },
                    };
                }
            }
            _ => {}
        }
    }

    if just_started_loading {
        return;
    }

    let state_clone = *state;

    let mut remove_bloom = || {
        debug!("Removing bloom");
        commands.entity(entity).remove::<BloomSettings>();
        *state = CameraState::Normal;
        camera.hdr = true;
        *tonemapping = Tonemapping::TonyMcMapface;
    };

    match state_clone {
        CameraState::BloomGoingDown { until } => {
            let now = Instant::now();
            if until < now {
                remove_bloom();
            } else {
                let mut settings = settings.expect("Bloom settings missing");

                let remaining_secs = (until - now).as_secs_f32();
                let remaining_frames = remaining_secs / time.delta_seconds();

                let new_intensity =
                    settings.intensity - settings.intensity / remaining_frames;

                // threshold under which we just remove it
                if new_intensity < 0.05 {
                    remove_bloom();
                } else {
                    settings.intensity = new_intensity;

                    let new_low_frequency_boost = settings.low_frequency_boost
                        - settings.low_frequency_boost / remaining_frames;
                    settings.low_frequency_boost = new_low_frequency_boost;
                }
            }
        }
        CameraState::BloomGoingUp => {
            let mut settings = settings.expect("Bloom settings missing");

            settings.intensity = (settings.intensity
                + consts::BLOOM_INTENSITY_INCREASE_PER_SECOND
                    * time.delta_seconds())
            .min(0.75);
            settings.low_frequency_boost = (settings.low_frequency_boost
                + consts::BLOOM_LFB_INCREASE_PER_SECOND * time.delta_seconds())
            .min(0.75);
        }
        CameraState::Normal => debug_assert!(settings.is_none()),
    }
}
