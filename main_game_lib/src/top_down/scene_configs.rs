//! Some per scene configurations for the top-down game that are useful to have
//! tightly coupled with the top down crate.

use crate::WhichTopDownScene;

impl WhichTopDownScene {
    /// Returns snake case version of the scene name.
    pub fn snake_case(self) -> String {
        untools::camel_to_snake(self.as_ref(), false)
    }
}

/// TODO: clean up
#[allow(missing_docs)]
pub enum LocalTileKind {
    Aisle1,
    Aisle2,
    Aisle3,
    Aisle4,
    BasementDoorZone,
    BedZone,
    BottomLeftApartmentBathroomDoorZone,
    BottomLeftApartmentDoorZone,
    BottomLeftApartmentZone,
    BottomRightApartmentDoorZone,
    BottomRightApartmentZone,
    Building1Entrance,
    ClinicEntrance,
    ClinicWardEntrance,
    CompoundEntrance,
    DoorZone,
    ElevatorZone,
    EnterTowerZone,
    ExitZone,
    Fridges,
    FruitsAndVeggies,
    GoodWater,
    GoToDowntownZone,
    HallwayZone,
    MallEntrance,
    MeditationZone,
    PlantShopEntrance,
    PlayerApartmentZone,
    PlayerDoorZone,
    SewersEntrance,
    TeaZone,
    TwinpeaksApartmentEntrance,
    UpperApartmentDoorZone,
    UpperApartmentWallHiddenZone,
}
