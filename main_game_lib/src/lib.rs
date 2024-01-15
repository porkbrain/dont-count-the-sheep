pub mod prelude;
pub mod state;
pub mod vec2_ext;

use bevy::{app::AppExit, prelude::*, window::WindowTheme};
use bevy_inspector_egui::quick::{StateInspectorPlugin, WorldInspectorPlugin};
use bevy_pixel_camera::PixelCameraPlugin;
pub use common_action;
use common_visuals::PRIMARY_COLOR;
pub use state::*;

pub fn windowed_app() -> App {
    let mut app = App::new();

    app.add_state::<GlobalGameState>()
        .register_type::<GlobalGameState>()
        .insert_resource(ClearColor(PRIMARY_COLOR))
        .insert_resource(GlobalGameStateTransitionStack::default())
        .register_type::<GlobalGameStateTransitionStack>();

    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                filter: "\
                main_game_lib=trace,\
                apartment=trace,\
                common_story=trace,\
                meditation=trace,\
                meditation::hoshi::sprite=debug\
                "
                .to_string(),
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ciesin".into(),
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    mode: bevy::window::WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
    );

    // dev only
    app.add_plugins((
        WorldInspectorPlugin::new(),
        StateInspectorPlugin::<GlobalGameState>::default(),
    ));

    app.add_plugins((
        PixelCameraPlugin,
        bevy_magic_light_2d::Plugin,
        common_visuals::Plugin,
        bevy_webp_anim::Plugin,
        common_loading_screen::Plugin,
        common_store::Plugin,
        common_action::Plugin,
    ));

    app.add_systems(OnEnter(GlobalGameState::Exit), exit);

    app
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit)
}
