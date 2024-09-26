use bevy::{core_pipeline::bloom::BloomSettings, render::view::RenderLayers};
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_visuals::camera::{
    order, render_layer, MainCamera, PIXEL_VISIBLE_WIDTH, PIXEL_ZOOM,
};
use main_game_lib::common_ext::QueryExt;

use super::{consts::*, ActionEvent};
use crate::{
    hoshi::{Hoshi, HoshiEntity},
    prelude::*,
};

#[derive(Component, Default, Clone)]
pub(super) enum CameraState {
    #[default]
    Normal,
    /// Camera is currently undergoing bloom&zoom effect.
    EffectOnSpecial {
        /// When did the effect start.
        /// Used to calculate phase and smooth out the animation.
        fired: Stopwatch,
        /// Where was the Hoshi when the special was started.
        look_at: Vec2,
    },
}

pub(super) fn spawn(mut cmd: Commands) {
    debug!("Spawning camera");

    cmd.spawn((
        MainCamera,
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::from_layers(&[0, render_layer::OBJ, render_layer::BG]),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::DEFAULT,
                ..default()
            },
            ..default()
        },
        CameraState::default(),
        HoshiEntity,
    ));
}

/// For now we use a simple camera that just follows Hoshi around.
///
/// This needs to be updated to a more complex camera system.
pub(super) fn follow_hoshi(
    mut camera: Query<
        (&CameraState, &mut Transform),
        (With<MainCamera>, Without<Hoshi>),
    >,
    hoshi: Query<&Transform, (With<Hoshi>, Without<MainCamera>)>,
) {
    let Some((CameraState::Normal, mut camera_transform)) =
        camera.get_single_mut_or_none()
    else {
        return;
    };

    if let Some(hoshi) = hoshi.get_single_or_none() {
        let z = camera_transform.translation.z;
        camera_transform.translation = hoshi.translation.truncate().extend(z);
    }
}

