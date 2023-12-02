pub(crate) use crate::generic::*;
pub(crate) use crate::zindex;
pub(crate) use bevy::prelude::*;
pub(crate) use std::time::Duration;

pub(crate) const fn from_millis(millis: u64) -> Duration {
    Duration::from_millis(millis)
}
