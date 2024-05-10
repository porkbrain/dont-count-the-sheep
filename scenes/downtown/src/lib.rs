#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod autogen;
mod layout;
mod prelude;

use common_loading_screen::LoadingScreenState;
use prelude::*;
use serde::{Deserialize, Serialize};

/// Important scene struct.
/// We use it as identifiable generic in some common logic such as layout or
/// asset.
#[derive(TypePath, Default)]
pub(crate) struct Downtown;

impl TopDownScene for Downtown {
    type LocalTileKind = DowntownTileKind;

    fn name() -> &'static str {
        "downtown"
    }

    fn bounds() -> [i32; 4] {
        [-350, 350, -500, 350]
    }
}

impl WithStandardStateSemantics for Downtown {
    fn loading() -> GlobalGameState {
        GlobalGameState::LoadingDowntown
    }

    fn running() -> GlobalGameState {
        GlobalGameState::AtDowntown
    }

    fn quitting() -> GlobalGameState {
        GlobalGameState::QuittingDowntown
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
pub enum DowntownTileKind {
    #[default]
    Building1Entrance,
    MallEntrance,
    ClinicEntrance,
}

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum DowntownAction {
    EnterBuilding1,
    EnterMall,
    EnterClinic,
}

pub fn add(app: &mut App) {
    info!("Adding downtown to app");

    app.add_event::<DowntownAction>();

    top_down::default_setup_for_scene::<Downtown>(app);

    #[cfg(feature = "devtools")]
    top_down::dev_default_setup_for_scene::<Downtown>(app);

    debug!("Adding plugins");

    app.add_plugins(layout::Plugin);

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(Downtown::in_loading_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(Downtown::in_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(Downtown::in_running_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(Downtown::in_quitting_state()),
    );

    info!("Added downtown to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap<Downtown>>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering downtown");
    next_state.set(Downtown::running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving downtown");

    // be a good guy and don't invade other game loops with our controls
    controls.consume_all();

    use GlobalGameStateTransition::*;
    match *transition {
        DowntownToBuilding1PlayerFloor => {
            next_state.set(GlobalGameState::LoadingBuilding1PlayerFloor);
        }
        DowntownToMall => {
            next_state.set(GlobalGameState::LoadingMall);
        }
        DowntownToClinic => {
            next_state.set(GlobalGameState::LoadingClinic);
        }
        _ => {
            unreachable!("Invalid Downtown transition {transition:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_valid_tscn_scene() {
        const TSCN: &str =
            include_str!("../../../main_game/assets/scenes/downtown.tscn",);
        rscn::parse(TSCN, &default());
    }
}
