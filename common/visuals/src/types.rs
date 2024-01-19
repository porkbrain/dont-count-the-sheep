use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::Instant};

use crate::EASE_IN_OUT;

/// Describes how to drive an animation.
/// The animation specifically integrates with texture atlas sprites.
#[derive(Component, Default)]
pub struct AtlasAnimation {
    /// What should happen when the last frame is reached?
    pub on_last_frame: AtlasAnimationEnd,
    /// The index of the first frame.
    /// Typically 0.
    pub first: usize,
    /// The index of the last frame of the atlas.
    pub last: usize,
    /// If the animation should be played in reverse, going from last to first.
    /// When the first frame is reached, the animation still acts in accordance
    /// with the [`AtlasAnimationEnd`] strategy.
    pub reversed: bool,
}

/// Different strategies for when the last frame of an animation is reached.
#[derive(Default)]
pub enum AtlasAnimationEnd {
    /// Loops the animation.
    #[default]
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
                    &AtlasAnimation,
                    &mut AtlasAnimationTimer,
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
pub struct AtlasAnimationTimer(pub(crate) Timer);

/// Allows to start an animation at random.
#[derive(Component, Default)]
pub struct BeginAtlasAnimationAtRandom {
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

/// Smoothly translates an entity from one position to another.
#[derive(Component, Default)]
pub struct SmoothTranslation {
    /// What happens when we're done?
    pub on_finished: SmoothTranslationEnd,
    /// Where did the object start?
    pub from: Vec2,
    /// Where should the object end up?
    pub target: Vec2,
    /// How long should the translation take?
    pub duration: Duration,
    /// How long has the translation been running?
    pub stopwatch: Stopwatch,
    /// Not not provided then the translation will be linear.
    pub animation_curve: Option<CubicSegment<Vec2>>,
}

/// What should happen when the translation is done?
#[derive(Default)]
pub enum SmoothTranslationEnd {
    #[default]
    /// Just remove the component.
    RemoveSmoothTranslationComponent,
    /// Can schedule commands.
    Custom(Box<dyn Fn(&mut Commands) + Send + Sync>),
}

impl SmoothTranslation {
    /// Sets the animation curve to be the ubiquitous "ease-in-out".
    pub fn with_ease_in_out(mut self) -> Self {
        self.animation_curve = Some(EASE_IN_OUT.clone());

        self
    }
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

impl AtlasAnimationTimer {
    /// Creates a new animation timer.
    #[inline]
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self(Timer::new(duration, mode))
    }
}

impl AtlasAnimation {
    /// Takes into account whether the animation is reversed or not.
    pub fn is_on_last_frame(&self, sprite: &TextureAtlasSprite) -> bool {
        if self.reversed {
            self.first == sprite.index
        } else {
            self.last == sprite.index
        }
    }
}
