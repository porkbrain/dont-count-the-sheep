use bevy::prelude::*;
use rand::seq::IteratorRandom;

use crate::{
    Animation, AnimationEnd, AnimationTimer, ChangeFrameAtRandom, Flicker,
};

pub(crate) fn advance_animation(
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
                }
            } else {
                atlas.index += 1
            };
        }
    }
}

/// Given a chance per second, changes the frame at random from a selection of
/// frames.
pub(crate) fn change_frame_at_random(
    mut query: Query<(&ChangeFrameAtRandom, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (change, mut atlas) in &mut query {
        if rand::random::<f32>()
            < change.chance_per_second * time.delta_seconds()
        {
            atlas.index = *change
                .frames
                .iter()
                .filter(|f| **f != atlas.index)
                .choose(&mut rand::thread_rng())
                .expect("Frames must not be empty");
        }
    }
}

/// Flickers the entity with the given chance per second.
/// The entity will be visible for the given duration if the chance hits.
/// For the rest of the time, the entity will be hidden.
pub(crate) fn flicker(
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
