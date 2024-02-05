#![feature(trivial_bounds)]
// #![deny(missing_docs)]

pub mod cutscene;
pub mod prelude;
pub mod state;
pub mod vec2_ext;

use bevy::{app::AppExit, prelude::*};
use bevy_inspector_egui::quick::{StateInspectorPlugin, WorldInspectorPlugin};
use bevy_pixel_camera::PixelCameraPlugin;
pub use common_action;
pub use common_assets;
pub use common_ext;
pub use common_loading_screen;
pub use common_store;
pub use common_story;
pub use common_top_down;
pub use common_visuals::{self, PRIMARY_COLOR};
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
                warn,\
                apartment=trace,\
                common_action=trace,\
                common_assets=trace,\
                common_top_down=trace,\
                common_top_down::actor::npc=debug,\
                common_loading_screen=trace,\
                common_physics=trace,\
                common_store=trace,\
                common_story=trace,\
                common_visuals=trace,\
                downtown=trace,\
                main_game_lib=trace,\
                meditation=trace,\
                meditation::hoshi::sprite=debug,\
                "
                .to_string(),
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ciesin".into(),
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
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    common_story::Character::load_all_sprite_atlases(
        &asset_server,
        &mut texture_atlases,
    );
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit)
}
