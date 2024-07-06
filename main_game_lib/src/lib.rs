#![doc = include_str!("../../README.md")]
#![feature(trivial_bounds)]
#![feature(let_chains)]
#![deny(missing_docs)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod cutscene;
pub mod dialog;
pub mod hud;
pub mod player_stats;
pub mod prelude;
pub mod rscn;
pub mod state;
pub mod top_down;
pub mod vec2_ext;

use bevy::{app::AppExit, prelude::*};
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
                level: bevy::log::Level::INFO,
                filter: "\
                info,\
                game=trace,\
                common_action=trace,\
                common_assets=trace,\
                common_loading_screen=trace,\
                common_physics=trace,\
                common_store=trace,\
                common_visuals=trace,\
                common_story=trace,\
                main_game_lib=trace,\
                main_game_lib::top_down=trace,\
                main_game_lib::top_down::actor::npc=debug,\
                main_game_lib::top_down::actor=debug,\
                main_game_lib::top_down::environmental_objects::door=debug,\
                main_game_lib::top_down::cameras=debug,\
                main_game_lib::top_down::layout=debug,\
                scene_building1_player_floor=trace,\
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
        // TODO: load from save file
        .init_resource::<player_stats::PlayerStats>()
        .insert_resource(ClearColor(PRIMARY_COLOR))
        .init_resource::<GlobalGameStateTransition>()
        .init_asset::<crate::rscn::TscnTree>()
        .init_asset_loader::<crate::rscn::TscnLoader>()
        .init_asset_loader::<common_assets::ignore_loader::Loader>();

    #[cfg(feature = "devtools")]
    {
        use bevy_inspector_egui::quick::{
            ResourceInspectorPlugin, StateInspectorPlugin, WorldInspectorPlugin,
        };

        app.register_type::<GlobalGameStateTransition>()
            .register_type::<GlobalGameState>()
            .register_type::<player_stats::PlayerStats>();

        app.add_plugins((
            bevy_egui::EguiPlugin,
            WorldInspectorPlugin::new(),
            StateInspectorPlugin::<GlobalGameState>::default(),
            ResourceInspectorPlugin::<player_stats::PlayerStats>::default(),
        ));
    }

    app.add_plugins((
        bevy_webp_anim::Plugin,
        common_action::Plugin,
        common_loading_screen::Plugin,
        common_store::Plugin,
        common_story::Plugin,
        crate::top_down::Plugin,
        common_visuals::Plugin,
        cutscene::Plugin,
        PixelCameraPlugin,
        crate::hud::Plugin,
        crate::dialog::Plugin,
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
