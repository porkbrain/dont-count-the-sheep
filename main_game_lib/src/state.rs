//! Game state management.

use crate::prelude::*;

/// Provides control for the game states.
/// Each scene can add whatever state it needs to this enum.
/// Transitions between states are controlled by the
/// [`GlobalGameStateTransition`].
/// It defines what transitions are allowed.
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect)]
#[allow(missing_docs)]
pub enum GlobalGameState {
    /// Dummy state so that we can do loading transitions.
    #[default]
    Blank,

    /// When new game is started.
    /// Populates the save log with the default values.
    NewGame,

    /// Sets up the floor with player's first apartment
    LoadingBuilding1PlayerFloor,
    AtBuilding1PlayerFloor,
    QuittingBuilding1PlayerFloor,

    /// Change the game state to this state to run systems that setup the
    /// meditation game in the background.
    /// Nothing is shown to the player yet.
    LoadingMeditation,
    /// Game is being played.
    InGameMeditation,
    /// Game is paused and menu is spawned.
    /// Menu is always spawned and destroyed, unlike the game resources.
    MeditationInMenu,
    /// Change the game state to this state to run systems that clean up the
    /// meditation game in the background.
    QuittingMeditation,

    LoadingDowntown,
    AtDowntown,
    QuittingDowntown,

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
    NewGameToBuilding1PlayerFloor,

    RestartMeditation,
    MeditationToBuilding1PlayerFloor,

    Building1PlayerFloorToMeditation,
    Building1PlayerFloorToDowntown,
    Building1PlayerFloorToBuilding1Basement1,

    DowntownToBuilding1PlayerFloor,
}
