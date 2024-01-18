use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::Instant};

/// Describes how to drive an animation.
/// The animation specifically integrates with texture atlas sprites.
#[derive(Component)]
pub struct Animation {
    /// What should happen when the last frame is reached?
    pub on_last_frame: AnimationEnd,
    /// The index of the first frame.
    /// Typically 0.
    pub first: usize,
    /// The index of the last frame of the atlas.
    pub last: usize,
}

/// Different strategies for when the last frame of an animation is reached.
pub enum AnimationEnd {
    /// Loops the animation.
    Loop,
    /// Removes the animation timer, hides the entity and sets the index back
    /// to the first frame.
    RemoveTimerAndHide,
    /// Just removes the animation timer.
    /// Keeps the entity visible and on the last frame.
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

/// Must be present for the systems to actually drive the animation.
#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub(crate) Timer);

/// Allows to start an animation at random.
#[derive(Component, Default)]
pub struct BeginAnimationAtRandom {
    /// We roll a dice every delta seconds.
    /// This scales that delta.
    pub chance_per_second: f32,
    /// Once the animation is started, how long should each frame be shown?
    pub frame_time: Duration,
    /// If present, the animation cannot be started before this time has
    /// passed.
    pub with_min_delay: Option<(Duration, Stopwatch)>,
}

/// Shows entity at random for a given duration.
/// Then hides it again.
#[derive(Component)]
pub struct Flicker {
    /// When did the flicker ran last?
    pub last: Instant,
    /// How likely is it to flicker every second?
    pub chance_per_second: f32,
    /// How long should the entity be shown before it's hidden again?
    pub shown_for: Duration,
}

impl Flicker {
    /// Creates a new flicker.
    #[inline]
    pub fn new(chance_per_second: f32, shown_for: Duration) -> Self {
        Self {
            last: Instant::now(),
            chance_per_second,
            shown_for,
        }
    }

    #[inline]
    pub(crate) fn reset(&mut self) {
        self.last = Instant::now();
    }
}

impl AnimationTimer {
    /// Creates a new animation timer.
    #[inline]
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self(Timer::new(duration, mode))
    }
}
