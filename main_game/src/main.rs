use bevy::prelude::*;
use main_game_lib::{
    common_loading_screen::{LoadingScreenSettings, LoadingScreenState},
    GlobalGameState,
};

fn main() {
    let mut app = main_game_lib::windowed_app();

    // we didn't finish yet the main menu, so meanwhile start wherever
    fn start(
        mut cmd: Commands,
        mut next_state: ResMut<NextState<GlobalGameState>>,
        mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    ) {
        // just a quick loading screen, no bg
        cmd.insert_resource(LoadingScreenSettings {
            bg_image_asset: None,
            ..default()
        });
        next_loading_state
            .set(main_game_lib::common_loading_screen::start_state());

        next_state.set(GlobalGameState::DowntownLoading);
    }
    app.add_systems(Update, start.run_if(in_state(GlobalGameState::Blank)));

    apartment::add(&mut app);
    meditation::add(&mut app);
    downtown::add(&mut app);

    app.run();
}
