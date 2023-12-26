//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and move+space to activate
//! the special. The sprite should feel floaty as if you were playing Puff in
//! Smashbros.

#![allow(clippy::assertions_on_constants)]
#![allow(clippy::type_complexity)]

mod background;
mod cameras;
mod climate;
mod distractions;
mod gravity;
mod path;
mod prelude;
mod ui;
mod weather;
mod zindex;

mod consts {
    /// What's shown on screen.
    pub(crate) const VISIBLE_WIDTH: f32 = 640.0;
    /// What's shown on screen.
    pub(crate) const VISIBLE_HEIGHT: f32 = 360.0;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_WIDTH: f32 = VISIBLE_WIDTH * 1.25;

    /// The stage is bigger than what's shown on screen.
    pub(crate) const GRAVITY_STAGE_HEIGHT: f32 = VISIBLE_HEIGHT * 1.25;
}

use bevy::window::WindowTheme;
use bevy_pixel_camera::PixelCameraPlugin;
use cameras::BackgroundLightScene;
use common_physics::PoissonsEquation;
use gravity::Gravity;
use prelude::*;

/// TODO: This will eventually be exported to the main game crate as this
/// workspace member becomes a library.
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum GlobalGameState {
    /// Dummy state so that we can do loading transitions.
    #[default]
    Blank,
    /// Change the game state to this state to run systems that setup the
    /// meditation game in the background.
    /// Nothing is shown to the player yet.
    MeditationLoading,
    /// Game is being played.
    MeditationInGame,
    /// Game is paused and menu is spawned.
    /// Menu is always spawned and destroyed, unlike the game resources.
    MeditationInMenu,
    /// Change the game state to this state to run systems that clean up the
    /// meditation game in the background.
    #[allow(dead_code)]
    MeditationQuitting,
}

fn main() {
    let mut app = App::new();

    // This will eventually be called outside of this crate.
    app.add_state::<GlobalGameState>();
    fn start(mut next_state: ResMut<NextState<GlobalGameState>>) {
        next_state.set(GlobalGameState::MeditationLoading);
    }
    app.add_systems(Update, start.run_if(in_state(GlobalGameState::Blank)));

    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                filter: "meditation=trace,meditation::weather::sprite=debug"
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
    app.add_plugins((
        bevy_magic_light_2d::Plugin,
        PixelCameraPlugin,
        bevy_webp_anim::Plugin,
        common_visuals::Plugin,
    ));

    app.add_systems(
        FixedUpdate,
        common_physics::systems::apply_velocity
            .run_if(in_state(GlobalGameState::MeditationInGame)),
    );
    app.add_plugins((
        ui::Plugin,
        climate::Plugin,
        distractions::Plugin,
        weather::Plugin,
        cameras::Plugin,
        background::Plugin,
    ))
    .add_systems(OnEnter(GlobalGameState::MeditationLoading), spawn)
    .add_systems(OnEnter(GlobalGameState::MeditationQuitting), despawn);

    common_physics::poissons_equation::register::<gravity::Gravity, _>(
        &mut app,
        GlobalGameState::MeditationInGame,
    );

    #[cfg(feature = "dev")]
    app.add_systems(
        Last,
        path::visualize.run_if(in_state(GlobalGameState::MeditationInGame)),
    );

    #[cfg(feature = "dev-poissons")]
    common_physics::poissons_equation::register_visualization::<
        gravity::Gravity,
        gravity::ChangeOfBasis,
        gravity::ChangeOfBasis,
        _,
    >(&mut app, GlobalGameState::MeditationInGame);

    // TODO: move
    app.run();
}

fn spawn(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GlobalGameState>>,
) {
    info!("Loading meditation game");

    debug!("Spawning resources ClearColor and PoissonsEquation<Gravity>");
    commands.insert_resource(ClearColor(background::COLOR));
    commands.insert_resource(gravity::field());

    next_state.set(GlobalGameState::MeditationInGame);
}

fn despawn(mut commands: Commands) {
    debug!("Despawning resources ClearColor and PoissonsEquation<Gravity>");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<PoissonsEquation<Gravity>>();
}
