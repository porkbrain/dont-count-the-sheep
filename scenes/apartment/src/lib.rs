#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

mod actor;
mod autogen;
mod layout;
mod prelude;

use bevy::utils::Instant;
use common_assets::{store::AssetList, AssetStore};
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_rscn::TscnInBevy;
use common_story::dialog::fe::portrait::in_portrait_dialog;
use layout::ApartmentTileKind;
use main_game_lib::cutscene::in_cutscene;
use prelude::*;

use crate::layout::LayoutEntity;

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

    #[cfg(feature = "devtools")]
    common_top_down::dev_default_setup_for_scene::<Apartment, _>(
        app,
        GlobalGameState::InApartment,
        GlobalGameState::ApartmentQuitting,
    );

    debug!("Adding plugins");

    app.add_plugins((layout::Plugin, actor::Plugin));

    debug!("Adding camera");

    app.add_systems(
        OnEnter(GlobalGameState::ApartmentLoading),
        common_visuals::camera::spawn,
    )
    .add_systems(
        OnExit(GlobalGameState::ApartmentQuitting),
        common_visuals::camera::despawn,
    )
    .add_systems(
        FixedUpdate,
        common_top_down::cameras::track_player_with_main_camera
            .after(common_top_down::actor::animate_movement::<Apartment>)
            .run_if(in_state(GlobalGameState::InApartment))
            .run_if(not(in_cutscene()))
            .run_if(not(in_portrait_dialog())),
    );

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
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
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
        info!("Leaving apartment");

        // reset local state for next time
        *exit_animation = None;

        // be a good guy and don't invade other game loops with our controls
        controls.consume_all();

        use GlobalGameStateTransition::*;
        match *transition {
            ApartmentToMeditation => {
                next_state.set(GlobalGameState::MeditationLoading);
            }
            ApartmentToDowntown => {
                next_state.set(GlobalGameState::DowntownLoading);
            }
            _ => {
                unreachable!("Invalid apartment transition {transition:?}");
            }
        }
    }
}

impl TopDownScene for Apartment {
    type LocalTileKind = ApartmentTileKind;

    fn name() -> &'static str {
        "apartment"
    }

    fn bounds() -> [i32; 4] {
        [-80, 40, -30, 20]
    }

    fn asset_path() -> &'static str {
        "maps/apartment.ron"
    }
}

impl TscnInBevy for Apartment {
    fn tscn_asset_path() -> &'static str {
        "scenes/apartment.tscn"
    }
}

impl AssetList for Apartment {
    fn folders() -> &'static [&'static str] {
        &["apartment"]
    }
}

impl std::fmt::Display for Apartment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Apartment::name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_valid_tscn_scene() {
        const TSCN: &str =
            include_str!("../../../main_game/assets/scenes/apartment.tscn");
        common_rscn::parse(TSCN, &default());
    }
}
