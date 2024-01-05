use bevy::{
    ecs::component::Component,
    time::{Stopwatch, Time},
};

use super::consts::{MIN_DASH_DELAY, MIN_DIP_DELAY, MIN_JUMP_DELAY};
use crate::prelude::{stopwatch_at, Radians};

#[derive(Component)]
pub(super) struct Normal {
    /// Hoshi has a limited number of jumps before it must reset
    /// via the [`Climate`]
    pub(super) jumps: usize,
    /// there's a minimum delay between jumps
    pub(super) last_jump: Stopwatch,
    /// there's a minimum delay between dashes
    pub(super) last_dash: Stopwatch,
    /// there's a minimum delay between dips
    pub(super) last_dip: Stopwatch,
    /// Hoshi can only use its special ability once per reset
    pub(super) can_use_special: bool,
}

#[derive(Component, Default)]
pub(crate) struct LoadingSpecial {
    /// Angle is given by the combination of keys pressed.
    /// See [`unit_circle_angle`].
    pub(super) angle: Radians,
    /// special mode has a set duration after which it fires
    pub(super) activated: Stopwatch,
    /// once special is fired, Hoshi can only do the same amount of jumps
    /// as it had before
    pub(super) jumps: usize,
}

impl Normal {
    pub(super) fn tick(&mut self, time: &Time) {
        self.last_jump.tick(time.delta());
        self.last_dash.tick(time.delta());
        self.last_dip.tick(time.delta());
    }
}

impl LoadingSpecial {
    pub(super) fn tick(&mut self, time: &Time) {
        self.activated.tick(time.delta());
    }
}

impl Default for Normal {
    fn default() -> Self {
        Self {
            jumps: 0,
            last_dash: stopwatch_at(MIN_DASH_DELAY),
            last_jump: stopwatch_at(MIN_JUMP_DELAY),
            last_dip: stopwatch_at(MIN_DIP_DELAY),
            can_use_special: true,
        }
    }
}
