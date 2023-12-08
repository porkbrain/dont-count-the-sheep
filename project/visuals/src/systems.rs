use bevy::{prelude::*, utils::Instant};
use rand::seq::SliceRandom;

use crate::{Animation, AnimationEnd, AnimationTimer, ChangeFrameAtRandom};

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

pub(crate) fn change_frame_at_random(
    mut query: Query<(&mut ChangeFrameAtRandom, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    for (mut change, mut atlas) in &mut query {
        if rand::random::<f32>()
            < change.chance_per_second * time.delta_seconds()
        {
            change.last_change = Instant::now();
            atlas.index = *change
                .frames
                .choose(&mut rand::thread_rng())
                .expect("Frames must not be empty");
        }
    }
}
