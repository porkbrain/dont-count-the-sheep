pub mod action;
pub mod prelude;
pub mod state;
pub mod vec2_ext;

pub use action::*;
use bevy::{app::AppExit, prelude::*, window::WindowTheme};
use bevy_inspector_egui::quick::{StateInspectorPlugin, WorldInspectorPlugin};
use bevy_pixel_camera::PixelCameraPlugin;
use common_visuals::PRIMARY_COLOR;
use leafwing_input_manager::{
    action_state::ActionState, plugin::InputManagerPlugin,
};
pub use state::*;

pub fn windowed_app() -> App {
    let mut app = App::new();

    app.add_state::<GlobalGameState>()
        .register_type::<GlobalGameState>()
        .insert_resource(ClearColor(PRIMARY_COLOR))
        .insert_resource(GlobalGameStateTransitionStack::default())
        .register_type::<GlobalGameStateTransitionStack>()
        .init_resource::<ActionState<GlobalAction>>()
        .register_type::<GlobalAction>()
        .insert_resource(GlobalAction::input_map());

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
        InputManagerPlugin::<GlobalAction>::default(),
        bevy_magic_light_2d::Plugin,
        common_visuals::Plugin,
        bevy_webp_anim::Plugin,
        common_loading_screen::Plugin,
        common_store::Plugin,
    ));

    app.add_systems(OnEnter(GlobalGameState::Exit), exit);

    app
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit)
}
