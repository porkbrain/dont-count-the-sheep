//! See the dev playground scene

use bevy::{core_pipeline::clear_color::ClearColor, render::color::Color};

fn main() {
    let mut app = main_game_lib::windowed_app();
    app.insert_resource(ClearColor(Color::WHITE));

    scene_dev_playground::add(&mut app);

    app.run();
}
