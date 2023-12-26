#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod assets;
mod background;
mod cameras;
mod climate;
mod consts;
mod distractions;
mod gravity;
mod path;
mod prelude;
mod ui;
mod weather;
mod zindex;

use cameras::BackgroundLightScene;
use common_physics::PoissonsEquation;
use gravity::Gravity;
use main_game_lib::GlobalGameStateTransitionStack;
use prelude::*;

pub fn add(app: &mut App) {
    app.add_plugins((
        ui::Plugin,
        climate::Plugin,
        distractions::Plugin,
        weather::Plugin,
        cameras::Plugin,
        background::Plugin,
    ));

    // TODO: compose these
    app.add_plugins((
        bevy_magic_light_2d::Plugin,
        bevy_webp_anim::Plugin,
        common_visuals::Plugin,
    ));

    app.add_systems(
        FixedUpdate,
        common_physics::systems::apply_velocity
            .run_if(in_state(GlobalGameState::MeditationInGame)),
    );

    app.add_systems(OnEnter(GlobalGameState::MeditationLoading), spawn);
    app.add_systems(OnEnter(GlobalGameState::MeditationQuitting), despawn);

    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::MeditationQuitting)),
    );

    common_physics::poissons_equation::register::<gravity::Gravity, _>(
        app,
        GlobalGameState::MeditationInGame,
    );

    #[cfg(feature = "dev")]
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

fn spawn(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    info!("Loading meditation game");

    debug!("Spawning resources ClearColor and PoissonsEquation<Gravity>");
    commands.insert_resource(ClearColor(background::COLOR));
    commands.insert_resource(gravity::field());

    next_state.set(GlobalGameState::MeditationInGame);
}

fn despawn(mut commands: Commands) {
    debug!("Despawning resources ClearColor and PoissonsEquation<Gravity>");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<PoissonsEquation<Gravity>>();
}

fn all_cleaned_up(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
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
