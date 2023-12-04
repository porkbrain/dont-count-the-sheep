use crate::{menu, prelude::*};
use bevy::time::Stopwatch;

#[derive(Component, Clone)]
pub(crate) struct Normal {
    /// weather has a limited number of jumps before it must reset
    /// via the [`Climate`]
    pub(crate) jumps: u8,
    /// there's a minimum delay between jumps
    pub(crate) last_jump: Stopwatch,
    /// there's a minimum delay between dashes
    pub(crate) last_dash: Stopwatch,
    /// there's a minimum delay between dips
    pub(crate) last_dip: Stopwatch,
    /// weather can only use its special ability once per reset
    pub(crate) can_use_special: bool,
}

#[derive(Component, Default)]
pub(crate) struct LoadingSpecial {
    /// Angle is given by the combination of keys pressed.
    /// See [`unit_circle_angle`].
    pub(crate) angle: Radians,
    /// special mode has a set duration after which it fires
    pub(crate) activated: Stopwatch,
    /// once special is fired, weather can only do the same amount of jumps
    /// as it had before
    pub(crate) jumps: u8,
}

#[derive(Component)]
pub(crate) struct InMenu {
    pub(crate) selection: menu::Selection,
    /// Remember the state before entering the menu.
    /// This is used to restore the state when exiting the menu.
    pub(crate) from_mode: Normal,
    /// We remove velocity when in menu to prevent weather from moving.
    /// This is used to restore the velocity when exiting the menu.
    pub(crate) from_velocity: Velocity,
    /// We remove transform when in menu to use its face as a cursor or
    /// something akin to that.
    pub(crate) from_transform: Transform,
}

impl Normal {
    pub(crate) fn tick(&mut self, time: &Time) -> &mut Self {
        self.last_jump.tick(time.delta());
        self.last_dash.tick(time.delta());
        self.last_dip.tick(time.delta());
        self
    }

    pub(crate) fn pause(&mut self) -> &mut Self {
        self.last_jump.pause();
        self.last_dash.pause();
        self.last_dip.pause();
        self
    }

    pub(crate) fn unpause(&mut self) -> &mut Self {
        self.last_jump.unpause();
        self.last_dash.unpause();
        self.last_dip.unpause();
        self
    }
}

impl LoadingSpecial {
    pub(crate) fn tick(&mut self, time: &Time) {
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
