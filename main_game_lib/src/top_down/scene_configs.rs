//! Some per scene configurations for the top-down game that are useful to have
//! tightly coupled with the top down crate.

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

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
    Aisle1,
    Aisle2,
    Aisle3,
    Aisle4,
    BasementDoor,
    Bed,
    BottomLeftApartmentBathroomDoor,
    BottomLeftApartmentDoor,
    BottomLeftApartment,
    BottomRightApartmentDoor,
    BottomRightApartment,
    Building1Entrance,
    ClinicEntrance,
    ClinicWardEntrance,
    CompoundEntrance,
    Door,
    Elevator,
    TowerEntrance,
    Exit,
    Fridges,
    FruitsAndVeggies,
    GoodWater,
    GoToDowntown,
    Hallway,
    MallEntrance,
    Meditation,
    PlantShopEntrance,
    PlayerApartment,
    PlayerDoor,
    SewersEntrance,
    Tea,
    TwinpeaksApartmentEntrance,
    UpperApartmentDoor,
    UpperApartmentWallHidden,
}
