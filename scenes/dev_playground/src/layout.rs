use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DevPlaygroundTileKind>();
    }
}

/// We arbitrarily derive the [`Default`] to allow reflection.
/// It does not have a meaningful default value.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumIter,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    Serialize,
    strum::Display,
)]
#[reflect(Default)]
#[allow(clippy::enum_variant_names)]
pub enum DevPlaygroundTileKind {
    #[default]
    ZoneA,
    ZoneB,
    ZoneC,
    ZoneD,
    ZoneE,
    ZoneF,
    ZoneG,
    ZoneH,
    ZoneI,
    ZoneJ,
    ZoneK,
}

impl common_top_down::layout::Tile for DevPlaygroundTileKind {
    #[inline]
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    #[inline]
    fn is_zone(&self) -> bool {
        match self {
            Self::ZoneA
            | Self::ZoneB
            | Self::ZoneC
            | Self::ZoneD
            | Self::ZoneE
            | Self::ZoneF
            | Self::ZoneG
            | Self::ZoneH
            | Self::ZoneI
            | Self::ZoneJ
            | Self::ZoneK => true,
        }
    }

    #[inline]
    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}
