use super::{
    example::{Example2, Example3, Example4},
    DialogFragment, Step,
};

/// These dialogs can be used in other dialogs as either choices or transitions.
pub(super) enum DialogTarget {
    Example2,
    Example3,
    Example4,
}

impl DialogTarget {
    pub(super) fn sequence(&self) -> Vec<Step> {
        match self {
            Self::Example2 => Example2::sequence(),
            Self::Example3 => Example3::sequence(),
            Self::Example4 => Example4::sequence(),
        }
    }
}
