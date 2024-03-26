mod new_game;

use bevy::prelude::*;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use main_game_lib::prelude::*;

fn main() {
    let mut app = main_game_lib::windowed_app();
    info!("Windowed app from main_game_lib created");

    // we didn't finish yet the main menu, so meanwhile start wherever
    fn start(
        mut cmd: Commands,
        mut next_state: ResMut<NextState<GlobalGameState>>,
        mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    ) {
        // Bevy from 0.13 requires that there's always a camera spawned.
        // This is needlessly too much effort for the design I picked where each
        // scene is taking care of its own camera.
        // For now the fix is simple: spawn an inactive camera to avoid panic.
        // The relevant function [`UiSurface::update_children`] emits:
        //
        // > Unstyled child in a UI entity hierarchy. You are using an entity
        // > without UI components as a child of an entity with UI components,
        // > results may be unexpected.
        cmd.spawn(Name::new("Inactive camera (see github issue #55)"))
            .insert(Camera2dBundle {
                camera: Camera {
                    is_active: false,
                    ..default()
                },
                ..default()
            });

        // just a quick loading screen, no bg
        cmd.insert_resource(LoadingScreenSettings {
            fade_loading_screen_in: from_millis(50),
            fade_loading_screen_out: from_millis(500),
            atlas: None,
            ..default()
        });
        next_loading_state.set(common_loading_screen::start_state());

        next_state.set(GlobalGameState::NewGame);
    }
    app.add_systems(Update, start.run_if(in_state(GlobalGameState::Blank)));
    app.add_systems(OnEnter(GlobalGameState::NewGame), new_game::on_enter);

    info!("Adding scenes");

    scene_building1_player_floor::add(&mut app);
    scene_building1_basement1::add(&mut app);
    scene_meditation::add(&mut app);
    scene_downtown::add(&mut app);

    info!("Starting Don't Count The Sheep");
    app.run();
}
