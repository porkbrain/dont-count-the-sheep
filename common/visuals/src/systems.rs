//! You ought to register these systems by yourself.

use bevy::prelude::*;
use common_ext::ColorExt;

use crate::{
    AtlasAnimation, AtlasAnimationEnd, AtlasAnimationStep, AtlasAnimationTimer,
    BeginAtlasAnimation, BeginAtlasAnimationCond, BeginInterpolationEvent,
    ColorInterpolation, Flicker, OnInterpolationFinished,
    TranslationInterpolation, UiStyleHeightInterpolation,
};

/// Advances the animation by one frame.
/// This requires that the [`AtlasAnimationTimer`] component is present along
/// with [`TextureAtlas`] and [`AtlasAnimation`].
pub fn advance_atlas_animation(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<(
        Entity,
        &AtlasAnimation,
        &mut AtlasAnimationTimer,
        &mut TextureAtlas,
        &mut Visibility,
    )>,
) {
    for (entity, animation, mut timer, mut atlas, mut visibility) in &mut query
    {
        timer.inner.tick(time.delta());
        if !timer.inner.just_finished() {
            continue;
        }

        match animation.next_step_index_and_frame(&atlas, timer.current_step) {
            Some((step_index, frame)) => {
                atlas.index = frame;
                timer.current_step = step_index;
            }
            None => {
                match &animation.on_last_frame {
                    AtlasAnimationEnd::RemoveTimerAndHideAndReset => {
                        cmd.entity(entity).remove::<AtlasAnimationTimer>();
                        *visibility = Visibility::Hidden;
                        atlas.index = match animation.play {
                            AtlasAnimationStep::Forward => animation.first,
                            AtlasAnimationStep::Backward => animation.last,
                        };
                    }
                    AtlasAnimationEnd::DespawnRecursiveItself => {
                        cmd.entity(entity).despawn_recursive();
                    }
                    AtlasAnimationEnd::RemoveTimer => {
                        cmd.entity(entity).remove::<AtlasAnimationTimer>();
                    }
                    AtlasAnimationEnd::Custom { with: Some(fun) } => {
                        fun(&mut cmd, entity, &mut atlas, &mut visibility);
                    }
                    AtlasAnimationEnd::Custom { with: None } => {
                        // nothing happens
                    }
                    AtlasAnimationEnd::LoopIndefinitely => {
                        atlas.index = match animation.play {
                            AtlasAnimationStep::Forward => animation.first,
                            AtlasAnimationStep::Backward => animation.last,
                        };
                    }
                }
            }
        }
    }
}

/// With given chance per second, start animation by inserting an
/// [`AtlasAnimationTimer`] component to the entity.
pub fn begin_atlas_animation_at_random(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<
        (Entity, &mut BeginAtlasAnimation, &mut Visibility),
        Without<AtlasAnimationTimer>,
    >,
) {
    for (entity, mut settings, mut visibility) in &mut query {
        if let Some((min_life, ref mut stopwatch)) =
            settings.with_min_delay.as_mut()
        {
            stopwatch.tick(time.delta());
            if stopwatch.elapsed() < *min_life {
                continue;
            }
        }
        settings.with_min_delay = None;

        let should_start = match &settings.cond {
            BeginAtlasAnimationCond::Immediately
            | BeginAtlasAnimationCond::Custom { with: None } => true,
            BeginAtlasAnimationCond::AtRandom(chance_per_second) => {
                rand::random::<f32>() < chance_per_second * time.delta_seconds()
            }
            BeginAtlasAnimationCond::Custom { with: Some(fun) } => {
                let frame_time = settings.frame_time;
                let fun = *fun;
                cmd.add(move |w: &mut World| {
                    if !fun(w, entity) {
                        return;
                    }

                    if let Some(mut entity) = w.get_entity_mut(entity) {
                        entity.remove::<BeginAtlasAnimation>();
                        entity.insert(AtlasAnimationTimer::new(
                            frame_time,
                            TimerMode::Repeating,
                        ));
                    }
                });

                // the condition has to be evaluated later bcs we don't have
                // access to the world here
                false
            }
        };

        if should_start {
            *visibility = Visibility::Visible;
            cmd.entity(entity).insert(AtlasAnimationTimer::new(
                settings.frame_time,
                TimerMode::Repeating,
            ));
        }
    }
}

/// Flickers the entity with the given chance per second.
/// The entity will be visible for the given duration if the chance hits.
/// For the rest of the time, the entity will be hidden.
pub fn flicker(
    time: Res<Time>,

    mut query: Query<(&mut Flicker, &mut Visibility)>,
) {
    for (mut flicker, mut visibility) in &mut query {
        if matches!(*visibility, Visibility::Hidden) {
            if flicker.last.elapsed() > flicker.shown_for {
                *visibility = Visibility::Visible;
            }
        } else if rand::random::<f32>()
            < flicker.chance_per_second * time.delta_seconds()
        {
            flicker.reset();
            *visibility = Visibility::Hidden;
        }
    }
}

/// Receives events to start interpolations.
///
/// This is always run last, so that no `Update` schedule system must explicitly
/// set order.
/// Also, this means that removal of the component will be processed before
/// insertion of the same component.
///
/// This system always runs if there are any events to process, it's registered
/// by the plugin.
pub(crate) fn recv_begin_interpolation_events(
    mut cmd: Commands,
    mut events: EventReader<BeginInterpolationEvent>,
) {
    for event in events.read().cloned() {
        event.insert(&mut cmd);
    }
}

