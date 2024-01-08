#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod background;
mod cameras;
mod climate;
mod consts;
mod gravity;
mod hoshi;
mod path;
mod polpos;
mod prelude;
mod ui;
mod zindex;

use bevy::utils::Instant;
use bevy_webp_anim::WebpAnimator;
use common_physics::PoissonsEquation;
use gravity::Gravity;
use main_game_lib::{
    loading_screen::{self, LoadingScreenSettings, LoadingScreenState},
    GlobalGameStateTransitionStack,
};
use prelude::*;

pub fn add(app: &mut App) {
    info!("Adding meditation to app");

    debug!("Adding plugins");

    app.add_plugins((
        ui::Plugin,
        climate::Plugin,
        polpos::Plugin,
        hoshi::Plugin,
        cameras::Plugin,
        background::Plugin,
    ));

    debug!("Adding visuals");

    app.add_systems(
        FixedUpdate,
        common_visuals::systems::advance_animation
            .run_if(in_state(GlobalGameState::MeditationInGame)),
    );
    app.add_systems(
        Update,
        (
            common_visuals::systems::begin_animation_at_random,
            common_visuals::systems::flicker,
            bevy_webp_anim::systems::start_loaded_videos::<()>,
            bevy_webp_anim::systems::load_next_frame,
        )
            .run_if(in_state(GlobalGameState::MeditationInGame)),
    );
    app.add_systems(
        Update,
        common_visuals::systems::flicker
            .run_if(in_state(GlobalGameState::MeditationInMenu)),
    );

    debug!("Adding physics");

    app.add_systems(
        FixedUpdate,
        common_physics::systems::apply_velocity
            .run_if(in_state(GlobalGameState::MeditationInGame)),
    );
    common_physics::poissons_equation::register::<gravity::Gravity, _>(
        app,
        GlobalGameState::MeditationInGame,
    );

    debug!("Adding game loop");

    // 1. start the spawning process (the loading screen is already started)
    app.add_systems(OnEnter(GlobalGameState::MeditationLoading), spawn);
    // 2. when everything is loaded, finish the loading process by transitioning
    //    to the next loading state (this will also spawn the camera)
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(in_state(GlobalGameState::MeditationLoading))
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // 3. ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_game.run_if(in_state(GlobalGameState::MeditationLoading)),
    );

    app.add_systems(OnExit(GlobalGameState::MeditationQuitting), despawn);
    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::MeditationQuitting)),
    );

    #[cfg(feature = "dev")]
    {
        debug!("Adding dev");

        app.add_systems(
            Last,
            path::visualize.run_if(in_state(GlobalGameState::MeditationInGame)),
        );

        #[cfg(feature = "dev-poissons")]
        common_physics::poissons_equation::register_visualization::<
            gravity::Gravity,
            gravity::ChangeOfBasis,
            gravity::ChangeOfBasis,
            _,
        >(app, GlobalGameState::MeditationInGame);
    }

    info!("Added meditation to app");
}

fn spawn(mut commands: Commands) {
    debug!("Spawning resources");

    commands.insert_resource(gravity::field());
    commands.init_resource::<WebpAnimator>();
}

fn despawn(mut commands: Commands) {
    debug!("Despawning resources");

    commands.remove_resource::<PoissonsEquation<Gravity>>();
    commands.remove_resource::<WebpAnimator>();
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    asset_server: Res<AssetServer>,

    images: Query<&Handle<Image>>,
) {
    let all_images_loaded = images.iter().all(|image| {
        image.is_weak() || asset_server.is_loaded_with_dependencies(image)
    });

    if !all_images_loaded {
        return;
    }

    debug!("All images loaded");

    next_loading_state.set(loading_screen::finish_state());
}

fn enter_the_game(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering meditation game");
    next_state.set(GlobalGameState::MeditationInGame);
}

fn all_cleaned_up(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
    settings: Res<LoadingScreenSettings>,

    mut since: Local<Option<Instant>>,
) {
    // this is reset to None when we're done with the exit animation
    let elapsed = since.get_or_insert_with(|| Instant::now()).elapsed();
    if elapsed < settings.fade_loading_screen_in {
        return;
    }

    info!("Leaving meditation game");

    // reset local state for next time
    *since = None;

    // be a good guy and don't invade other game loops with our controls
    controls.consume_all();

    match stack.pop_next_for(GlobalGameState::MeditationQuitting) {
        // possible restart or change of game loop
        Some(next) => next_state.set(next),
        None => {
            unreachable!(
                "There's nowhere to transition from MeditationQuitting"
            );
        }
    }
}
