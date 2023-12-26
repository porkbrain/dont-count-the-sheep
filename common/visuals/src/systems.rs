use bevy::prelude::*;

use crate::{
    Animation, AnimationEnd, AnimationTimer, BeginAnimationAtRandom, Flicker,
};

pub fn advance_animation(
    mut query: Query<(
        Entity,
        &Animation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Visibility,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, animation, mut timer, mut atlas, mut visibility) in &mut query
    {
        timer.tick(time.delta());
        if timer.just_finished() {
            if atlas.index == animation.last {
                match &animation.on_last_frame {
                    AnimationEnd::RemoveTimer => {
                        commands.entity(entity).remove::<AnimationTimer>();
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
                            &mut commands,
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
    mut query: Query<
        (Entity, &BeginAnimationAtRandom, &mut Visibility),
        Without<AnimationTimer>,
    >,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, settings, mut visibility) in &mut query {
        if rand::random::<f32>()
            < settings.chance_per_second * time.delta_seconds()
        {
            *visibility = Visibility::Visible;
            commands.entity(entity).insert(AnimationTimer::new(
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