/// Zooms in on Hoshi and makes it glow when special is being loaded,
/// then resets to initial state.
///
/// We need to do this for each camera in case there are more.
pub(super) fn zoom_on_special(
    mut cmd: Commands,
    mut action: EventReader<ActionEvent>,
    time: Res<Time>,

    mut state: Query<&mut CameraState>,
    mut cameras: Query<
        (
            Entity,
            &mut Transform,
            &mut OrthographicProjection,
            Option<&mut BloomSettings>,
        ),
        With<MainCamera>,
    >,
) {
    let mut state = state.single_mut();

    let just_started_loading_from_translation = action
        .read()
        .find_map(|e| match e {
            ActionEvent::StartLoadingSpecial {
                at_translation: from_translation,
            } => Some(from_translation),
            _ => None,
        })
        .cloned();

    if let Some(look_at) = just_started_loading_from_translation {
        debug!("Special started loading from {look_at}");
        *state = CameraState::EffectOnSpecial {
            fired: Stopwatch::new(),
            look_at,
        };

        for (entity, _, _, _) in cameras.iter_mut() {
            cmd.entity(entity).insert(BloomSettings {
                intensity: INITIAL_BLOOM_INTENSITY,
                low_frequency_boost: INITIAL_BLOOM_LFB,
                ..default()
            });
        }

        return;
    }

    let CameraState::EffectOnSpecial { fired, look_at } = &mut *state else {
        return;
    };
    fired.tick(time.delta());

    debug_assert!(
        FADE_BLOOM_WHEN_SPECIAL_IS_LOADED_IN
            > FROM_ZOOMED_BACK_TO_NORMAL_WHEN_SPECIAL_IS_LOADED_IN
    );
    if fired.elapsed()
        > FADE_BLOOM_WHEN_SPECIAL_IS_LOADED_IN + SPECIAL_LOADING_TIME
    {
        debug!("Removing bloom and zoom");

        for (entity, mut transform, mut projection, _) in cameras.iter_mut() {
            cmd.entity(entity).remove::<BloomSettings>();
            *state = CameraState::Normal;

            projection.scale = 1.0;
            transform.translation = default();
        }

        return;
    }

    // The camera needs to be clamped horizontally.
    // That's because this game is a vertical scroller, but the player should
    // not see outside of the sides.
    fn freedom_of_camera_translation(scale: f32) -> Vec3 {
        let horizontal_freedom = {
            if scale > 0.999 {
                0.0
            } else {
                PIXEL_VISIBLE_WIDTH * (1.0 - scale) / 2.0
            }
        };

        Vec3::new(horizontal_freedom, f32::MAX, 0.0)
    }

    struct CameraUpdateArgs {
        intensity: f32,
        low_frequency_boost: f32,
        scale: f32,
        /// Used for lerp.
        /// Translates towards this point with some bias where
        translate_towards: Vec3,
        /// Used for lerp.
        /// 1.0 means translate towards it completely.
        translate_bias: f32,
        /// We clamp the translation to this from both sides.
        translate_freedom: Vec3,
    }

    let CameraUpdateArgs {
        intensity,
        low_frequency_boost,
        scale,
        translate_towards,
        translate_bias,
        translate_freedom,
    } = if fired.elapsed() < SPECIAL_LOADING_TIME {
        // we are bloomi'n'zoomin'

        let animation_elapsed =
            fired.elapsed_secs() / SPECIAL_LOADING_TIME.as_secs_f32();

        let new_intensity = INITIAL_BLOOM_INTENSITY
            + (PEAK_BLOOM_INTENSITY - INITIAL_BLOOM_INTENSITY)
                * animation_elapsed;

        let new_lfb = INITIAL_BLOOM_LFB
            + (PEAK_BLOOM_LFB - INITIAL_BLOOM_LFB) * animation_elapsed;

        let new_scale = 1.0 - (1.0 - ZOOM_IN_SCALE) * animation_elapsed;

        let freedom = freedom_of_camera_translation(new_scale);

        CameraUpdateArgs {
            intensity: new_intensity,
            low_frequency_boost: new_lfb,
            scale: new_scale,
            translate_towards: look_at.extend(0.0),
            translate_bias: animation_elapsed,
            translate_freedom: freedom,
        }
    } else {
        // fade bloom and zoom out

        let how_long_after_fired = fired.elapsed() - SPECIAL_LOADING_TIME;

        let animation_elapsed = how_long_after_fired.as_secs_f32()
            / FADE_BLOOM_WHEN_SPECIAL_IS_LOADED_IN.as_secs_f32();

        let new_intensity =
            PEAK_BLOOM_INTENSITY - PEAK_BLOOM_INTENSITY * animation_elapsed;

        let new_lfb = PEAK_BLOOM_LFB - PEAK_BLOOM_LFB * animation_elapsed;

        if how_long_after_fired
            < FROM_ZOOMED_BACK_TO_NORMAL_WHEN_SPECIAL_IS_LOADED_IN
        {
            // zooming out

            let animation_elapsed = how_long_after_fired.as_secs_f32()
                / FROM_ZOOMED_BACK_TO_NORMAL_WHEN_SPECIAL_IS_LOADED_IN
                    .as_secs_f32();

            let new_scale =
                ZOOM_IN_SCALE + (1.0 - ZOOM_IN_SCALE) * animation_elapsed;

            let freedom = freedom_of_camera_translation(new_scale);

            CameraUpdateArgs {
                intensity: new_intensity,
                low_frequency_boost: new_lfb,
                scale: new_scale,
                translate_towards: default(),
                translate_bias: animation_elapsed,
                translate_freedom: freedom,
            }
        } else {
            // zoomed out

            CameraUpdateArgs {
                intensity: new_intensity,
                low_frequency_boost: new_lfb,
                scale: 1.0,
                translate_towards: default(),
                translate_bias: 1.0,
                translate_freedom: default(),
            }
        }
    };

    for (_, mut transform, mut projection, settings) in cameras.iter_mut() {
        let mut settings = settings.expect("bloom settings");
        settings.intensity = intensity;
        settings.low_frequency_boost = low_frequency_boost;
        projection.scale = scale;

        transform.translation = transform
            .translation
            .lerp(translate_towards, translate_bias)
            .clamp(-translate_freedom, translate_freedom);
    }
}
