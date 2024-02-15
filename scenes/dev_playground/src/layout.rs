use bevy_grid_squared::SquareLayout;
use lazy_static::lazy_static;
use main_game_lib::{common_top_down, common_top_down::TopDownScene};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{prelude::*, Test};

lazy_static! {
    static ref LAYOUT: SquareLayout = SquareLayout {
        square_size: 4.0,
        origin: vec2(0.0, 0.0),
    };
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TestTileKind>();
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
pub enum TestTileKind {
    #[default]
    ZoneA,
    ZoneB,
    ZoneC,
    ZoneD,
    ZoneE,
    ZoneF,
    ZoneG,
    ZoneH,
}

impl common_top_down::layout::Tile for TestTileKind {
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    fn is_zone(&self) -> bool {
        match self {
            Self::ZoneA
            | Self::ZoneB
            | Self::ZoneC
            | Self::ZoneD
            | Self::ZoneE
            | Self::ZoneF
            | Self::ZoneG
            | Self::ZoneH => true,
        }
    }

    fn zones_iter() -> impl Iterator<Item = Self> {
        Self::iter().filter(|kind| kind.is_zone())
    }
}

impl TopDownScene for Test {
    type LocalTileKind = TestTileKind;

    fn name() -> &'static str {
        "test"
    }

    fn bounds() -> [i32; 4] {
        [-100, 100, -100, 100]
    }

    fn asset_path() -> &'static str {
        "../../scenes/test/assets/map.ron"
    }

    fn layout() -> &'static SquareLayout {
        &LAYOUT
    }
}
