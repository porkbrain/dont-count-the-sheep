use super::{consts, ActionEvent};
use crate::prelude::*;
use std::time::{Duration, Instant};

#[derive(Component)]
pub(crate) struct Transition {
    current: Kind,
    at: Instant,
    /// This is updated each time an action is received in
    /// [`crate::weather::anim::sprite`].
    last_action: Option<(ActionEvent, Instant)>,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    #[default]
    Default,
    Falling,
    Plunging,
    BootyDanceLeft,
    BootyDanceRight,
}

impl Kind {
    pub(crate) fn index(&self) -> usize {
        use consts::SPRITE_ATLAS_COLS as COLS;
        use Kind::*;

        match self {
            // first row
            Kind::Default => 0,
            // second row
            Falling => COLS,
            Plunging => COLS + 1,
            // third row
            BootyDanceLeft => COLS * 2,
            BootyDanceRight => COLS * 2 + 1,
        }
    }
}

impl Transition {
    pub(crate) fn current_sprite(&self) -> Kind {
        self.current
    }

    /// Does nothing if the current sprite is already the same.
    pub(crate) fn update_sprite(&mut self, kind: Kind) {
        if kind == self.current {
            return;
        }

        trace!("Updating sprite to {kind:?}");
        self.current = kind;
        self.at = Instant::now();
    }

    /// Does not check if the current sprite is already the same.
    pub(crate) fn force_update_sprite(&mut self, kind: Kind) {
        trace!("Force updating sprite to {kind:?}");
        self.current = kind;
        self.at = Instant::now();
    }

    pub(crate) fn has_elapsed_since_sprite_change(
        &self,
        duration: Duration,
    ) -> bool {
        self.at.elapsed() >= duration
    }

    pub(crate) fn current_sprite_index(&self) -> usize {
        self.current.index()
    }

    #[allow(dead_code)]
    pub(crate) fn is_current_sprite(&self, kind: Kind) -> bool {
        self.current == kind
    }
}

impl Transition {
    pub(crate) fn last_action_within(
        &self,
        within: Duration,
    ) -> Option<ActionEvent> {
        self.last_action
            .as_ref()
            .filter(|(_, at)| at.elapsed() <= within)
            .map(|(action, _)| *action)
    }

    pub(crate) fn update_action(&mut self, action: ActionEvent) {
        self.last_action = Some((action, Instant::now()));
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            current: Kind::default(),
            at: Instant::now(),
            last_action: None,
        }
    }
}
