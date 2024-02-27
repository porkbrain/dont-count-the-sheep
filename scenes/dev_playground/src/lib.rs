#![doc = include_str!("../README.md")]

mod actor;
mod autogen;
mod cameras;
mod layout;
mod prelude;

use common_assets::{store::AssetList, AssetStore};
use common_top_down::TopDownScene;
use layout::DevPlaygroundTileKind;
use prelude::*;

/// Important scene struct.
/// We use it as identifiable generic in some common logic such as layout or
/// asset.
#[derive(TypePath, Default)]
pub struct DevPlayground;

pub fn add(app: &mut App) {
    info!("Adding dev playground to app");

    common_top_down::default_setup_for_scene::<DevPlayground, _>(
        app,
        GlobalGameState::Blank,
        GlobalGameState::InDevPlayground,
        GlobalGameState::Exit,
    );

    common_top_down::dev_default_setup_for_scene::<DevPlayground, _>(
        app,
        GlobalGameState::InDevPlayground,
        GlobalGameState::Exit,
    );

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, layout::Plugin, actor::Plugin));

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::Blank),
        common_assets::store::insert_as_resource::<DevPlayground>,
    );

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded.run_if(in_state(GlobalGameState::Blank)),
    );

    info!("Added test to app");
}

fn finish_when_everything_loaded(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    map: Option<Res<common_top_down::TileMap<DevPlayground>>>,
    asset_server: Res<AssetServer>,
    asset_store: Res<AssetStore<DevPlayground>>,
) {
    if map.is_none() {
        return;
    }

    if !asset_store.are_all_loaded(&asset_server) {
        return;
    }

    debug!("All assets loaded");
    next_state.set(GlobalGameState::InDevPlayground);
}

impl TopDownScene for DevPlayground {
    type LocalTileKind = DevPlaygroundTileKind;

    fn name() -> &'static str {
        "dev_playground"
    }

    fn bounds() -> [i32; 4] {
        [-100, 100, -100, 100]
    }

    fn asset_path() -> &'static str {
        "../../scenes/dev_playground/assets/map.ron"
    }
}

impl AssetList for DevPlayground {}

impl std::fmt::Display for DevPlayground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", DevPlayground::name())
    }
}
