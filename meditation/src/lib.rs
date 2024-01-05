#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod assets;
mod background;
mod cameras;
mod climate;
mod consts;
mod distractions;
mod gravity;
mod hoshi;
mod path;
mod prelude;
mod ui;
mod zindex;

use bevy_webp_anim::WebpAnimator;
use common_physics::PoissonsEquation;
use gravity::Gravity;
use main_game_lib::GlobalGameStateTransitionStack;
use prelude::*;

pub fn add(app: &mut App) {
    info!("Adding meditation to app");

    debug!("Adding plugins");

    app.add_plugins((
        ui::Plugin,
        climate::Plugin,
        distractions::Plugin,
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
        ((
            common_visuals::systems::begin_animation_at_random,
            common_visuals::systems::flicker,
            bevy_webp_anim::systems::start_loaded_videos::<()>,
            bevy_webp_anim::systems::load_next_frame,
        )
            .run_if(in_state(GlobalGameState::MeditationInGame)),),
    );
    app.add_systems(
        Update,
        (common_visuals::systems::flicker
            .run_if(in_state(GlobalGameState::MeditationInMenu)),),
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

    app.add_systems(OnEnter(GlobalGameState::MeditationLoading), spawn);
    app.add_systems(
        Last,
        all_loaded.run_if(in_state(GlobalGameState::MeditationLoading)),
    );

    app.add_systems(OnEnter(GlobalGameState::MeditationQuitting), despawn);
    app.add_systems(
        Last,
        all_cleaned_up.run_if(in_state(GlobalGameState::MeditationQuitting)),
    );

    #[cfg(feature = "dev")]
    debug!("Adding dev");

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

    info!("Added meditation to app");
}

fn spawn(mut commands: Commands) {
    debug!("Spawning resources");
    commands.insert_resource(ClearColor(background::COLOR));
    commands.insert_resource(gravity::field());
    commands.init_resource::<WebpAnimator>();
}

fn despawn(mut commands: Commands) {
    debug!("Despawning resources");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<PoissonsEquation<Gravity>>();
    commands.remove_resource::<WebpAnimator>();
}

fn all_loaded(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering meditation game");

    next_state.set(GlobalGameState::MeditationInGame);
}

fn all_cleaned_up(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    info!("Leaving meditation game");

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
