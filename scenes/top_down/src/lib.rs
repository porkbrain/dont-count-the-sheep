#![doc = include_str!("../README.md")]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]
#![feature(trivial_bounds)]
#![feature(let_chains)]

mod building1_basement1;
mod layout;
mod prelude;

use common_loading_screen::LoadingScreenState;
use prelude::*;

use crate::layout::LayoutEntity;

pub fn add(app: &mut App) {
    info!("Adding top down scenes to app");

    debug!("Adding plugins");

    app.add_plugins(building1_basement1::Plugin);

    debug!("Adding game loop");

    // when everything is loaded, finish the loading process by transitioning
    // to the next loading state
    app.add_systems(
        Last,
        finish_when_everything_loaded
            .run_if(in_top_down_loading_state())
            .run_if(|q: Query<(), With<LayoutEntity>>| !q.is_empty())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );
    // ready to enter the game when the loading screen is completely gone
    app.add_systems(
        OnEnter(LoadingScreenState::DespawnLoadingScreen),
        enter_the_scene.run_if(in_top_down_loading_state()),
    );

    app.add_systems(
        Update,
        common_loading_screen::finish
            .run_if(in_top_down_running_state())
            .run_if(in_state(LoadingScreenState::WaitForSignalToFinish)),
    );

    app.add_systems(
        Update,
        // wait for the loading screen to fade in before changing state,
        // otherwise the player might see a flicker
        exit.run_if(in_state(common_loading_screen::wait_state()))
            .run_if(in_top_down_leaving_state()),
    );
}

fn finish_when_everything_loaded(
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    map: Option<Res<top_down::TileMap>>,
) {
    if map.is_none() {
        return;
    }

    debug!("All assets loaded");

    next_loading_state.set(common_loading_screen::finish_state());
}

fn enter_the_scene(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    scene: Res<State<WhichTopDownScene>>,
) {
    info!("Entering {}", **scene);
    next_state.set(scene.running());
}

fn exit(
    transition: Res<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
    scene: Res<State<WhichTopDownScene>>,
) {
    info!("Leaving {}", **scene);

    // be a good guy and don't invade other game loops with "Enter"
    controls.consume(&GlobalAction::Interact);

    use GlobalGameStateTransition::*;
    match *transition {
        Building1Basement1ToPlayerFloor => {
            next_state.set(WhichTopDownScene::Building1PlayerFloor.loading());
        }
        Building1Basement1ToDowntown => {
            next_state.set(WhichTopDownScene::Downtown.loading());
        }
        Building1Basement1ToBasement2 => {
            next_state.set(WhichTopDownScene::Building1Basement2.loading());
        }
        _ => {
            unreachable!("Invalid {} transition {transition:?}", **scene);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_valid_tscn_scenes() {
        for entry in std::fs::read_dir("../../main_game/assets/scenes")
            .expect("Cannot find scene assets")
        {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let path = path.to_str().unwrap();
                if path.ends_with(".tscn") {
                    let tscn = std::fs::read_to_string(path).unwrap();
                    println!("Parsing {path:?}");
                    rscn::parse(&tscn, &default());
                }
            }
        }
    }
}