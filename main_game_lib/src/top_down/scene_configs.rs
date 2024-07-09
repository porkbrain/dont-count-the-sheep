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
    BasementDoor = 4,
    Bed = 5,
    BottomLeftApartmentBathroomDoor = 6,
    BottomLeftApartmentDoor = 7,
    BottomLeftApartment = 8,
    BottomRightApartmentDoor = 9,
    BottomRightApartment = 10,
    Building1Entrance = 11,
    ClinicEntrance = 12,
    ClinicWardEntrance = 13,
    CompoundEntrance = 14,
    Door = 15,
    Elevator = 16,
    TowerEntrance = 17,
    Exit = 18,
    Fridges = 19,
    FruitsAndVeggies = 20,
    GoodWater = 21,
    GoToDowntown = 22,
    Hallway = 23,
    MallEntrance = 24,
    Meditation = 25,
    PlantShopEntrance = 26,
    PlayerApartment = 27,
    PlayerDoor = 28,
    SewersEntrance = 29,
    Tea = 30,
    TwinpeaksApartmentEntrance = 31,
    UpperApartmentDoor = 32,
    UpperApartmentWallHidden = 33,
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
