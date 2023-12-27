#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod assets;
mod bedroom;
mod cameras;
mod consts;
mod prelude;
mod zindex;

use main_game_lib::{
    GlobalGameStateTransition, GlobalGameStateTransitionStack,
};
use prelude::*;

pub fn add(app: &mut App) {
    info!("Adding apartment to app");

    debug!("Adding plugins");

    app.add_plugins((cameras::Plugin, bedroom::Plugin));

    debug!("Adding game loop");

    app.add_systems(OnEnter(GlobalGameState::ApartmentLoading), spawn);
    app.add_systems(
        Last,
        all_loaded.run_if(in_state(GlobalGameState::ApartmentLoading)),
    );

    app.add_systems(
        Update,
        (close_game, open_meditation)
            .run_if(in_state(GlobalGameState::InApartment)),
    );

    app.add_systems(OnEnter(GlobalGameState::ApartmentQuitting), despawn);
    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::ApartmentQuitting)),
    );

    info!("Added apartment to app");
}

/// Temp. solution: press ESC to quit.
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

/// Temp. solution: press M to open meditation.
fn open_meditation(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    keyboard: ResMut<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::M) {
        stack.push(
            GlobalGameStateTransition::ApartmentQuittingToMeditationLoading,
        );
        next_state.set(GlobalGameState::ApartmentQuitting);
    }
}

fn spawn(mut commands: Commands) {
    debug!("Spawning resources ClearColor");

    commands.insert_resource(ClearColor(PRIMARY_COLOR));
}

fn despawn(mut commands: Commands) {
    debug!("Despawning resources ClearColor");

    commands.remove_resource::<ClearColor>();
}

fn all_loaded(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering apartment");

    next_state.set(GlobalGameState::InApartment);
}

fn all_cleaned_up(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    info!("Leaving apartment");

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
