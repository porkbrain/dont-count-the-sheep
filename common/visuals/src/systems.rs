//! You ought to register these systems by yourself.

use bevy::prelude::*;
use common_ext::ColorExt;

use crate::{
    AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer,
    BeginAtlasAnimationAtRandom, BeginInterpolationEvent, ColorInterpolation,
    Flicker, InterpolationOf, OnInterpolationFinished,
    TranslationInterpolation,
};

/// Advances the animation by one frame.
/// This requires that the [`AtlasAnimationTimer`] component is present along
/// with [`TextureAtlasSprite`] and [`AtlasAnimation`].
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
        timer.tick(time.delta());
        if timer.just_finished() {
            if animation.is_on_last_frame(&atlas) {
                match &animation.on_last_frame {
                    AtlasAnimationEnd::RemoveTimerAndHide => {
                        cmd.entity(entity).remove::<AtlasAnimationTimer>();
                        *visibility = Visibility::Hidden;
                        atlas.index = if animation.reversed {
                            animation.last
                        } else {
                            animation.first
                        };
                    }
                    AtlasAnimationEnd::RemoveTimer => {
                        cmd.entity(entity).remove::<AtlasAnimationTimer>();
                    }
                    AtlasAnimationEnd::Custom(fun) => {
                        fun(
                            entity,
                            animation,
                            &mut timer,
                            &mut atlas,
                            &mut visibility,
                            &mut cmd,
                            &time,
                        );
                    }
                    AtlasAnimationEnd::Loop => {
                        atlas.index = if animation.reversed {
                            animation.last
                        } else {
                            animation.first
                        };
                    }
                }
            } else if animation.reversed {
                atlas.index -= 1;
            } else {
                atlas.index += 1;
            };
        }
    }
}

/// With given chance per second, start animation by inserting an
/// [`AtlasAnimationTimer`] component to the entity.
pub fn begin_atlas_animation_at_random(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<
        (Entity, &mut BeginAtlasAnimationAtRandom, &mut Visibility),
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

        if rand::random::<f32>()
            < settings.chance_per_second * time.delta_seconds()
        {
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
    for BeginInterpolationEvent {
        entity,
        of,
        over,
        animation_curve: curve,
        when_finished,
    } in events.read()
    {
        let mut entity_cmd = cmd.entity(*entity);
        match of {
            InterpolationOf::Color { from, to } => {
                entity_cmd.insert(ColorInterpolation {
                    from: *from,
                    to: *to,
                    over: *over,
                    started_at: Default::default(),
                    animation_curve: curve.clone(),
                    when_finished: when_finished.clone(),
                })
            }
            InterpolationOf::Translation { from, to } => {
                entity_cmd.insert(TranslationInterpolation {
                    from: *from,
                    to: *to,
                    over: *over,
                    started_at: Default::default(),
                    animation_curve: curve.clone(),
                    when_finished: when_finished.clone(),
                })
            }
        };
    }
}

/// Runs interpolation logic on the entities that have the relevant components.
/// Must run before `Last` schedule, or at least before the
/// `recv_begin_interpolation_events`.
pub fn interpolate(
    mut cmd: Commands,
    time: Res<Time>,

    // color interpolation
    mut sprites: Query<(Entity, &mut Sprite, &mut ColorInterpolation)>,

    // translation interpolation
    mut translations: Query<(
        Entity,
        &mut Transform,
        &mut TranslationInterpolation,
    )>,
) {
    let dt = time.delta();

    // color interpolation

    for (entity, mut sprite, mut interpolation) in sprites.iter_mut() {
        interpolation.started_at.tick(dt);

        let elapsed_fraction = interpolation.started_at.elapsed_secs()
            / interpolation.over.as_secs_f32();

        if elapsed_fraction >= 1.0 {
            sprite.color = interpolation.to;
            cmd.entity(entity).remove::<ColorInterpolation>();

            match &interpolation.when_finished {
                Some(OnInterpolationFinished::Custom(fun)) => {
                    fun(&mut cmd);
                }
                None => {}
            }
        } else {
            let lerp_factor = interpolation
                .animation_curve
                .as_ref()
                .map(|curve| curve.ease(elapsed_fraction))
                .unwrap_or(elapsed_fraction);

            let from = interpolation.from.get_or_insert(sprite.color);
            sprite.color = from.lerp(interpolation.to, lerp_factor);
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
}
