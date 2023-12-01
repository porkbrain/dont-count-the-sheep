use super::{consts, controls, event};
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

pub(crate) fn rotate(
    mut weather: Query<
        (&Velocity, &mut AngularVelocity, &mut Transform),
        With<controls::Normal>,
    >,
    time: Res<Time>,
) {
    const UPRIGHT_ROTATION: Quat = Quat::from_xyzw(0.0, 0.0, 0.0, 1.0);

    let Ok((vel, mut angvel, mut transform)) = weather.get_single_mut() else {
        return;
    };

    let dt = time.delta_seconds();
    let rot = transform.rotation.normalize();

    // the sprite is attracted towards being upright
    let (axis, angle) = rot.to_axis_angle();
    let attract_towards_up = angle % (2.0 * PI)
        * -axis.z.signum()
        * consts::ATTRACTION_TO_UPRIGHT_ROTATION;

    let mut dangvel = attract_towards_up;

    // 0 or +-PI means movement straight up or down
    let a = vel.normalize().angle_between(Vec2::new(0.0, 1.0));
    if a.is_finite() {
        // if a positive ~ 0 => alpha is zero (no rot)
        // if a negative ~ 0 => alpha is zero
        // if a positive ~ PI => alpha is zero
        // if a negative ~ -PI => alpha is zero
        // if a positive ~ PI/2 => alpha is PI/2
        // if a negavite ~ -PI/2 => alpha is PI/2
        let unsign_alpha = PI / 2.0 - (a.abs() - PI / 2.0).abs();
        let alpha = unsign_alpha * -a.signum();

        let velocity_boost = alpha * consts::ROTATION_SPEED;
        dangvel += velocity_boost;
    }

    // add boost from velocity and attraction towards up
    angvel.0 += dangvel * dt;

    if angvel.0.abs() < 0.075 && UPRIGHT_ROTATION.angle_between(rot) < 0.05 {
        // set upright if it's close enough
        transform.rotation = UPRIGHT_ROTATION;
    } else {
        transform.rotate_z(angvel.0 * time.delta_seconds());

        // slow down rotation over time
        angvel.0 -= angvel.0 * 0.75 * dt;
    }
}

/// If the special is loading then bloom effect is applied.
/// It's smoothly animated in and out.
pub(crate) fn apply_bloom(
    mut loading: EventReader<event::StartLoadingSpecial>,
    mut loaded: EventReader<event::LoadedSpecial>,
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

    if let Some(event::LoadedSpecial { fired }) = loaded.read().last() {
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

    if !loading.is_empty() {
        debug!("Special loading started xx");
    }
    let recvd_loading = loading.read().last().is_some();
    if recvd_loading {
        debug!("Special loading started yy");
    }
    if recvd_loading && !matches!(*state, CameraState::BloomGoingUp) {
        debug!("Special has just been triggered");

        *state = CameraState::BloomGoingUp;

        camera.hdr = true;
        *tonemapping = Tonemapping::TonyMcMapface;
        commands.entity(entity).insert(BloomSettings {
            intensity: consts::INITIAL_BLOOM_INTENSITY,
            low_frequency_boost: consts::INITIAL_BLOOM_LFB,
            ..default()
        });
    } else {
        let state_clone = state.clone();

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
                    let mut settings =
                        settings.expect("Bloom settings missing");

                    let remaining_secs = (until - now).as_secs_f32();
                    let remaining_frames =
                        remaining_secs / time.delta_seconds();

                    let new_intensity = settings.intensity
                        - settings.intensity / remaining_frames;

                    // threshold under which we just remove it
                    if new_intensity < 0.05 {
                        remove_bloom();
                    } else {
                        settings.intensity = new_intensity;

                        let new_low_frequency_boost = settings
                            .low_frequency_boost
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
                    + consts::BLOOM_LFB_INCREASE_PER_SECOND
                        * time.delta_seconds())
                .min(0.75);
            }
            CameraState::Normal => debug_assert!(settings.is_none()),
        }
    }
}