/// Runs interpolation logic on the entities that have the relevant components.
/// Must run before `Last` schedule, or at least before the
/// `recv_begin_interpolation_events`.
pub fn interpolate(
    mut cmd: Commands,
    time: Res<Time>,

    // color interpolation
    mut sprites: Query<
        (Entity, &mut Sprite, &mut ColorInterpolation),
        Without<Text>,
    >,
    mut text: Query<
        (Entity, &mut Text, &mut ColorInterpolation),
        Without<Sprite>,
    >,

    // translation interpolation
    mut translations: Query<(
        Entity,
        &mut Transform,
        &mut TranslationInterpolation,
    )>,

    // Ui style height interpolation
    mut styles: Query<(Entity, &mut Style, &mut UiStyleHeightInterpolation)>,
) {
    let dt = time.delta();

    // color interpolation

    let mut color_interpolation =
        |entity, color: &mut Color, interpolation: &mut ColorInterpolation| {
            interpolation.started_at.tick(dt);

            let elapsed_fraction = interpolation.started_at.elapsed_secs()
                / interpolation.over.as_secs_f32();

            if elapsed_fraction >= 1.0 {
                *color = interpolation.to;
                cmd.entity(entity).remove::<ColorInterpolation>();

                match &interpolation.when_finished {
                    Some(OnInterpolationFinished::Custom(fun)) => {
                        fun(&mut cmd);
                    }
                    Some(OnInterpolationFinished::DespawnRecursiveItself) => {
                        cmd.entity(entity).despawn_recursive();
                    }
                    None => {}
                }
            } else {
                let lerp_factor = interpolation
                    .animation_curve
                    .as_ref()
                    .map(|curve| curve.ease(elapsed_fraction))
                    .unwrap_or(elapsed_fraction);

                let from = interpolation.from.get_or_insert(*color);
                *color = from.lerp(interpolation.to, lerp_factor);
            }
        };

    for (entity, mut sprite, mut interpolation) in sprites.iter_mut() {
        color_interpolation(entity, &mut sprite.color, &mut interpolation);
    }
    for (entity, mut text, mut interpolation) in text.iter_mut() {
        // Color interpolation is currently supported only for one text section
        debug_assert_eq!(1, text.sections.len());
        if let Some(section) = &mut text.sections.first_mut() {
            color_interpolation(
                entity,
                &mut section.style.color,
                &mut interpolation,
            );
        }
    }

    // translation interpolation

    for (entity, mut transform, mut interpolation) in translations.iter_mut() {
        interpolation.started_at.tick(dt);

        let elapsed_fraction = interpolation.started_at.elapsed_secs()
            / interpolation.over.as_secs_f32();

        if elapsed_fraction >= 1.0 {
            transform.translation =
                interpolation.to.extend(transform.translation.z);
            cmd.entity(entity).remove::<TranslationInterpolation>();

            match &interpolation.when_finished {
                Some(OnInterpolationFinished::Custom(fun)) => {
                    fun(&mut cmd);
                }
                Some(OnInterpolationFinished::DespawnRecursiveItself) => {
                    cmd.entity(entity).despawn_recursive();
                }
                None => {}
            }
        } else {
            let lerp_factor = interpolation
                .animation_curve
                .as_ref()
                .map(|curve| curve.ease(elapsed_fraction))
                .unwrap_or(elapsed_fraction);

            let from = interpolation
                .from
                .get_or_insert(transform.translation.truncate());
            transform.translation = from
                .lerp(interpolation.to, lerp_factor)
                .extend(transform.translation.z);
        }
    }

    // Ui style height interpolation

    for (entity, mut style, mut interpolation) in styles.iter_mut() {
        interpolation.started_at.tick(dt);

        let elapsed_fraction = interpolation.started_at.elapsed_secs()
            / interpolation.over.as_secs_f32();

        if elapsed_fraction >= 1.0 {
            style.height = interpolation.to;
            cmd.entity(entity).remove::<UiStyleHeightInterpolation>();

            match &interpolation.when_finished {
                Some(OnInterpolationFinished::Custom(fun)) => {
                    fun(&mut cmd);
                }
                Some(OnInterpolationFinished::DespawnRecursiveItself) => {
                    cmd.entity(entity).despawn_recursive();
                }
                None => {}
            }
        } else {
            let lerp_factor = interpolation
                .animation_curve
                .as_ref()
                .map(|curve| curve.ease(elapsed_fraction))
                .unwrap_or(elapsed_fraction);

            let to = interpolation.to;
            let from = interpolation.from.get_or_insert(style.height);
            style.height = match (from, to) {
                (Val::Percent(from), Val::Percent(to)) => {
                    Val::Percent(from.lerp(to, lerp_factor))
                }
                (Val::Px(from), Val::Px(to)) => {
                    Val::Px(from.lerp(to, lerp_factor))
                }
                (Val::Vw(from), Val::Vw(to)) => {
                    Val::Vw(from.lerp(to, lerp_factor))
                }
                (Val::Vh(from), Val::Vh(to)) => {
                    Val::Vh(from.lerp(to, lerp_factor))
                }
                (Val::VMin(from), Val::VMin(to)) => {
                    Val::VMin(from.lerp(to, lerp_factor))
                }
                (Val::VMax(from), Val::VMax(to)) => {
                    Val::VMax(from.lerp(to, lerp_factor))
                }
                _ => {
                    debug_assert!(
                        false,
                        "Cannot interpolate between different units"
                    );
                    error!("Cannot interpolate between different units");
                    continue;
                }
            };
        }
    }
}
