use bevy::{
    ecs::component::Component,
    time::{Stopwatch, Time},
};

use crate::prelude::Radians;

pub(super) trait Mode {
    fn tick(&mut self, time: &Time);
}

#[derive(Component)]
pub(super) struct Normal {
    /// weather has a limited number of jumps before it must reset
    /// via the [`Climate`]
    pub(super) jumps: u8,
    /// there's a minimum delay between jumps
    pub(super) last_jump: Stopwatch,
    /// there's a minimum delay between dashes
    pub(super) last_dash: Stopwatch,
    /// there's a minimum delay between dips
    pub(super) last_dip: Stopwatch,
    /// weather can only use its special ability once per reset
    pub(super) can_use_special: bool,
}

#[derive(Component, Default)]
pub(super) struct LoadingSpecial {
    /// Angle is given by the combination of keys pressed.
    /// See [`unit_circle_angle`].
    ///
    /// If the no angle was chosen then the special is canceled.
    pub(super) angle: Option<Radians>,
    /// special mode has a set duration after which it fires
    pub(super) activated: Stopwatch,
    /// once special is fired, weather can only do the same amount of jumps
    /// as it had before
    pub(super) jumps: u8,
}

impl Mode for Normal {
    fn tick(&mut self, time: &Time) {
        self.last_jump.tick(time.delta());
        self.last_dash.tick(time.delta());
        self.last_dip.tick(time.delta());
    }
}

impl Mode for LoadingSpecial {
    fn tick(&mut self, time: &Time) {
        self.activated.tick(time.delta());
    }
}

impl Default for Normal {
    fn default() -> Self {
        Self {
            jumps: 0,
            last_dash: Stopwatch::default(),
            last_jump: Stopwatch::default(),
            last_dip: Stopwatch::default(),
            can_use_special: true,
        }
    }
}
