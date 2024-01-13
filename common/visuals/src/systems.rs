//! You ought to register these systems by yourself.

use bevy::prelude::*;

use crate::{
    Animation, AnimationEnd, AnimationTimer, BeginAnimationAtRandom, Flicker,
};

/// Advances the animation by one frame.
/// This requires that the [`AnimationTimer`] component is present along with
/// [`TextureAtlasSprite`] and [`Animation`].
pub fn advance_animation(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<(
        Entity,
        &Animation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Visibility,
    )>,
) {
    for (entity, animation, mut timer, mut atlas, mut visibility) in &mut query
    {
        timer.tick(time.delta());
        if timer.just_finished() {
            if atlas.index == animation.last {
                match &animation.on_last_frame {
                    AnimationEnd::RemoveTimer => {
                        cmd.entity(entity).remove::<AnimationTimer>();
                        *visibility = Visibility::Hidden;
                        atlas.index = animation.first
                    }
                    AnimationEnd::Custom(fun) => {
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
                    AnimationEnd::Loop => {
                        atlas.index = animation.first;
                    }
                }
            } else {
                atlas.index += 1
            };
        }
    }
}

/// With given chance per second, start animation by inserting an
/// [`AnimationTimer`] component to the entity.
pub fn begin_animation_at_random(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<
        (Entity, &mut BeginAnimationAtRandom, &mut Visibility),
        Without<AnimationTimer>,
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
            cmd.entity(entity).insert(AnimationTimer::new(
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
    mut query: Query<(&mut Flicker, &mut Visibility)>,
    time: Res<Time>,
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
