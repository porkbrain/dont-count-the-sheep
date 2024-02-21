use bevy::prelude::*;
use main_game_lib::{
    common_loading_screen::{LoadingScreenSettings, LoadingScreenState},
    prelude::from_millis,
    GlobalGameState,
};

fn main() {
    let mut app = main_game_lib::windowed_app();
    info!("Windowed app from main_game_lib created");

    // we didn't finish yet the main menu, so meanwhile start wherever
    fn start(
        mut cmd: Commands,
        mut next_state: ResMut<NextState<GlobalGameState>>,
        mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    ) {
        cmd.spawn(Camera2dBundle::default());

        // just a quick loading screen, no bg
        cmd.insert_resource(LoadingScreenSettings {
            fade_loading_screen_in: from_millis(50),
            fade_loading_screen_out: from_millis(500),
            bg_image_asset: None,
            ..default()
        });
        next_loading_state
            .set(main_game_lib::common_loading_screen::start_state());

        next_state.set(GlobalGameState::ApartmentLoading);
    }
    app.add_systems(Update, start.run_if(in_state(GlobalGameState::Blank)));

    info!("Adding scenes");

    scene_apartment::add(&mut app);
    scene_meditation::add(&mut app);
    scene_downtown::add(&mut app);

    info!("Starting Don't Count The Sheep");
    app.run();
}
