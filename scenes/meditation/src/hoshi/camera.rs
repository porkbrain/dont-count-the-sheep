use bevy::{core_pipeline::bloom::BloomSettings, render::view::RenderLayers};
use bevy_pixel_camera::{PixelViewport, PixelZoom};
use common_visuals::camera::{order, render_layer, MainCamera, PIXEL_ZOOM};
use main_game_lib::common_ext::QueryExt;

use super::{consts::*, ActionEvent};
use crate::{
    consts::HALF_LEVEL_WIDTH_PX,
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

/// Will position the camera to look at Hoshi in such a way that we won't have
/// a flicker once the camera starts following Hoshi.
pub(super) fn spawn(
    cmd: &mut Commands,
    window: &Window,
    hoshi_transform: &Transform,
) {
    debug!("Spawning camera");

    let mut camera_transform = Transform::default();
    clamp_camera_to_screen(window, hoshi_transform, &mut camera_transform);

    cmd.spawn((
        MainCamera,
        Name::new("MainCamera"),
        PixelZoom::Fixed(PIXEL_ZOOM),
        PixelViewport,
        RenderLayers::from_layers(&[0, render_layer::OBJ, render_layer::BG]),
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                order: order::DEFAULT,
                ..default()
            },
            transform: camera_transform,
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
    window: Query<&Window>,
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
        clamp_camera_to_screen(window.single(), hoshi, &mut camera_transform);
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

    window: Query<&Window>,
    hoshi: Query<&Transform, (With<Hoshi>, Without<MainCamera>)>,
    mut camera: Query<
        (
            Entity,
            &mut CameraState,
            &mut Transform,
            &mut OrthographicProjection,
            Option<&mut BloomSettings>,
        ),
        (With<MainCamera>, Without<Hoshi>),
    >,
) {
    let Some((
        camera_entity,
        mut camera_state,
        mut camera_transform,
        mut camera_projection,
        camera_bloom,
    )) = camera.get_single_mut_or_none()
    else {
        error!("No camera found");
        return;
    };

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
        *camera_state = CameraState::EffectOnSpecial {
            fired: Stopwatch::new(),
            look_at,
        };

        cmd.entity(camera_entity).insert(BloomSettings {
            intensity: INITIAL_BLOOM_INTENSITY,
            low_frequency_boost: INITIAL_BLOOM_LFB,
            ..default()
        });

        return;
    }

    let CameraState::EffectOnSpecial { fired, look_at } = &mut *camera_state
    else {
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

        cmd.entity(camera_entity).remove::<BloomSettings>();
        *camera_state = CameraState::Normal;

        camera_projection.scale = 1.0;
        // we don't update the translation, it will be done in follow_hoshi

        return;
    }

    let window_width_px = window.single().resolution.width();

    struct CameraUpdateArgs {
        intensity: f32,
        low_frequency_boost: f32,
        scale: f32,
        /// Used for lerp.
        /// Translates towards this point with some bias where
        translate_towards: Vec2,
        /// Used for lerp.
        /// 1.0 means translate towards it completely.
        translate_bias: f32,
        /// We clamp the translation to this from both sides.
        translate_freedom: Vec2,
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

        let freedom = Vec2::new(camera_x_clamp(window_width_px), f32::MAX);

        CameraUpdateArgs {
            intensity: new_intensity,
            low_frequency_boost: new_lfb,
            scale: new_scale,
            translate_towards: *look_at,
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

        let hoshi_position = hoshi.single().translation.truncate();

        if how_long_after_fired
            < FROM_ZOOMED_BACK_TO_NORMAL_WHEN_SPECIAL_IS_LOADED_IN
        {
            // zooming out

            let animation_elapsed = how_long_after_fired.as_secs_f32()
                / FROM_ZOOMED_BACK_TO_NORMAL_WHEN_SPECIAL_IS_LOADED_IN
                    .as_secs_f32();

            let new_scale =
                ZOOM_IN_SCALE + (1.0 - ZOOM_IN_SCALE) * animation_elapsed;

            let freedom = Vec2::new(camera_x_clamp(window_width_px), f32::MAX);

            CameraUpdateArgs {
                intensity: new_intensity,
                low_frequency_boost: new_lfb,
                scale: new_scale,
                translate_towards: hoshi_position,
                translate_bias: animation_elapsed,
                translate_freedom: freedom,
            }
        } else {
            // zoomed out

            CameraUpdateArgs {
                intensity: new_intensity,
                low_frequency_boost: new_lfb,
                scale: 1.0,
                translate_towards: hoshi_position,
                translate_bias: 1.0,
                translate_freedom: Vec2::new(
                    camera_x_clamp(window_width_px),
                    f32::MAX,
                ),
            }
        }
    };

    let mut settings = camera_bloom.expect("bloom settings");
    settings.intensity = intensity;
    settings.low_frequency_boost = low_frequency_boost;
    camera_projection.scale = scale;

    let camera_z = camera_transform.translation.z;
    camera_transform.translation = camera_transform
        .translation
        .truncate()
        .lerp(translate_towards, translate_bias)
        .clamp(-translate_freedom, translate_freedom)
        .extend(camera_z)
}

/// Takes into account the window size and clamps the camera to the screen.
fn clamp_camera_to_screen(
    window: &Window,
    hoshi_transform: &Transform,
    camera_transform: &mut Transform,
) {
    let z = camera_transform.translation.z;
    let y = hoshi_transform.translation.y;

    let window_width_px = window.resolution.width();
    let x_clamp = camera_x_clamp(window_width_px);
    // camera is bounded by the screen size
    // so we need to find the window width and figure out the bounds
    let x = hoshi_transform.translation.x.clamp(-x_clamp, x_clamp);
    camera_transform.translation = Vec3::new(x, y, z);
}

fn camera_x_clamp(window_width_px: f32) -> f32 {
    let half_window_width = window_width_px / 2.0 / PIXEL_ZOOM as f32;
    if half_window_width > HALF_LEVEL_WIDTH_PX {
        // The screen is wider than the level, camera cannot move.
        // In fact, we might prefer decreasing the scale to fit the level.
        0.0
    } else {
        HALF_LEVEL_WIDTH_PX - half_window_width
    }
}
