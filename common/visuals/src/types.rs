use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::Instant};

use crate::EASE_IN_OUT;

/// Describes how to drive an animation.
/// The animation specifically integrates with texture atlas sprites.
#[derive(Component, Default, Reflect)]
pub struct AtlasAnimation {
    /// What should happen when the last frame is reached?
    #[reflect(ignore)]
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
#[derive(Default, Reflect)]
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
    #[reflect(ignore)]
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
#[derive(Component, Default, Reflect)]
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
#[derive(Component, Reflect)]
pub struct Flicker {
    /// When did the flicker ran last?
    pub last: Instant,
    /// How likely is it to flicker every second?
    pub chance_per_second: f32,
    /// How long should the entity be shown before it's hidden again?
    pub shown_for: Duration,
}

/// Smoothly translates an entity from one position to another.
///
/// TODO: merge to interpolation
#[derive(Component, Default, Reflect)]
pub struct SmoothTranslation {
    /// What happens when we're done?
    #[reflect(ignore)]
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
    #[reflect(ignore)]
    pub animation_curve: Option<CubicSegment<Vec2>>,
}

/// What should happen when the translation is done?
#[derive(Default, Reflect)]
pub enum SmoothTranslationEnd {
    #[default]
    /// Just remove the component.
    RemoveSmoothTranslationComponent,
    #[reflect(ignore)]
    /// Can schedule commands.
    Custom(Box<dyn Fn(&mut Commands) + Send + Sync>),
}

/// We use events instead of inserting the component directly because there
/// might be races such as one interpolation finishing which removes the
/// relevant component and another starting which inserts the same component.
/// If the insertion is applied before the removal, the removal will actually
/// remove the component that was just inserted.
///
/// Event system helps us serialize the order of the ops.
#[derive(Event)]
pub struct BeginInterpolationEvent {
    /// The interpolation target.
    /// See the enum for info about properties of which components are
    /// affected.
    pub of: InterpolationOf,
    /// How long should the interpolation take?
    /// Once the duration is over, the interpolation component is removed.
    ///
    /// Defaults to 1 second.
    pub(crate) over: Duration,
    /// The entity to interpolate.
    pub entity: Entity,
    /// Optionally select a cubic curve to follow instead of the default linear
    /// interpolation.
    pub animation_curve: Option<CubicSegment<Vec2>>,
}

impl BeginInterpolationEvent {
    /// Interpolates the color of a sprite.
    ///
    /// Defaults to 1 second and lerps from the latest color to the new color
    /// unless the initial color is provided.
    pub fn of_color(entity: Entity, from: Option<Color>, to: Color) -> Self {
        Self {
            entity,
            over: Duration::from_secs(1),
            of: InterpolationOf::Color { from, to },
            animation_curve: None,
        }
    }

    /// Interpolates translation of an entity.
    ///
    /// Defaults to 1 second and lerps from the latest position to the new
    /// position unless the initial position is provided.
    pub fn of_translation(
        entity: Entity,
        from: Option<Vec2>,
        to: Vec2,
    ) -> Self {
        Self {
            entity,
            over: Duration::from_secs(1),
            of: InterpolationOf::Translation { from, to },
            animation_curve: None,
        }
    }

    /// How long should the interpolation take?
    /// Defaults to 1 second.
    pub fn over(mut self, over: Duration) -> Self {
        debug_assert!(over.as_millis() > 0, "Duration mustn't be zero");
        self.over = over;
        self
    }

    /// Optionally select a cubic curve to follow instead of the default linear
    /// interpolation.
    pub fn with_animation_curve(mut self, curve: CubicSegment<Vec2>) -> Self {
        self.animation_curve = Some(curve);
        self
    }
}

/// What should be interpolated?
pub enum InterpolationOf {
    /// Interpolate the color of [`TextureAtlasSprite`] and [`Sprite`].
    Color {
        /// The color to interpolate from.
        /// If not provided, the latest color is used.
        from: Option<Color>,
        /// The color to interpolate to.
        to: Color,
    },
    /// Interpolate the position of an entity.
    Translation {
        /// Where does the object start?
        /// If not provided, the current position is used.
        from: Option<Vec2>,
        /// Where should the object end up?
        to: Vec2,
    },
}

/// Interpolates the color of a sprite.
#[derive(Component, Reflect)]
pub struct ColorInterpolation {
    /// Can be none on the first run, then we default it to the color of the
    /// sprite.
    pub(crate) from: Option<Color>,
    pub(crate) to: Color,
    pub(crate) started_at: Stopwatch,
    pub(crate) over: Duration,
    #[reflect(ignore)]
    pub(crate) animation_curve: Option<CubicSegment<Vec2>>,
}

/// Interpolates translation of an entity.
#[derive(Component, Reflect)]
pub struct TranslationInterpolation {
    /// Can be none on the first run, then we default it to the position of the
    /// entity.
    pub(crate) from: Option<Vec2>,
    pub(crate) to: Vec2,
    pub(crate) started_at: Stopwatch,
    pub(crate) over: Duration,
    #[reflect(ignore)]
    pub(crate) animation_curve: Option<CubicSegment<Vec2>>,
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
