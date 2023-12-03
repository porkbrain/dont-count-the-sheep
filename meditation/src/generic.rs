#![allow(dead_code)]

use bevy::{prelude::*, utils::Instant};
use rand::seq::SliceRandom;

#[derive(Default, Deref, DerefMut, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Radians(f32);

#[derive(
    Component, Default, Deref, DerefMut, Clone, Copy, PartialEq, Debug,
)]
pub(crate) struct Velocity(Vec2);

#[derive(
    Component, Default, Deref, DerefMut, Clone, Copy, PartialEq, Debug,
)]
/// Positive should be counter-clockwise.
pub(crate) struct AngularVelocity(pub f32);

#[derive(Component)]
pub(crate) struct Animation {
    pub(crate) on_last_frame: AnimationEnd,
    pub(crate) first: usize,
    pub(crate) last: usize,
}

pub(crate) enum AnimationEnd {
    /// Removes the animation timer.
    RemoveTimer,
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AnimationTimer(pub(crate) Timer);

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum MotionDirection {
    #[allow(dead_code)]
    None,
    Left,
    Right,
}

impl MotionDirection {
    #[inline]
    pub(crate) fn sign(&self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
            Self::None => 0.0,
        }
    }

    #[inline]
    pub(crate) fn is_aligned(&self, point: f32) -> bool {
        match self {
            Self::Left => point < 0.0,
            Self::Right => point > 0.0,
            Self::None => point == 0.0,
        }
    }
}

impl Radians {
    pub(crate) fn new(radians: f32) -> Self {
        Self(radians)
    }
}

impl Velocity {
    pub(crate) fn new(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<Vec2> for Velocity {
    fn from(vec: Vec2) -> Self {
        Self(vec)
    }
}

impl From<f32> for AngularVelocity {
    fn from(radians: f32) -> Self {
        Self(radians)
    }
}

impl AngularVelocity {
    pub(crate) fn new(radians: f32) -> Self {
        Self(radians)
    }
}

#[derive(Component)]
pub(crate) struct ChangeFrameAtRandom {
    last_change: Instant,
    chance_per_second: f32,
    frames: &'static [usize],
}

pub(crate) fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut transform, vel) in &mut query {
        // TODO
        transform.translation.x += vel.x * dt / 2.0;
        transform.translation.y += vel.y * dt / 2.0;
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
                }
            } else {
                atlas.index += 1
            };
        }
    }
}
