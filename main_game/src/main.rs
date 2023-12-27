use bevy::prelude::*;
use main_game_lib::GlobalGameState;

fn main() {
    let mut app = main_game_lib::windowed_app();

    fn start(mut next_state: ResMut<NextState<GlobalGameState>>) {
        next_state.set(GlobalGameState::ApartmentLoading);
    }
    app.add_systems(Update, start.run_if(in_state(GlobalGameState::Blank)));

    apartment::add(&mut app);
    meditation::add(&mut app);

    app.run();
}
