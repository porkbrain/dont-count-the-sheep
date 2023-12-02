use std::time::{Duration, Instant};

use crate::prelude::*;

pub(crate) type HasElapsedFor = (Duration, &'static [Kind]);

#[derive(Component)]
pub(crate) struct Transition {
    current: Kind,
    since: Instant,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    #[default]
    Default,
    Falling,
    Plunging,
}

impl Kind {
    pub(crate) fn index(&self) -> usize {
        match self {
            Kind::Default => 0,
            Kind::Falling => 2,
            Kind::Plunging => 3,
        }
    }
}

impl Transition {
    /// Does nothing if the current sprite is already the same.
    pub(crate) fn update(&mut self, kind: Kind) {
        if kind == self.current {
            return;
        }

        trace!("Updating sprite to {kind:?}");
        self.current = kind;
        self.since = Instant::now();
    }

    #[allow(dead_code)]
    pub(crate) fn has_elapsed(&self, duration: Duration) -> bool {
        self.since.elapsed() >= duration
    }

    /// Returns true if the current sprite is the same as the given kind or if
    /// the given duration has elapsed.
    pub(crate) fn has_elapsed_for(
        &self,
        (duration, kinds): HasElapsedFor,
    ) -> bool {
        kinds.contains(&self.current) || self.since.elapsed() >= duration
    }

    pub(crate) fn current_sprite_index(&self) -> usize {
        self.current.index()
    }

    #[allow(dead_code)]
    pub(crate) fn is_current(&self, kind: Kind) -> bool {
        self.current == kind
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            current: Kind::default(),
            since: Instant::now(),
        }
    }
}
