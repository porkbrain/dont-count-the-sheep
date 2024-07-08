//! Some per scene configurations for the top-down game that are useful to have
//! tightly coupled with the top down crate.

use bevy::{ecs::entity::Entity, reflect::Reflect};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use super::layout::Tile;
use crate::WhichTopDownScene;

impl WhichTopDownScene {
    /// Returns snake case version of the scene name.
    pub fn snake_case(self) -> String {
        untools::camel_to_snake(self.as_ref(), false)
    }
}

/// TODO: clean up
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
)]
#[allow(missing_docs)]
pub enum ZoneTileKind {
    #[default]
    Empty = 34,
    Aisle1 = 0,
    Aisle2 = 1,
    Aisle3 = 2,
    Aisle4 = 3,
    BasementDoorZone = 4,
    BedZone = 5,
    BottomLeftApartmentBathroomDoorZone = 6,
    BottomLeftApartmentDoorZone = 7,
    BottomLeftApartmentZone = 8,
    BottomRightApartmentDoorZone = 9,
    BottomRightApartmentZone = 10,
    Building1Entrance = 11,
    ClinicEntrance = 12,
    ClinicWardEntrance = 13,
    CompoundEntrance = 14,
    DoorZone = 15,
    ElevatorZone = 16,
    EnterTowerZone = 17,
    ExitZone = 18,
    Fridges = 19,
    FruitsAndVeggies = 20,
    GoodWater = 21,
    GoToDowntownZone = 22,
    HallwayZone = 23,
    MallEntrance = 24,
    MeditationZone = 25,
    PlantShopEntrance = 26,
    PlayerApartmentZone = 27,
    PlayerDoorZone = 28,
    SewersEntrance = 29,
    TeaZone = 30,
    TwinpeaksApartmentEntrance = 31,
    UpperApartmentDoorZone = 32,
    UpperApartmentWallHiddenZone = 33,
}

impl Tile for ZoneTileKind {
    fn is_walkable(&self, _: Entity) -> bool {
        true
    }

    fn is_zone(&self) -> bool {
        true
    }

    fn zones_iter() -> impl Iterator<Item = Self> {
        ZoneTileKind::iter()
    }
}
