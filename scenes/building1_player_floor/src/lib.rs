#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]
#![allow(clippy::too_many_arguments)]

mod actor;
mod autogen;
mod layout;
mod prelude;

use common_loading_screen::LoadingScreenState;
use prelude::*;
use serde::{Deserialize, Serialize};

use crate::layout::LayoutEntity;

/// Important scene struct.
/// We use it as identifiable generic in common logic.
#[derive(TypePath, Default, Debug)]
pub struct Building1PlayerFloor;

impl TopDownScene for Building1PlayerFloor {
    type LocalTileKind = Building1PlayerFloorTileKind;

    fn name() -> &'static str {
        "building1_player_floor"
    }

    fn bounds() -> [i32; 4] {
        [-150, 150, -150, 150]
    }
}

impl WithStandardStateSemantics for Building1PlayerFloor {
    fn loading() -> GlobalGameState {
        GlobalGameState::LoadingBuilding1PlayerFloor
    }

    fn running() -> GlobalGameState {
        GlobalGameState::AtBuilding1PlayerFloor
    }

    fn quitting() -> GlobalGameState {
        GlobalGameState::QuittingBuilding1PlayerFloor
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
#[reflect(Default)]
#[allow(clippy::enum_variant_names)]
pub enum Building1PlayerFloorTileKind {
    /// We want to darken the hallway when the player is in the apartment.
    HallwayZone,
    /// Everything that's in the player's apartment.
    PlayerApartmentZone,
    #[default]
    BedZone,
    ElevatorZone,
    PlayerDoorZone,
    MeditationZone,
    TeaZone,
    BottomLeftApartmentZone,
    BottomLeftApartmentDoorZone,
    BottomLeftApartmentBathroomDoorZone,
    BottomRightApartmentZone,
    BottomRightApartmentDoorZone,
}

#[derive(Event, Reflect, Clone, strum::EnumString, PartialEq, Eq)]
pub enum Building1PlayerFloorAction {
    EnterElevator,
    StartMeditation,
    Sleep,
    BrewTea,
}

pub fn add(app: &mut App) {
    info!("Adding {Building1PlayerFloor:?} to app");

    app.add_event::<Building1PlayerFloorAction>();

    top_down::default_setup_for_scene::<Building1PlayerFloor>(app);

    #[cfg(feature = "devtools")]
    top_down::dev_default_setup_for_scene::<Building1PlayerFloor>(app);

    debug!("Adding plugins");

    app.add_plugins((layout::Plugin, actor::Plugin));

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(Building1PlayerFloor::in_loading_state())
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(Building1PlayerFloor::in_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(Building1PlayerFloor::in_running_state())
            .run_if(in_state(common_loading_screen::wait_state())),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(Building1PlayerFloor::in_quitting_state()),
    );

    info!("Added {Building1PlayerFloor:?} to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap<Building1PlayerFloor>>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering {Building1PlayerFloor:?}");
    next_state.set(Building1PlayerFloor::running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving {Building1PlayerFloor:?}");

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    use GlobalGameStateTransition::*;
    match *transition {
        Building1PlayerFloorToBuilding1Basement1 => {
            next_state.set(GlobalGameState::LoadingBuilding1Basement1);
        }
        Building1PlayerFloorToMeditation => {
            next_state.set(GlobalGameState::LoadingMeditation);
        }
        Building1PlayerFloorToDowntown => {
            next_state.set(GlobalGameState::LoadingDowntown);
        }
        Sleeping => {
            next_state.set(GlobalGameState::LoadingBuilding1PlayerFloor);
        }
        _ => {
            unreachable!(
                "Invalid {Building1PlayerFloor:?} transition {transition:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_valid_tscn_scene() {
        const TSCN: &str = include_str!(
            "../../../main_game/assets/scenes/building1_player_floor.tscn",
        );
        rscn::parse(TSCN, &default());
    }
}
