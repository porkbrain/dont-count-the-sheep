use bevy::{
    asset::AssetServer, ecs::system::Commands, reflect::DynamicTypePath,
};
use common_store::GlobalStore;

use super::{
    apartment_elevator::{
        EnteredTheElevator, LeaveTheElevatorToStayOnCurrentFloor,
        TakeTheElevatorToFirstFloor, TakeTheElevatorToGroundFloor,
    },
    AsChoice, AsSequence, Step,
};

/// These are the root dialogs that can be spawned from the game.
/// Useful to avoid generics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogRoot {
    /// Enters the elevator at the apartment...
    EnteredTheElevator,
}

/// These dialogs can be used in other dialogs as either choices or transitions.
#[allow(clippy::enum_variant_names)] // this can be deleted with more dialog
#[derive(Clone, Copy, Debug)]
pub(super) enum DialogTargetChoice {
    TakeTheElevatorToGroundFloor,
    TakeTheElevatorToFirstFloor,
    LeaveTheElevatorToStayOnCurrentFloor,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum DialogTargetGoto {
    EnteredTheElevator,
}

impl DialogRoot {
    /// Spawns given dialog.
    /// It's sufficient to call this method, then everything else is taken care
    /// of by the dialog system.
    /// Just make sure that the movement has been taken away from the player.
    pub fn spawn(
        self,
        cmd: &mut Commands,
        asset_server: &AssetServer,
        global_store: &GlobalStore,
    ) {
        match self {
            Self::EnteredTheElevator => {
                EnteredTheElevator::spawn(cmd, asset_server, global_store)
            }
        }
    }
}

impl DialogTargetChoice {
    pub(super) fn sequence(&self) -> Vec<Step> {
        match self {
            Self::TakeTheElevatorToGroundFloor => {
                TakeTheElevatorToGroundFloor::sequence()
            }
            Self::TakeTheElevatorToFirstFloor => {
                TakeTheElevatorToFirstFloor::sequence()
            }
            Self::LeaveTheElevatorToStayOnCurrentFloor => {
                LeaveTheElevatorToStayOnCurrentFloor::sequence()
            }
        }
    }

    pub(super) fn choice(&self) -> &'static str {
        match self {
            Self::TakeTheElevatorToGroundFloor => {
                TakeTheElevatorToGroundFloor::choice()
            }
            Self::TakeTheElevatorToFirstFloor => {
                TakeTheElevatorToFirstFloor::choice()
            }
            Self::LeaveTheElevatorToStayOnCurrentFloor => {
                LeaveTheElevatorToStayOnCurrentFloor::choice()
            }
        }
    }

    pub(super) fn type_path(&self) -> &'static str {
        match self {
            Self::TakeTheElevatorToGroundFloor => {
                TakeTheElevatorToGroundFloor.reflect_type_path()
            }
            Self::TakeTheElevatorToFirstFloor => {
                TakeTheElevatorToFirstFloor.reflect_type_path()
            }
            Self::LeaveTheElevatorToStayOnCurrentFloor => {
                LeaveTheElevatorToStayOnCurrentFloor.reflect_type_path()
            }
        }
    }
}

impl DialogTargetGoto {
    pub(super) fn sequence(&self) -> Vec<Step> {
        match self {
            Self::EnteredTheElevator => EnteredTheElevator::sequence(),
        }
    }

    pub(super) fn type_path(&self) -> &'static str {
        match self {
            Self::EnteredTheElevator => EnteredTheElevator.reflect_type_path(),
        }
    }
}
