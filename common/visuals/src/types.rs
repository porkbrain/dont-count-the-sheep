use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::Instant};

#[derive(Component)]
pub struct Animation {
    pub on_last_frame: AnimationEnd,
    pub first: usize,
    pub last: usize,
}

pub enum AnimationEnd {
    /// Loops the animation.
    Loop,
    /// Removes the animation timer.
    RemoveTimer,
    /// Can mutate state.
    #[allow(clippy::type_complexity)]
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

#[derive(Component, Default)]
pub struct BeginAnimationAtRandom {
    pub chance_per_second: f32,
    pub frame_time: Duration,
    pub with_min_life: Option<(Duration, Stopwatch)>,
}

/// Shows entity at random for a given duration.
/// Then hides it again.
#[derive(Component)]
pub struct Flicker {
    /// When did the flicker ran last?
    pub last: Instant,
    pub chance_per_second: f32,
    pub shown_for: Duration,
}

impl Flicker {
    #[inline]
    pub fn new(chance_per_second: f32, shown_for: Duration) -> Self {
        Self {
            last: Instant::now(),
            chance_per_second,
            shown_for,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.last = Instant::now();
    }
}

impl AnimationTimer {
    #[inline]
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self(Timer::new(duration, mode))
    }
}
