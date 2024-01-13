#![feature(trivial_bounds)]

pub mod action;
pub mod loading_screen;
pub mod prelude;
pub mod store;
pub mod vec2_ext;

pub use action::*;
use bevy::{app::AppExit, prelude::*, window::WindowTheme};
use bevy_inspector_egui::quick::{StateInspectorPlugin, WorldInspectorPlugin};
use bevy_pixel_camera::PixelCameraPlugin;
use leafwing_input_manager::{
    action_state::ActionState, plugin::InputManagerPlugin,
};
use prelude::PRIMARY_COLOR;
pub use store::*;

/// TODO: move to own mod
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
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
#[derive(Debug, Reflect, Clone, Eq, PartialEq)]
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
#[derive(Resource, Debug, Default, Reflect)]
pub struct GlobalGameStateTransitionStack {
    stack: Vec<GlobalGameStateTransition>,
}

pub fn windowed_app() -> App {
    let mut app = App::new();

    app.add_state::<GlobalGameState>()
        .register_type::<GlobalGameState>()
        .insert_resource(ClearColor(PRIMARY_COLOR))
        .init_resource::<GlobalStore>()
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
        loading_screen::Plugin,
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
