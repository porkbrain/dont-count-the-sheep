#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

mod layout;
mod prelude;

use common_loading_screen::LoadingScreenState;
use prelude::*;

use crate::layout::LayoutEntity;

/// Important scene struct.
/// We use it as identifiable generic in common logic.
#[derive(TypePath, Default, Debug)]
pub struct Building1Basement1;

impl TopDownScene for Building1Basement1 {
    fn name() -> &'static str {
        "building1_basement1"
    }
}

impl WithStandardStateSemantics for Building1Basement1 {
    fn loading() -> GlobalGameState {
        GlobalGameState::LoadingBuilding1Basement1
    }

    fn running() -> GlobalGameState {
        GlobalGameState::AtBuilding1Basement1
    }

    fn quitting() -> GlobalGameState {
        GlobalGameState::QuittingBuilding1Basement1
    }
}

#[derive(Event, Reflect, Clone, strum::EnumString, Eq, PartialEq)]
pub enum Building1Basement1Action {
    EnterElevator,
    EnterBasement2,
}

pub fn add(app: &mut App) {
    info!("Adding {Building1Basement1:?} to app");

    app.add_event::<Building1Basement1Action>();

    top_down::default_setup_for_scene::<Building1Basement1>(app);

    #[cfg(feature = "devtools")]
    top_down::dev_default_setup_for_scene::<Building1Basement1>(app);

    debug!("Adding plugins");

    app.add_plugins(layout::Plugin);

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(Building1Basement1::in_loading_state())
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(Building1Basement1::in_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(Building1Basement1::in_running_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(Building1Basement1::in_quitting_state()),
    );

    info!("Added {Building1Basement1:?} to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap<Building1Basement1>>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering {Building1Basement1:?}");
    next_state.set(Building1Basement1::running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving {Building1Basement1:?}");

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    use GlobalGameStateTransition::*;
    match *transition {
        Building1Basement1ToPlayerFloor => {
            next_state.set(GlobalGameState::LoadingBuilding1PlayerFloor);
        }
        Building1Basement1ToDowntown => {
            next_state.set(GlobalGameState::LoadingDowntown);
        }
        Building1Basement1ToBasement2 => {
            next_state.set(GlobalGameState::LoadingBuilding1Basement2);
        }
        _ => {
            unreachable!(
                "Invalid {Building1Basement1:?} transition {transition:?}"
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
            "../../../main_game/assets/scenes/building1_basement1.tscn",
        );
        rscn::parse(TSCN, &default());
    }
}
