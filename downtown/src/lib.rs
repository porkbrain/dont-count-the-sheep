#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod actor;
mod cameras;
mod consts;
mod layout;
mod prelude;
mod zindex;

use common_assets::{store::AssetList, AssetStore};
use common_loading_screen::LoadingScreenState;
use common_story::{portrait_dialog::in_portrait_dialog, DialogAssets};
use main_game_lib::{
    common_action::{interaction_just_pressed, move_action_just_pressed},
    GlobalGameStateTransitionStack,
};
use prelude::*;

/// Important scene struct.
/// We use it as identifiable generic in some common logic such as layout or
/// asset.
#[derive(TypePath)]
pub(crate) struct Downtown;

pub fn add(app: &mut App) {
    info!("Adding downtown to app");

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, layout::Plugin, actor::Plugin));

    debug!("Adding assets");

    app.add_systems(
        OnEnter(GlobalGameState::DowntownLoading),
        (
            common_assets::store::insert_as_resource::<Downtown>,
            common_assets::store::insert_as_resource::<DialogAssets>,
        ),
    );
    app.add_systems(
        OnExit(GlobalGameState::DowntownQuitting),
        (
            common_assets::store::remove_as_resource::<Downtown>,
            common_assets::store::remove_as_resource::<DialogAssets>,
        ),
    );

    debug!("Adding map layout");

    // TODO: https://github.com/bevyengine/bevy/pull/10153
    // common_layout::register::<Downtown, _>(
    //     app,
    //     GlobalGameState::DowntownLoading,
    //     #[cfg(feature = "dev")]
    //     GlobalGameState::AtDowntown,
    // );

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

    debug!("Adding visuals");

    app.add_systems(
        FixedUpdate,
        common_visuals::systems::advance_animation
            .run_if(in_state(GlobalGameState::AtDowntown)),
    );

    debug!("Adding story");

    app.add_systems(
        OnEnter(GlobalGameState::DowntownLoading),
        common_story::spawn_camera,
    );
    app.add_systems(
        Update,
        common_story::portrait_dialog::change_selection
            .run_if(in_state(GlobalGameState::AtDowntown))
            .run_if(in_portrait_dialog())
            .run_if(move_action_just_pressed()),
    );
    app.add_systems(
        Last,
        common_story::portrait_dialog::advance
            .run_if(in_state(GlobalGameState::AtDowntown))
            .run_if(in_portrait_dialog())
            .run_if(interaction_just_pressed()),
    );
    app.add_systems(
        OnEnter(GlobalGameState::DowntownQuitting),
        common_story::despawn_camera,
    );

    info!("Added downtown to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<common_top_down::Map<Downtown>>>,
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
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving downtown");

    // be a good guy and don't invade other game loops with our controls
    controls.consume_all();

    match stack.pop_next_for(GlobalGameState::DowntownQuitting) {
        // possible restart or change of game loop
        Some(next) => next_state.set(next),
        None => {
            unreachable!("There's nowhere to transition from DowntownQuitting");
        }
    }
}

impl AssetList for Downtown {
    fn folders() -> &'static [&'static str] {
        &[assets::FOLDER]
    }
}
