//! Game state management.

use crate::prelude::*;

/// Provides control for the game states.
/// Each scene can add whatever state it needs to this enum.
/// Transitions between states are controlled by the
/// [`GlobalGameStateTransition`].
/// It defines what transitions are allowed.
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect)]
pub enum GlobalGameState {
    /// Dummy state so that we can do loading transitions.
    #[default]
    Blank,

    /// When new game is started.
    /// Populates the save log with the default values.
    NewGame,

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

    /// Sets up the downtown scene in the background.
    DowntownLoading,
    /// Player is at downtown.
    AtDowntown,
    /// Despawn downtown game resources.
    DowntownQuitting,

    /// Performs all necessary cleanup and exits the game.
    Exit,
}

/// What are the allowed transitions between game states?
#[allow(missing_docs)]
#[derive(Resource, Debug, Default, Reflect, Clone, Copy, Eq, PartialEq)]
#[reflect(Resource)]
pub enum GlobalGameStateTransition {
    #[default]
    BlankToNewGame,
    NewGameToApartment,

    RestartMeditation,
    MeditationToApartment,

    ApartmentToMeditation,
    ApartmentToDowntown,

    DowntownToApartment,
}
