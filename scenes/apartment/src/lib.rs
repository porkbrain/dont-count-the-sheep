#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]

mod actor;
mod autogen;
mod cameras;
mod consts;
mod layout;
mod prelude;
mod zindex;

use actor::ApartmentAction;
use bevy::utils::Instant;
use common_assets::{store::AssetList, AssetStore};
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_top_down::TopDownScene;
use consts::START_LOADING_SCREEN_AFTER;
use layout::ApartmentTileKind;
use prelude::*;

/// Important scene struct.
/// We use it as identifiable generic in some common logic such as layout or
/// asset.
#[derive(TypePath, Default)]
pub struct Apartment;

pub fn add(app: &mut App) {
    info!("Adding apartment to app");

    common_top_down::default_setup_for_scene::<Apartment, _>(
        app,
        GlobalGameState::ApartmentLoading,
        GlobalGameState::InApartment,
        GlobalGameState::ApartmentQuitting,
    );

    #[cfg(feature = "dev")]
    common_top_down::dev_default_setup_for_scene::<Apartment, _>(
        app,
        GlobalGameState::InApartment,
        GlobalGameState::ApartmentQuitting,
    );

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, layout::Plugin, actor::Plugin));

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::ApartmentLoading),
        common_assets::store::insert_as_resource::<Apartment>,
    );
    app.add_systems(
        OnExit(GlobalGameState::ApartmentQuitting),
        common_assets::store::remove_as_resource::<Apartment>,
    );

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(in_state(GlobalGameState::ApartmentLoading))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_apartment.run_if(in_state(GlobalGameState::ApartmentLoading)),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(in_state(GlobalGameState::InApartment))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        smooth_exit.run_if(in_state(GlobalGameState::ApartmentQuitting)),
    );

    info!("Added apartment to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<common_top_down::TileMap<Apartment>>>,
    asset_server: Res<AssetServer>,
    asset_store: Res<AssetStore<Apartment>>,
) {
    if map.is_none() {
        return;
    }

    if !asset_store.are_all_loaded(&asset_server) {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_apartment(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering apartment");
    next_state.set(GlobalGameState::InApartment);
}

struct ExitAnimation {
    since: Instant,
    loading_screen_started: bool,
}

// TODO: this can be done easier in a new version of bevy where delay timers
// exist
fn smooth_exit(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
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
        info!("Leaving apartment");

        // reset local state for next time
        *exit_animation = None;

        // be a good guy and don't invade other game loops with our controls
        controls.consume_all();

        match stack.pop_next_for(GlobalGameState::ApartmentQuitting) {
            // possible restart or change of game loop
            Some(next) => next_state.set(next),
            None => {
                unreachable!(
                    "There's nowhere to transition from ApartmentQuitting"
                );
            }
        }
    }
}

impl TopDownScene for Apartment {
    type LocalTileKind = ApartmentTileKind;
    type LocalActionEvent = ApartmentAction;

    fn name() -> &'static str {
        "apartment"
    }

    fn bounds() -> [i32; 4] {
        [-80, 40, -30, 20]
    }

    fn asset_path() -> &'static str {
        assets::MAP
    }
}

impl AssetList for Apartment {
    fn folders() -> &'static [&'static str] {
        &[assets::FOLDER]
    }
}

impl std::fmt::Display for Apartment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Apartment::name())
    }
}
