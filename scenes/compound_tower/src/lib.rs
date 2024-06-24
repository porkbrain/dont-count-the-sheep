#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

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
pub struct CompoundTower;

impl TopDownScene for CompoundTower {
    type LocalTileKind = CompoundTowerTileKind;

    fn name() -> &'static str {
        "compound_tower"
    }

    fn bounds() -> [i32; 4] {
        [-70, 40, -50, 20]
    }
}

impl WithStandardStateSemantics for CompoundTower {
    fn loading() -> GlobalGameState {
        GlobalGameState::LoadingCompoundTower
    }

    fn running() -> GlobalGameState {
        GlobalGameState::AtCompoundTower
    }

    fn quitting() -> GlobalGameState {
        GlobalGameState::QuittingCompoundTower
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
pub enum CompoundTowerTileKind {
    #[default]
    ExitZone,
}

#[derive(Event, Reflect, Clone, strum::EnumString, PartialEq, Eq)]
pub enum CompoundTowerAction {
    ExitScene,
}

pub fn add(app: &mut App) {
    info!("Adding {CompoundTower:?} to app");

    app.add_event::<CompoundTowerAction>();

    top_down::default_setup_for_scene::<CompoundTower>(app);

    #[cfg(feature = "devtools")]
    top_down::dev_default_setup_for_scene::<CompoundTower>(app);

    debug!("Adding plugins");

    app.add_plugins(layout::Plugin);

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(CompoundTower::in_loading_state())
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(CompoundTower::in_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(CompoundTower::in_running_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(CompoundTower::in_quitting_state()),
    );

    info!("Added {CompoundTower:?} to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap<CompoundTower>>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering {CompoundTower:?}");
    next_state.set(CompoundTower::running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving {CompoundTower:?}");

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    use GlobalGameStateTransition::*;
    match *transition {
        TowerToCompound => {
            next_state.set(GlobalGameState::LoadingCompound);
        }
        _ => {
            unreachable!("Invalid {CompoundTower:?} transition {transition:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_valid_tscn_scene() {
        const TSCN: &str = include_str!(
            "../../../main_game/assets/scenes/compound_tower.tscn"
        );
        rscn::parse(TSCN, &default());
    }
}