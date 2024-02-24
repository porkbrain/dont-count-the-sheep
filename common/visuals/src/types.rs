use std::{sync::Arc, time::Duration};

use bevy::{
    ecs::system::EntityCommands, prelude::*, time::Stopwatch, utils::Instant,
};

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
                    &mut TextureAtlas,
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

/// We use events instead of inserting the component directly because there
/// might be races such as one interpolation finishing which removes the
/// relevant component and another starting which inserts the same component.
/// If the insertion is applied before the removal, the removal will actually
/// remove the component that was just inserted.
///
/// Event system helps us serialize the order of the ops.
#[derive(Event, Clone)]
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
    /// Any extra logic when interpolation is done?
    pub(crate) when_finished: Option<OnInterpolationFinished>,
}

/// What should happen when the interpolation is done?
#[derive(Clone)]
pub(crate) enum OnInterpolationFinished {
    /// Can schedule commands.
    Custom(Arc<dyn Fn(&mut Commands) + Send + Sync>),
}

impl BeginInterpolationEvent {
    /// There are two ways of starting interpolations.
    /// One is to emit this struct as an event with [`EventWriter`].
    /// The advantage is deterministic order of operations.
    ///
    /// Use this method if you're not worried about the component potentially
    /// being removed in a scenario such as:
    ///
    /// ```ignore
    /// use bevy::prelude::*;
    /// fn main() {
    ///     App::new()
    ///         .add_systems(Startup, setup)
    ///         .add_systems(Update, (
    ///             schedule_insert_foo,
    ///             schedule_remove_foo,
    ///         ).chain())
    ///         .run()
    /// }
    ///
    /// #[derive(Component)]
    /// struct Bar;
    ///
    /// #[derive(Component)]
    /// struct Foo;
    ///
    /// fn setup(mut cmd: Commands) {
    ///     cmd.spawn(Bar);
    /// }
    ///
    /// fn schedule_insert_foo(mut cmd: Commands, q: Query<Entity, With<Bar>>) {
    ///     let bar = q.single();
    ///     cmd.entity(bar).insert(Foo);
    /// }
    ///
    /// fn schedule_remove_foo(mut cmd: Commands, q: Query<Entity, With<Bar>>) {
    ///     let bar = q.single();
    ///     cmd.entity(bar).remove::<Foo>();
    /// }
    /// ```
    pub fn insert_to(self, entity_cmd: &mut EntityCommands) {
        let Self {
            of,
            over,
            animation_curve,
            when_finished,
            entity: _,
        } = self;

        match of {
            InterpolationOf::Color { from, to } => {
                entity_cmd.insert(ColorInterpolation {
                    from,
                    to,
                    over,
                    animation_curve,
                    when_finished,
                    started_at: Default::default(),
                })
            }
            InterpolationOf::Translation { from, to } => {
                entity_cmd.insert(TranslationInterpolation {
                    from,
                    to,
                    over,
                    animation_curve,
                    when_finished,
                    started_at: Default::default(),
                })
            }
            InterpolationOf::UiStyleHeight { from, to } => {
                entity_cmd.insert(UiStyleHeightInterpolation {
                    from,
                    to,
                    over,
                    animation_curve,
                    when_finished,
                    started_at: Default::default(),
                })
            }
        };
    }

    /// Equivalent to [`Self::insert_to`] but just takes the [`EntityCommands`]
    /// directly.
    pub fn insert(self, cmd: &mut Commands) {
        let mut entity_cmd = cmd.entity(self.entity);
        self.insert_to(&mut entity_cmd);
    }

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
            when_finished: None,
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
            when_finished: None,
        }
    }

    /// Interpolates the height of a UI element.
    /// If variants of [`Val`] don't match, the interpolation will fail.
    pub fn of_ui_style_height(
        entity: Entity,
        from: Option<Val>,
        to: Val,
    ) -> Self {
        if let Some(from) = from {
            // must be same units
            debug_assert!(matches!(
                (from, to),
                (Val::Auto, Val::Auto)
                    | (Val::Px(_), Val::Px(_))
                    | (Val::Percent(_), Val::Percent(_))
                    | (Val::Vw(_), Val::Vw(_))
                    | (Val::Vh(_), Val::Vh(_))
                    | (Val::VMin(_), Val::VMin(_))
                    | (Val::VMax(_), Val::VMax(_))
            ));
        }

        Self {
            entity,
            over: Duration::from_secs(1),
            of: InterpolationOf::UiStyleHeight { from, to },
            animation_curve: None,
            when_finished: None,
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

    /// Like [`Self::with_animation_curve`] but works with APIs that use option.
    pub fn with_animation_opt_curve(
        mut self,
        curve: Option<CubicSegment<Vec2>>,
    ) -> Self {
        self.animation_curve = curve;
        self
    }

    /// Sets the animation curve to be the ubiquitous "ease-in-out".
    pub fn with_ease_in_out(self) -> Self {
        self.with_animation_curve(EASE_IN_OUT.clone())
    }

    /// Any commands to schedule when interpolation is done?
    pub fn when_finished_do(
        self,
        when_finished: impl Fn(&mut Commands) + Send + Sync + 'static,
    ) -> Self {
        self.when_finished(OnInterpolationFinished::Custom(Arc::new(
            when_finished,
        )))
    }

    /// Any extra logic when interpolation is done?
    pub(crate) fn when_finished(
        mut self,
        when_finished: OnInterpolationFinished,
    ) -> Self {
        self.when_finished = Some(when_finished);
        self
    }
}

/// What should be interpolated?
#[derive(Clone)]
pub enum InterpolationOf {
    /// Interpolate the color of [`TextureAtlas`] and [`Sprite`].
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
    /// Interpolate the height of a UI element.
    ///
    /// If variants of [`Val`] don't match, the interpolation will fail.
    UiStyleHeight {
        /// The initial height set to this.
        /// It must match the variant of [`Val`] used with `to`.
        from: Option<Val>,
        /// The height to interpolate to.
        to: Val,
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
    #[reflect(ignore)]
    pub(crate) when_finished: Option<OnInterpolationFinished>,
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
    #[reflect(ignore)]
    pub(crate) when_finished: Option<OnInterpolationFinished>,
}

/// Interpolates the height of a UI element.
#[derive(Component, Reflect)]
pub struct UiStyleHeightInterpolation {
    pub(crate) from: Option<Val>,
    pub(crate) to: Val,
    pub(crate) started_at: Stopwatch,
    pub(crate) over: Duration,
    #[reflect(ignore)]
    pub(crate) animation_curve: Option<CubicSegment<Vec2>>,
    #[reflect(ignore)]
    pub(crate) when_finished: Option<OnInterpolationFinished>,
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
    pub fn is_on_last_frame(&self, sprite: &TextureAtlas) -> bool {
        if self.reversed {
            self.first == sprite.index
        } else {
            self.last == sprite.index
        }
    }
}
