use super::{
    example::{Example2, Example3, Example4},
    AsChoice, AsSequence, Step,
};

/// These dialogs can be used in other dialogs as either choices or transitions.
#[derive(Clone, Copy, Debug)]
pub(super) enum DialogTargetChoice {
    Example2,
    Example3,
    Example4,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum DialogTargetGoto {
    Example2,
}

impl DialogTargetChoice {
    pub(super) fn sequence(&self) -> Vec<Step> {
        match self {
            Self::Example2 => Example2::sequence(),
            Self::Example3 => Example3::sequence(),
            Self::Example4 => Example4::sequence(),
        }
    }

    pub(super) fn choice(&self) -> &'static str {
        match self {
            Self::Example2 => Example2::choice(),
            Self::Example3 => Example3::choice(),
            Self::Example4 => Example4::choice(),
        }
    }
}

impl DialogTargetGoto {
    pub(super) fn sequence(&self) -> Vec<Step> {
        match self {
            Self::Example2 => Example2::sequence(),
        }
    }
}
