//! Contains state management for traits and player stats.
//!
//! See the wiki for more information about how the traits exactly work etc,
//! because lots of logic related to them is spread across the codebase.

use crate::{hud::daybar::Beats, prelude::*};

/// The main resource for the player's stats.
#[derive(Resource, Default)]
#[cfg_attr(feature = "devtools", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "devtools", reflect(Resource, InspectorOptions))]
pub struct PlayerStats {
    /// Starts at o (day 1) and increases by 1 every time the player goes to
    /// sleep.
    pub days_passed: usize,
    /// Spiritual points are rewarded for doing certain actions.
    pub spiritual_points: usize,
    /// Material points are rewarded for doing certain actions.
    pub material_points: usize,
    /// The player's character is improved with traits.
    pub traits: Traits,
    /// If positive, how many days in a row did the player deplete their beats?
    /// If negative, how many days in a row didn't they deplete their beats?
    pub days_depleting_beats_streak: isize,
}

/// List of traits that the player has or can have.
#[derive(Default)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
pub struct Traits {
    /// See [`NightOwl`].
    pub night_owl: NightOwl,
    /// See [`EarlyBird`].
    pub early_bird: EarlyBird,
}

/// Relates to timekeeping.
///
/// Mutually exclusive with [`EarlyBird`] most of the time.
#[cfg_attr(feature = "devtools", derive(Reflect))]
pub struct NightOwl {
    /// The player saved this many beats on their actions thanks to this trait.
    pub extra_beats_today: Beats,
    /// Whether the player has this trait.
    pub is_active: bool,
    /// How much percentage discount the player gets on their actions
    /// (`<0; 1>`).
    ///
    /// The actual discount will be 0 in the beginning of the day and converge
    /// to this value at the end of the day.
    pub full_discount: f32,
}

/// Relates to timekeeping.
///
/// Mutually exclusive with [`NightOwl`] most of the time.
#[cfg_attr(feature = "devtools", derive(Reflect))]
pub struct EarlyBird {
    /// The player saved this many beats on their actions thanks to this trait.
    pub extra_beats_today: Beats,
    /// Whether the player has this trait.
    pub is_active: bool,
    /// How much percentage discount the player gets on their actions
    /// (`<0; 1>`).
    ///
    /// The actual discount will be high in the beginning of the day and
    /// converge to 0 soon.
    pub full_discount: f32,
}

impl Default for EarlyBird {
    fn default() -> Self {
        Self {
            extra_beats_today: Beats(0),
            is_active: false,
            full_discount: Self::INITIAL_FULL_DISCOUNT,
        }
    }
}

impl Default for NightOwl {
    fn default() -> Self {
        Self {
            extra_beats_today: Beats(0),
            is_active: false,
            full_discount: Self::INITIAL_FULL_DISCOUNT,
        }
    }
}

impl PlayerStats {
    /// We can be sure that self is more than 1 because otherwise there's
    /// nothing to discount.
    pub(crate) fn discount_activity(
        &mut self,
        elapsed: Beats,
        cost: Beats,
    ) -> Beats {
        debug_assert!(cost.0 > 1);
        debug_assert!(elapsed.0 >= 0);

        let cost = self.traits.night_owl.discount_activity(elapsed, cost);
        let cost = self.traits.early_bird.discount_activity(elapsed, cost);

        cost.max(Beats(1))
    }
}

impl NightOwl {
    const INITIAL_FULL_DISCOUNT: f32 = 0.07;

    pub(crate) fn discount_activity(
        &mut self,
        elapsed: Beats,
        Beats(cost): Beats,
    ) -> Beats {
        if !self.is_active || cost <= 1 {
            return Beats(cost);
        }

        // `x^8` gives us a nice curve that's close to zero and picks up after
        // 3 quarters of the day fast to 1.
        //
        // <0; 1>
        let multiplier = elapsed.as_fraction_of_day().powi(8);

        // full discount is reached only at the end of the day, as the full
        // discount is proportional to the time elapsed
        let actual_discount = multiplier * self.full_discount;

        // - 1 because an activity must always cost at least 1 beat
        let discounted_beats = (cost as f32 * actual_discount)
            .clamp(0.0, (cost - 1) as f32)
            as isize;

        self.extra_beats_today += Beats(discounted_beats);

        Beats(cost - discounted_beats)
    }
}

impl EarlyBird {
    const INITIAL_FULL_DISCOUNT: f32 = 0.07;

    pub(crate) fn discount_activity(
        &mut self,
        elapsed: Beats,
        Beats(cost): Beats,
    ) -> Beats {
        if !self.is_active || cost <= 1 {
            return Beats(cost);
        }

        // `e^{-16x}` gives us a nice curve that's close to one and goes down
        // fast after the first quarter of the day.
        //
        // <0; 1>
        let multiplier = (-16.0 * elapsed.as_fraction_of_day()).exp();

        // full discount is only applicable at the beginning of the day, as the
        // full discount is inversely proportional to the time elapsed
        let actual_discount = multiplier * self.full_discount;

        // - 1 because an activity must always cost at least 1 beat
        let discounted_beats = (cost as f32 * actual_discount)
            .clamp(0.0, (cost - 1) as f32)
            as isize;

        self.extra_beats_today += Beats(discounted_beats);

        Beats(cost - discounted_beats)
    }
}
