//! You ought to register these systems by yourself.

use bevy::prelude::*;

use crate::{
    AtlasAnimation, AtlasAnimationEnd, AtlasAnimationTimer,
    BeginAtlasAnimationAtRandom, Flicker, SmoothTranslation,
    SmoothTranslationEnd,
};

/// Advances the animation by one frame.
/// This requires that the [`AnimationTimer`] component is present along with
/// [`TextureAtlasSprite`] and [`Animation`].
pub fn advance_atlas_animation(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<(
        Entity,
        &AtlasAnimation,
        &mut AtlasAnimationTimer,
        &mut TextureAtlasSprite,
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
            } else {
                if animation.reversed {
                    atlas.index -= 1;
                } else {
                    atlas.index += 1;
                }
            };
        }
    }
}

/// With given chance per second, start animation by inserting an
/// [`AnimationTimer`] component to the entity.
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

/// Smoothly translates entities from one position to another.
pub fn smoothly_translate(
    mut cmd: Commands,
    time: Res<Time>,

    mut entities: Query<(Entity, &mut Transform, &mut SmoothTranslation)>,
) {
    let dt = time.delta();
    for (entity, mut transform, mut animation) in entities.iter_mut() {
        animation.stopwatch.tick(dt);

        let keep_z = transform.translation.z;

        if animation.stopwatch.elapsed() >= animation.duration {
            transform.translation = animation.target.extend(keep_z);

            match &animation.on_finished {
                SmoothTranslationEnd::RemoveSmoothTranslationComponent => {
                    cmd.entity(entity).remove::<SmoothTranslation>();
                }
                SmoothTranslationEnd::Custom(fun) => fun(&mut cmd),
            }
            continue;
        }

        let lerp_factor = animation.stopwatch.elapsed_secs()
            / animation.duration.as_secs_f32();

        transform.translation = animation
            .animation_curve
            .as_ref()
            .map(|curve| {
                animation
                    .from
                    .lerp(animation.target, curve.ease(lerp_factor))
            })
            .unwrap_or_else(|| {
                animation.from.lerp(animation.target, lerp_factor)
            })
            .extend(keep_z);
    }
}
