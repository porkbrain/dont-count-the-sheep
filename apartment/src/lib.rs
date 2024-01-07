#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod cameras;
mod consts;
mod controllable;
mod layout;
mod prelude;
mod zindex;

use bevy::utils::Instant;
use consts::START_LOADING_SCREEN_AFTER;
use leafwing_input_manager::action_state::ActionState;
use main_game_lib::{
    loading_screen::{self, LoadingScreenSettings, LoadingScreenState},
    GlobalAction, GlobalGameStateTransition, GlobalGameStateTransitionStack,
};
use prelude::*;

use crate::layout::Apartment;

pub fn add(app: &mut App) {
    info!("Adding apartment to app");

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, layout::Plugin, controllable::Plugin));

    debug!("Adding map layout");

    common_layout::register::<Apartment, _>(
        app,
        GlobalGameState::ApartmentLoading,
        #[cfg(feature = "dev")]
        GlobalGameState::InApartment,
    );

    debug!("Adding game loop");

    app.add_systems(
        Last,
        all_loaded.run_if(in_state(GlobalGameState::ApartmentLoading)),
    );

    app.add_systems(
        Update,
        close_game.run_if(in_state(GlobalGameState::InApartment)),
    );

    app.add_systems(
        Update,
        smooth_exit.run_if(in_state(GlobalGameState::ApartmentQuitting)),
    );

    info!("Added apartment to app");
}

/// TODO: Temp. solution: press ESC to quit.
fn close_game(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    keyboard: ResMut<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        stack.push(GlobalGameStateTransition::ApartmentQuittingToExit);
        next_state.set(GlobalGameState::ApartmentQuitting);
    }
}

fn all_loaded(
    map: Option<Res<common_layout::Map<Apartment>>>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    if map.is_none() {
        return;
    }

    info!("Entering apartment");

    next_state.set(GlobalGameState::InApartment);
}

struct ExitAnimation {
    since: Instant,
    loading_screen_started: bool,
}

// TODO: this can be done easier in new version of bevy where delay timers
// exist
fn smooth_exit(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
    mut next_loading_screen_state: ResMut<NextState<LoadingScreenState>>,
    settings: Res<LoadingScreenSettings>,

    mut local: Local<Option<ExitAnimation>>,
) {
    // this is reset to None when we're done with the exit animation
    let ExitAnimation {
        since,
        loading_screen_started,
    } = local.get_or_insert_with(|| ExitAnimation {
        since: Instant::now(),
        loading_screen_started: false,
    });

    if !*loading_screen_started && since.elapsed() > START_LOADING_SCREEN_AFTER
    {
        debug!("Transitioning to first loading screen state");
        next_loading_screen_state.set(loading_screen::start_state());
        *loading_screen_started = true;
    }

    if since.elapsed()
        > START_LOADING_SCREEN_AFTER + settings.fade_loading_screen_in * 2
    {
        info!("Leaving apartment");

        // reset local state for next time
        *local = None;

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
