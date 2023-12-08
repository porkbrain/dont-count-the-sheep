use std::time::Duration;

use bevy::{prelude::*, utils::Instant};

#[derive(Component)]
pub struct Animation {
    pub on_last_frame: AnimationEnd,
    pub first: usize,
    pub last: usize,
}

pub enum AnimationEnd {
    /// Removes the animation timer.
    RemoveTimer,
    /// Can mutate state.
    Custom(
        Box<
            dyn Fn(
                    Entity,
                    &Animation,
                    &mut AnimationTimer,
                    &mut TextureAtlasSprite,
                    &mut Visibility,
                    &mut Commands,
                    &Time,
                ) + Send
                + Sync,
        >,
    ),
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub(crate) Timer);

#[derive(Component)]
pub struct ChangeFrameAtRandom {
    pub(crate) last_change: Instant,
    pub(crate) chance_per_second: f32,
    pub(crate) frames: &'static [usize],
}

impl AnimationTimer {
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self(Timer::new(duration, mode))
    }
}
