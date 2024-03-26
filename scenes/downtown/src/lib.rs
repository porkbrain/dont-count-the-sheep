#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod actor;
mod autogen;
mod layout;
mod prelude;

use common_assets::{store::AssetList, AssetStore};
use common_loading_screen::LoadingScreenState;
use common_rscn::TscnInBevy;
use common_story::dialog::fe::portrait::in_portrait_dialog;
use layout::DowntownTileKind;
use main_game_lib::cutscene::in_cutscene;
use prelude::*;

/// Important scene struct.
/// We use it as identifiable generic in some common logic such as layout or
/// asset.
#[derive(TypePath, Default)]
pub(crate) struct Downtown;

pub fn add(app: &mut App) {
    info!("Adding downtown to app");

    common_top_down::default_setup_for_scene::<Downtown, _>(
        app,
        GlobalGameState::DowntownLoading,
        GlobalGameState::AtDowntown,
        GlobalGameState::DowntownQuitting,
    );

    #[cfg(feature = "devtools")]
    common_top_down::dev_default_setup_for_scene::<Downtown, _>(
        app,
        GlobalGameState::AtDowntown,
        GlobalGameState::DowntownQuitting,
    );

    debug!("Adding plugins");

    app.add_plugins((layout::Plugin, actor::Plugin));

    debug!("Adding camera");

    app.add_systems(
        OnEnter(GlobalGameState::DowntownLoading),
        common_visuals::camera::spawn,
    )
    .add_systems(
        OnExit(GlobalGameState::DowntownQuitting),
        common_visuals::camera::despawn,
    )
    .add_systems(
        Update,
        common_top_down::cameras::track_player_with_main_camera
            .after(common_top_down::actor::animate_movement::<Downtown>)
            .run_if(in_state(GlobalGameState::AtDowntown))
            .run_if(not(in_cutscene()))
            .run_if(not(in_portrait_dialog())),
    );

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::DowntownLoading),
        common_assets::store::insert_as_resource::<Downtown>,
    );
    app.add_systems(
        OnExit(GlobalGameState::DowntownQuitting),
        common_assets::store::remove_as_resource::<Downtown>,
    );

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(in_state(GlobalGameState::DowntownLoading))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_downtown.run_if(in_state(GlobalGameState::DowntownLoading)),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(in_state(GlobalGameState::AtDowntown))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        exit.run_if(in_state(GlobalGameState::DowntownQuitting)),
    );

    info!("Added downtown to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<common_top_down::TileMap<Downtown>>>,
    asset_server: Res<AssetServer>,
    asset_store: Res<AssetStore<Downtown>>,
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

fn enter_the_downtown(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering downtown");
    next_state.set(GlobalGameState::AtDowntown);
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
        DowntownToApartment => {
            next_state.set(GlobalGameState::ApartmentLoading);
        }
        _ => {
            unreachable!("Invalid Downtown transition {transition:?}");
        }
    }
}

impl TopDownScene for Downtown {
    type LocalTileKind = DowntownTileKind;

    fn name() -> &'static str {
        "downtown"
    }

    fn bounds() -> [i32; 4] {
        [-200, 200, -200, 200]
    }

    fn asset_path() -> &'static str {
        "maps/downtown.ron"
    }
}

impl TscnInBevy for Downtown {
    fn tscn_asset_path() -> &'static str {
        "scenes/downtown.tscn"
    }
}

impl AssetList for Downtown {
    fn folders() -> &'static [&'static str] {
        &["downtown"]
    }
}
