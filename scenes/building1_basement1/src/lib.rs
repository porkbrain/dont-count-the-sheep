#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

mod autogen;
mod layout;
mod prelude;

use bevy::utils::Instant;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use prelude::*;
use serde::{Deserialize, Serialize};

use crate::layout::LayoutEntity;

/// Important scene struct.
/// We use it as identifiable generic in common logic.
#[derive(TypePath, Default)]
pub struct Building1Basement1;

impl TopDownScene for Building1Basement1 {
    type LocalTileKind = Building1Basement1TileKind;

    fn name() -> &'static str {
        "building1_basement1"
    }

    fn bounds() -> [i32; 4] {
        [-90, 40, -30, 0]
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
pub enum Building1Basement1TileKind {
    #[default]
    ElevatorZone,
    BasementDoorZone,
}

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum Building1Basement1Action {
    EnterElevator,
    EnterBasement,
}

pub fn add(app: &mut App) {
    info!("Adding Building1Basement1 to app");

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
        smooth_exit.run_if(Building1Basement1::in_quitting_state()),
    );

    info!("Added Building1Basement1 to app");
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
    info!("Entering Building1Basement1");
    next_state.set(Building1Basement1::running());
}

struct ExitAnimation {
    since: Instant,
    loading_screen_started: bool,
}

fn smooth_exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    mut exit_animation: Local<Option<ExitAnimation>>,
) {
    // this resets to None when we're done with the exit animation
    let ExitAnimation {
        since,
        loading_screen_started,
    } = exit_animation.get_or_insert_with(|| ExitAnimation {
        since: Instant::now(),
        loading_screen_started: false,
    });

    if !*loading_screen_started && since.elapsed() > START_LOADING_SCREEN_AFTER
    {
        debug!("Transitioning to first loading screen state");
        next_loading_screen_state.set(common_loading_screen::start_state());
        *loading_screen_started = true;
    }

    if since.elapsed()
        > START_LOADING_SCREEN_AFTER + settings.fade_loading_screen_in * 2
    {
        info!("Leaving Building1Basement1");

        // reset local state for next time
        *exit_animation = None;

        // be a good guy and don't invade other game loops with our controls
        controls.consume_all();

        use GlobalGameStateTransition::*;
        match *transition {
            Building1Basement1ToPlayerFloor => {
                next_state.set(GlobalGameState::LoadingBuilding1PlayerFloor);
            }
            Building1Basement1ToDowntown => {
                next_state.set(GlobalGameState::LoadingDowntown);
            }
            _ => {
                unreachable!(
                    "Invalid Building1Basement1 transition {transition:?}"
                );
            }
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
