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
pub struct TwinpeaksApartment;

impl TopDownScene for TwinpeaksApartment {
    type LocalTileKind = TwinpeaksApartmentTileKind;

    fn name() -> &'static str {
        "twinpeaks_apartment"
    }

    fn bounds() -> [i32; 4] {
        [-70, 60, -50, 20]
    }
}

impl WithStandardStateSemantics for TwinpeaksApartment {
    fn loading() -> GlobalGameState {
        GlobalGameState::LoadingTwinpeaksApartment
    }

    fn running() -> GlobalGameState {
        GlobalGameState::AtTwinpeaksApartment
    }

    fn quitting() -> GlobalGameState {
        GlobalGameState::QuittingTwinpeaksApartment
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
pub enum TwinpeaksApartmentTileKind {
    #[default]
    ExitZone,
}

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum TwinpeaksApartmentAction {
    ExitScene,
}

pub fn add(app: &mut App) {
    info!("Adding {TwinpeaksApartment:?} to app");

    app.add_event::<TwinpeaksApartmentAction>();

    top_down::default_setup_for_scene::<TwinpeaksApartment>(app);

    #[cfg(feature = "devtools")]
    top_down::dev_default_setup_for_scene::<TwinpeaksApartment>(app);

    debug!("Adding plugins");

    app.add_plugins(layout::Plugin);

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(TwinpeaksApartment::in_loading_state())
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(TwinpeaksApartment::in_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(TwinpeaksApartment::in_running_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(TwinpeaksApartment::in_quitting_state()),
    );

    info!("Added {TwinpeaksApartment:?} to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap<TwinpeaksApartment>>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering {TwinpeaksApartment:?}");
    next_state.set(TwinpeaksApartment::running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving {TwinpeaksApartment:?}");

    // be a good guy and don't invade other game loops with our controls
    controls.consume_all();

    use GlobalGameStateTransition::*;
    match *transition {
        TwinpeaksApartmentToDowntown => {
            next_state.set(GlobalGameState::LoadingDowntown);
        }
        _ => {
            unreachable!(
                "Invalid {TwinpeaksApartment:?} transition {transition:?}"
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
            "../../../main_game/assets/scenes/twinpeaks_apartment.tscn"
        );
        rscn::parse(TSCN, &default());
    }
}