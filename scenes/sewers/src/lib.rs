#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

mod layout;
mod prelude;

use common_loading_screen::LoadingScreenState;
use prelude::*;

use crate::layout::LayoutEntity;

/// Important scene struct.
/// We use it as identifiable generic in common logic.
#[derive(TypePath, Default, Debug)]
pub struct Sewers;

impl TopDownScene for Sewers {
    fn name() -> &'static str {
        "sewers"
    }

    fn bounds() -> [i32; 4] {
        [-100, 100, -100, 100]
    }
}

impl WithStandardStateSemantics for Sewers {
    fn loading() -> GlobalGameState {
        GlobalGameState::LoadingSewers
    }

    fn running() -> GlobalGameState {
        GlobalGameState::AtSewers
    }

    fn quitting() -> GlobalGameState {
        GlobalGameState::QuittingSewers
    }
}

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum SewersAction {
    ExitScene,
}

pub fn add(app: &mut App) {
    info!("Adding {Sewers:?} to app");

    app.add_event::<SewersAction>();

    top_down::default_setup_for_scene::<Sewers>(app);

    #[cfg(feature = "devtools")]
    top_down::dev_default_setup_for_scene::<Sewers>(app);

    debug!("Adding plugins");

    app.add_plugins(layout::Plugin);

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(Sewers::in_loading_state())
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(Sewers::in_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(Sewers::in_running_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(Sewers::in_quitting_state()),
    );

    info!("Added {Sewers:?} to app");
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap<Sewers>>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(mut next_state: ResMut<NextState<GlobalGameState>>) {
    info!("Entering {Sewers:?}");
    next_state.set(Sewers::running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    info!("Leaving {Sewers:?}");

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    use GlobalGameStateTransition::*;
    match *transition {
        SewersToDowntown => {
            next_state.set(GlobalGameState::LoadingDowntown);
        }
        _ => {
            unreachable!("Invalid {Sewers:?} transition {transition:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_valid_tscn_scene() {
        const TSCN: &str =
            include_str!("../../../main_game/assets/scenes/sewers.tscn");
        rscn::parse(TSCN, &default());
    }
}
