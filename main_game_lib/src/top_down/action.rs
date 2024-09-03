use crate::prelude::*;

/// List of all possible actions the player can take in the top down scenes.
#[derive(Event, Reflect, Clone, strum::EnumString, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum TopDownAction {
    BrewTea,
    EnterBasement2,
    EnterBuilding1,
    EnterClinic,
    EnterClinicWard,
    EnterCompound,
    EnterElevator,
    EnterMall,
    EnterPlantShop,
    EnterSewers,
    EnterTower,
    EnterTwinpeaksApartment,
    Exit,
    GoToDowntown,
    Sleep,
    StartGingerCatDialog,
    StartMeditation,
}
