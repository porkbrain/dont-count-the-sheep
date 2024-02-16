#![doc = include_str!("../README.md")]

mod actor;
mod cameras;
mod layout;
mod prelude;

use common_assets::{store::AssetList, AssetStore};
use common_story::{portrait_dialog::in_portrait_dialog, DialogAssets};
use main_game_lib::{
    common_action::{interaction_just_pressed, move_action_just_pressed},
    common_top_down::TopDownScene,
};
use prelude::*;

/// Important scene struct.
/// We use it as identifiable generic in some common logic such as layout or
/// asset.
#[derive(TypePath, Default)]
pub struct DevPlayground;

pub fn add(app: &mut App) {
    info!("Adding dev playground to app");

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, layout::Plugin, actor::Plugin));

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::Blank),
        (
            common_assets::store::insert_as_resource::<DevPlayground>,
            common_assets::store::insert_as_resource::<DialogAssets>,
        ),
    );

    debug!("Adding map layout");

    common_top_down::layout::register::<DevPlayground, _>(
        app,
        GlobalGameState::Blank,
        GlobalGameState::InDevPlayground,
    );

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded.run_if(in_state(GlobalGameState::Blank)),
    );

    debug!("Adding visuals");

    app.add_systems(
        FixedUpdate,
        (
            common_visuals::systems::advance_atlas_animation,
            common_visuals::systems::smoothly_translate,
            common_visuals::systems::interpolate,
        )
            .run_if(in_state(GlobalGameState::InDevPlayground)),
    );

    debug!("Adding story");

    app.add_systems(
        OnEnter(GlobalGameState::Blank),
        common_story::spawn_camera,
    );
    app.add_systems(
        Update,
        common_story::portrait_dialog::change_selection
            .run_if(in_state(GlobalGameState::InDevPlayground))
            .run_if(in_portrait_dialog())
            .run_if(move_action_just_pressed()),
    );
    app.add_systems(
        Last,
        common_story::portrait_dialog::advance
            .run_if(in_state(GlobalGameState::InDevPlayground))
            .run_if(in_portrait_dialog())
            .run_if(interaction_just_pressed()),
    );
    app.add_systems(
        OnExit(GlobalGameState::InDevPlayground),
        common_story::despawn_camera,
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

impl AssetList for DevPlayground {}

impl std::fmt::Display for DevPlayground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", DevPlayground::name())
    }
}
