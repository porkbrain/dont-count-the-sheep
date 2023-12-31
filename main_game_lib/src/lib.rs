pub mod prelude;
pub mod vec2_ext;

use bevy::{app::AppExit, prelude::*, window::WindowTheme};
use bevy_pixel_camera::PixelCameraPlugin;

/// What's shown on screen.
pub const VISIBLE_WIDTH: f32 = 640.0;
/// What's shown on screen.
pub const VISIBLE_HEIGHT: f32 = 360.0;
/// One pixel is 3x3 pixels on screen.
pub const PIXEL_ZOOM: f32 = 3.0;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GlobalGameState {
    /// Dummy state so that we can do loading transitions.
    #[default]
    Blank,

    /// Sets up the apartment game in the background.
    ApartmentLoading,
    /// Player is at apartment.
    InApartment,
    /// Despawn apartment game resources.
    ApartmentQuitting,

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
    MeditationQuitting,

    /// Performs all necessary cleanup and exits the game.
    Exit,
}

/// What are the allowed transitions between game states?
#[derive(Debug)]
pub enum GlobalGameStateTransition {
    /// Restart the game
    MeditationQuittingToMeditationLoading,
    /// Exit back to the apartment
    MeditationQuittingToApartment,

    /// Play the meditation mini game
    ApartmentQuittingToMeditationLoading,
    /// Quit the game
    ApartmentQuittingToExit,
}

/// Certain states have multiple allowed transitions.
/// The tip of the stack must always match the current state.
#[derive(Resource, Debug, Default)]
pub struct GlobalGameStateTransitionStack {
    stack: Vec<GlobalGameStateTransition>,
}

pub fn windowed_app() -> App {
    let mut app = App::new();

    app.add_state::<GlobalGameState>();
    app.insert_resource(GlobalGameStateTransitionStack::default());

    app.add_plugins(
        DefaultPlugins
            .set(bevy::log::LogPlugin {
                level: bevy::log::Level::WARN,
                filter: "apartment=trace,meditation=trace,meditation::weather::sprite=debug"
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
        PixelCameraPlugin,
        bevy_magic_light_2d::Plugin,
        common_visuals::Plugin,
    ));

    app.add_systems(OnEnter(GlobalGameState::Exit), exit);

    app
}

fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit)
}

impl GlobalGameStateTransitionStack {
    pub fn push(&mut self, transition: GlobalGameStateTransition) {
        self.stack.push(transition);
    }

    pub fn pop_next_for(
        &mut self,
        state: GlobalGameState,
    ) -> Option<GlobalGameState> {
        use GlobalGameState::*;
        use GlobalGameStateTransition::*;

        match (self.stack.pop(), state) {
            (None, state) => {
                debug!("There's nowhere to transition from {state:?}");
                None
            }
            (
                Some(MeditationQuittingToMeditationLoading),
                MeditationQuitting,
            ) => Some(MeditationLoading),
            (Some(MeditationQuittingToApartment), MeditationQuitting) => {
                Some(ApartmentLoading)
            }
            (Some(ApartmentQuittingToExit), ApartmentQuitting) => Some(Exit),
            (Some(ApartmentQuittingToMeditationLoading), ApartmentQuitting) => {
                Some(MeditationLoading)
            }
            (Some(transition), state) => {
                error!(
                    "Next transition {transition:?} does not match {state:?}"
                );
                None
            }
        }
    }
}
