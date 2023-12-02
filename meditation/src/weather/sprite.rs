use super::{consts, ActionEvent};
use crate::prelude::*;
use std::time::{Duration, Instant};

#[derive(Component)]
pub(crate) struct Transition {
    current_body: BodyKind,
    current_body_set_at: Instant,
    current_face: FaceKind,
    current_face_set_at: Instant,
    /// This is updated each time an action is received in
    /// [`crate::weather::anim::sprite`].
    last_action: Option<(ActionEvent, Instant)>,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub(crate) enum BodyKind {
    #[default]
    Default,
    Falling,
    Plunging,
    BootyDanceLeft,
    BootyDanceRight,
    SpearingTowards,
    SlowingSpearingTowards,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub(crate) enum FaceKind {
    #[default]
    Default,
    Happy,
    Surprised,
    Intense,
}

impl BodyKind {
    pub(crate) fn index(&self) -> usize {
        use consts::BODY_ATLAS_COLS as COLS;
        use BodyKind::*;

        match self {
            // first row
            BodyKind::Default => 0,
            // second row
            Falling => COLS,
            Plunging => COLS + 1,
            // third row
            BootyDanceLeft => COLS * 2,
            BootyDanceRight => COLS * 2 + 1,
            // fourth row
            SpearingTowards => COLS * 3,
            SlowingSpearingTowards => COLS * 3 + 1,
        }
    }

    pub(crate) fn should_hide_face(&self) -> bool {
        matches!(self, Self::BootyDanceLeft | Self::BootyDanceRight)
    }
}

impl FaceKind {
    pub(crate) fn index(&self) -> usize {
        use consts::FACE_ATLAS_COLS as COLS;
        use FaceKind::*;

        match self {
            // first row
            Surprised => 3,
            // second row
            Happy => COLS,
            Default => COLS + 2,
            // third row
            Intense => COLS * 2,
        }
    }
}

impl Transition {
    #[inline]
    pub(crate) fn current_body(&self) -> BodyKind {
        self.current_body
    }

    /// Does nothing if the current body is already the same.
    #[inline]
    pub(crate) fn update_body(&mut self, kind: BodyKind) {
        if kind == self.current_body {
            return;
        }

        trace!("Updating body to {kind:?}");
        self.current_body = kind;
        self.current_body_set_at = Instant::now();
    }

    /// Does not check if the current body is already the same.
    #[inline]
    pub(crate) fn force_update_body(&mut self, kind: BodyKind) {
        trace!("Force updating body to {kind:?}");
        self.current_body = kind;
        self.current_body_set_at = Instant::now();
    }

    #[inline]
    pub(crate) fn has_elapsed_since_body_change(
        &self,
        duration: Duration,
    ) -> bool {
        self.current_body_set_at.elapsed() >= duration
    }

    #[inline]
    pub(crate) fn current_body_index(&self) -> usize {
        self.current_body.index()
    }

    #[allow(dead_code)]
    pub(crate) fn is_current_body(&self, kind: BodyKind) -> bool {
        self.current_body == kind
    }
}

impl Transition {
    #[inline]
    pub(crate) fn has_elapsed_since_face_change(
        &self,
        duration: Duration,
    ) -> bool {
        self.current_face_set_at.elapsed() >= duration
    }

    #[inline]
    pub(crate) fn update_face(&mut self, kind: FaceKind) {
        if kind == self.current_face {
            return;
        }

        trace!("Updating face to {kind:?}");
        self.current_face = kind;
        self.current_face_set_at = Instant::now();
    }

    #[inline]
    pub(crate) fn current_face_index(&self) -> usize {
        self.current_face.index()
    }
}

impl Transition {
    #[inline]
    pub(crate) fn last_action_within(
        &self,
        within: Duration,
    ) -> Option<ActionEvent> {
        self.last_action
            .as_ref()
            .filter(|(_, at)| at.elapsed() <= within)
            .map(|(action, _)| *action)
    }

    #[inline]
    pub(crate) fn update_action(&mut self, action: ActionEvent) {
        self.last_action = Some((action, Instant::now()));
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            current_body: BodyKind::default(),
            current_body_set_at: Instant::now(),
            current_face: FaceKind::default(),
            current_face_set_at: Instant::now(),
            last_action: None,
        }
    }
}
