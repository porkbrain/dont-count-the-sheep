//! See the dev playground scene

use bevy::render::{camera::ClearColor, color::Color};

fn main() {
    let mut app = main_game_lib::windowed_app();
    app.insert_resource(ClearColor(Color::WHITE));

    scene_dev_playground::add(&mut app);

    app.run();
}
