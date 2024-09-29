#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod consts;
mod hoshi;
mod prelude;
mod room;
mod ui;
mod zindex;

use bevy::utils::Instant;
use common_assets::{store::AssetList, AssetStore};
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use consts::CLEAR_COLOR_BG;
use hoshi::Hoshi;
use main_game_lib::common_ext::QueryExt;
use prelude::*;

/// Important scene struct.
/// Identifies anything that's related to meditation.
#[derive(TypePath, Debug, Default)]
struct Meditation;

pub fn add(app: &mut App) {
    info!("Adding meditation to app");

    debug!("Adding plugins");

    app.add_plugins((ui::Plugin, hoshi::Plugin, room::Plugin));

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::LoadingMeditation),
        common_assets::store::insert_as_resource::<Meditation>,
    );
    app.add_systems(
        OnExit(GlobalGameState::QuittingMeditation),
        common_assets::store::remove_as_resource::<Meditation>,
    );

    debug!("Adding visuals");

    app.add_systems(
        Update,
        common_visuals::systems::flicker
            .run_if(in_state(GlobalGameState::MeditationInMenu)),
    );

    debug!("Adding physics");

    app.add_systems(
        FixedUpdate,
        common_physics::systems::apply_velocity
            .run_if(in_state(GlobalGameState::InGameMeditation)),
    );

    debug!("Adding game loop");

    // When everything is loaded, finish the loading process by hiding the
    // loading screen and entering the game.
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(in_state(GlobalGameState::LoadingMeditation))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    )
    .add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_game.run_if(in_state(GlobalGameState::LoadingMeditation)),
    );

    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::QuittingMeditation)),
    );

    info!("Added meditation to app");
}

/// Loading screen is being displayed.
/// When everything is loaded, finish the loading process by transitioning
/// to the loading screen state that means "we are ready to close the loading
/// screen".
///
/// Once loading screen fades out, [enter_the_game] will be called.
fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    asset_store: Res<AssetStore<Meditation>>,
    asset_server: Res<AssetServer>,

    hoshi: Query<Entity, With<Hoshi>>,
) {
    if hoshi.get_single_or_none().is_none() {
        return;
    }

    if !asset_store.are_all_loaded(&asset_server) {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_game(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    info!("Entering meditation game");
    next_state.set(GlobalGameState::InGameMeditation);
    cmd.insert_resource(ClearColor(CLEAR_COLOR_BG));
}

fn all_cleaned_up(
    mut cmd: Commands,
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
    settings: Res<LoadingScreenSettings>,

    mut since: Local<Option<Instant>>,
) {
    // this is reset to None when we're done with the exit animation
    let elapsed = since.get_or_insert_with(Instant::now).elapsed();
    if elapsed < settings.fade_loading_screen_in {
        return;
    }

    info!("Leaving meditation game");

    // reset local state for next time
    *since = None;

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    // back to the primary color
    cmd.insert_resource(ClearColor(PRIMARY_COLOR));

    use GlobalGameStateTransition::*;
    match *transition {
        RestartMeditation => {
            next_state.set(GlobalGameState::LoadingMeditation);
        }
        MeditationToBuilding1PlayerFloor => {
            next_state.set(WhichTopDownScene::Building1PlayerFloor.loading());
        }
        _ => {
            unreachable!("Invalid meditation transition {transition:?}");
        }
    }
}

impl AssetList for Meditation {
    fn folders() -> &'static [&'static str] {
        &[common_assets::meditation::FOLDER]
    }
}
