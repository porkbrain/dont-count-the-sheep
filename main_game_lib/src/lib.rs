#![doc = include_str!("../../README.md")]
#![feature(trivial_bounds)]
#![deny(missing_docs)]

pub mod cutscene;
pub mod prelude;
pub mod state;
pub mod vec2_ext;

use bevy::{app::AppExit, prelude::*};
use bevy_inspector_egui::quick::{StateInspectorPlugin, WorldInspectorPlugin};
use bevy_pixel_camera::PixelCameraPlugin;
pub use common_ext;

use crate::prelude::*;

/// Constructs a new app with all the necessary plugins and systems.
///
/// Main game bin then adds scenes and runs it.
pub fn windowed_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                filter: "\
                warn,\
                apartment=trace,\
                bevy_magic_light_2d=trace,\
                common_action=trace,\
                common_assets=trace,\
                common_loading_screen=trace,\
                common_physics=trace,\
                common_store=trace,\
                common_story=trace,\
                common_top_down=trace,\
                common_top_down::actor::npc=debug,\
                common_top_down::actor=debug,\
                common_top_down::environmental_objects::door=debug,\
                common_top_down::cameras=debug,\
                common_top_down::layout=debug,\
                common_visuals=trace,\
                dev_playground=trace,\
                downtown=trace,\
                game=trace,\
                main_game_lib=trace,\
                meditation=trace,\
                meditation::hoshi::sprite=debug,\
                "
                .to_string(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some({
                    let mut w = Window {
                        title: "Don't Count The Sheep".into(),
                        ..default()
                    };

                    w.set_maximized(true);
                    w
                }),
                ..default()
            }),
    );

    info!("Initializing Don't Count The Sheep");

    app.init_state::<GlobalGameState>()
        .register_type::<GlobalGameState>()
        .insert_resource(ClearColor(PRIMARY_COLOR))
        .insert_resource(GlobalGameStateTransitionStack::default())
        .register_type::<GlobalGameStateTransitionStack>();

    // TODO: dev only
    app.add_plugins((
        bevy_egui::EguiPlugin,
        WorldInspectorPlugin::new(),
        StateInspectorPlugin::<GlobalGameState>::default(),
    ));
    fn configure_visuals_system(mut contexts: bevy_egui::EguiContexts) {
        contexts.ctx_mut().set_visuals(bevy_egui::egui::Visuals {
            window_rounding: 0.0.into(),
            ..Default::default()
        });
    }
    app.add_systems(Startup, configure_visuals_system);

    app.add_plugins((
        bevy_magic_light_2d::Plugin,
        bevy_webp_anim::Plugin,
        common_action::Plugin,
        common_loading_screen::Plugin,
        common_store::Plugin,
        common_top_down::Plugin,
        common_visuals::Plugin,
        cutscene::Plugin,
        PixelCameraPlugin,
    ));

    info!("Plugins added");

    app.add_systems(
        Startup,
        (
            begin_loading_static_assets_on_startup,
            common_assets::store::insert_as_resource::<
                common_assets::store::StaticScene,
            >,
        ),
    );
    app.add_systems(OnEnter(GlobalGameState::Exit), exit);

    app
}

/// All assets that should be kept in memory throughout the game.
fn begin_loading_static_assets_on_startup(
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    common_story::Character::load_all_sprite_atlas_layouts(
        &mut texture_atlases,
    );
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}
