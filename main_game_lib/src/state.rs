//! Game state management.

use bevy::ecs::system::SystemParam;
use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};

use crate::prelude::*;

/// Provides control for the game states.
/// Each scene can add whatever state it needs to this enum.
/// Transitions between states are controlled by the
/// [`GlobalGameStateTransition`].
/// It defines what transitions are allowed.
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
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

    LoadingBuilding1Basement1,
    AtBuilding1Basement1,
    QuittingBuilding1Basement1,

    LoadingBuilding1Basement2,
    AtBuilding1Basement2,
    QuittingBuilding1Basement2,

    LoadingClinic,
    AtClinic,
    QuittingClinic,

    LoadingClinicWard,
    AtClinicWard,
    QuittingClinicWard,

    LoadingPlantShop,
    AtPlantShop,
    QuittingPlantShop,

    LoadingSewers,
    AtSewers,
    QuittingSewers,

    LoadingTwinpeaksApartment,
    AtTwinpeaksApartment,
    QuittingTwinpeaksApartment,

    LoadingMall,
    AtMall,
    QuittingMall,

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

    LoadingCompound,
    AtCompound,
    QuittingCompound,

    LoadingCompoundTower,
    AtCompoundTower,
    QuittingCompoundTower,

    /// Performs all necessary cleanup and exits the game.
    Exit,
}

/// What are the allowed transitions between game states?
#[allow(missing_docs)]
#[derive(
    Resource,
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    strum::EnumString,
    strum::Display,
)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[cfg_attr(feature = "devtools", reflect(Resource))]
pub enum GlobalGameStateTransition {
    #[default]
    BlankToNewGame,
    NewGameToBuilding1PlayerFloor,

    RestartMeditation,
    MeditationToBuilding1PlayerFloor,

    Building1PlayerFloorToMeditation,
    Building1PlayerFloorToBuilding1Basement1,
    Sleeping,
    DowntownToBuilding1PlayerFloor,
    Building1PlayerFloorToDowntown,

    Building1Basement1ToPlayerFloor,
    Building1Basement1ToDowntown,
    Building1Basement1ToBasement2,
    Building1Basement2ToBasement1,

    DowntownToMall,
    MallToDowntown,

    DowntownToCompound,
    CompoundToDowntown,

    CompoundToTower,
    TowerToCompound,

    TwinpeaksApartmentToDowntown,
    DowntownToTwinpeaksApartment,

    PlantShopToDowntown,
    DowntownToPlantShop,

    SewersToDowntown,
    DowntownToSewers,

    ClinicToDowntown,
    DowntownToClinic,
    ClinicWardToDowntown,
    DowntownToClinicWard,
}

/// Typical scene has several states with standard semantics.
pub struct StandardStateSemantics {
    /// The state when the scene is loading.
    /// Setups up resources.
    pub loading: GlobalGameState,
    /// The state when the scene is running.
    pub running: GlobalGameState,
    /// The state when the scene is quitting.
    /// Cleans up resources.
    pub quitting: GlobalGameState,
    /// Some scenes have a paused state.
    pub paused: Option<GlobalGameState>,
}

/// Helper params that are used in transitions.
/// Use [`TransitionParams::begin`] to start a transition.
#[derive(SystemParam)]
pub struct TransitionParams<'w, 's> {
    /// Commands to insert loading screen settings.
    pub cmd: Commands<'w, 's>,
    /// Sets the transition that should happen.
    pub transition: ResMut<'w, GlobalGameStateTransition>,
    /// The next state is derived from the transition.
    pub next_state: ResMut<'w, NextState<GlobalGameState>>,
    /// Always set to start state.
    pub next_loading_screen_state: ResMut<'w, NextState<LoadingScreenState>>,
}

/// Typical scene has several states with standard semantics.
pub trait WithStandardStateSemantics {
    /// The state when the scene is loading.
    fn loading() -> GlobalGameState;
    /// The state when the scene is running.
    fn running() -> GlobalGameState;
    /// The state when the scene is quitting.
    fn quitting() -> GlobalGameState;

    /// Some scenes have a paused state.
    fn paused() -> Option<GlobalGameState> {
        None
    }

    /// Converts these methods into a struct
    fn semantics() -> StandardStateSemantics {
        StandardStateSemantics {
            loading: Self::loading(),
            running: Self::running(),
            quitting: Self::quitting(),
            paused: Self::paused(),
        }
    }

    /// Helper to check if the state is in the loading state.
    fn in_loading_state(
    ) -> impl FnMut(Option<Res<State<GlobalGameState>>>) -> bool + Clone {
        in_state(Self::loading())
    }

    /// Helper to check if the state is in the running state.
    fn in_running_state(
    ) -> impl FnMut(Option<Res<State<GlobalGameState>>>) -> bool + Clone {
        in_state(Self::running())
    }

    /// Helper to check if the state is in the quitting state.
    fn in_quitting_state(
    ) -> impl FnMut(Option<Res<State<GlobalGameState>>>) -> bool + Clone {
        in_state(Self::quitting())
    }
}

impl GlobalGameState {
    /// Many scenes have a standard state semantics: loading, running, quitting
    /// and paused.
    pub fn state_semantics(self) -> Option<StandardStateSemantics> {
        use GlobalGameState::*;

        let (loading, running, quitting, paused) = match self {
            LoadingBuilding1PlayerFloor
            | AtBuilding1PlayerFloor
            | QuittingBuilding1PlayerFloor => (
                LoadingBuilding1PlayerFloor,
                AtBuilding1PlayerFloor,
                QuittingBuilding1PlayerFloor,
                None,
            ),

            LoadingMall | AtMall | QuittingMall => {
                (LoadingMall, AtMall, QuittingMall, None)
            }

            LoadingBuilding1Basement1
            | AtBuilding1Basement1
            | QuittingBuilding1Basement1 => (
                LoadingBuilding1Basement1,
                AtBuilding1Basement1,
                QuittingBuilding1Basement1,
                None,
            ),

            LoadingBuilding1Basement2
            | AtBuilding1Basement2
            | QuittingBuilding1Basement2 => (
                LoadingBuilding1Basement2,
                AtBuilding1Basement2,
                QuittingBuilding1Basement2,
                None,
            ),

            LoadingClinic | AtClinic | QuittingClinic => {
                (LoadingClinic, AtClinic, QuittingClinic, None)
            }

            LoadingClinicWard | AtClinicWard | QuittingClinicWard => {
                (LoadingClinicWard, AtClinicWard, QuittingClinicWard, None)
            }

            LoadingPlantShop | AtPlantShop | QuittingPlantShop => {
                (LoadingPlantShop, AtPlantShop, QuittingPlantShop, None)
            }

            LoadingSewers | AtSewers | QuittingSewers => {
                (LoadingSewers, AtSewers, QuittingSewers, None)
            }

            LoadingTwinpeaksApartment
            | AtTwinpeaksApartment
            | QuittingTwinpeaksApartment => (
                LoadingTwinpeaksApartment,
                AtTwinpeaksApartment,
                QuittingTwinpeaksApartment,
                None,
            ),

            LoadingMeditation | InGameMeditation | MeditationInMenu
            | QuittingMeditation => (
                LoadingMeditation,
                InGameMeditation,
                QuittingMeditation,
                Some(MeditationInMenu),
            ),

            LoadingDowntown | AtDowntown | QuittingDowntown => {
                (LoadingDowntown, AtDowntown, QuittingDowntown, None)
            }

            LoadingCompound | AtCompound | QuittingCompound => {
                (LoadingCompound, AtCompound, QuittingCompound, None)
            }

            LoadingCompoundTower | AtCompoundTower | QuittingCompoundTower => (
                LoadingCompoundTower,
                AtCompoundTower,
                QuittingCompoundTower,
                None,
            ),

            Blank | Exit | NewGame => return None,
        };

        Some(StandardStateSemantics {
            loading,
            running,
            quitting,
            paused,
        })
    }
}

impl GlobalGameStateTransition {
    /// We expect the transition to start at this state.
    pub fn from_state(self) -> GlobalGameState {
        use GlobalGameStateTransition::*;

        match self {
            BlankToNewGame => GlobalGameState::Blank,
            NewGameToBuilding1PlayerFloor => GlobalGameState::NewGame,
            RestartMeditation => GlobalGameState::QuittingMeditation,
            MeditationToBuilding1PlayerFloor => {
                GlobalGameState::QuittingMeditation
            }
            Building1PlayerFloorToMeditation
            | Building1PlayerFloorToBuilding1Basement1
            | Sleeping
            | Building1PlayerFloorToDowntown => {
                GlobalGameState::QuittingBuilding1PlayerFloor
            }
            Building1Basement1ToBasement2
            | Building1Basement1ToPlayerFloor
            | Building1Basement1ToDowntown => {
                GlobalGameState::QuittingBuilding1Basement1
            }
            Building1Basement2ToBasement1 => {
                GlobalGameState::QuittingBuilding1Basement2
            }
            DowntownToBuilding1PlayerFloor
            | DowntownToClinic
            | DowntownToClinicWard
            | DowntownToCompound
            | DowntownToMall
            | DowntownToPlantShop
            | DowntownToSewers
            | DowntownToTwinpeaksApartment => GlobalGameState::QuittingDowntown,
            ClinicToDowntown => GlobalGameState::QuittingClinic,
            ClinicWardToDowntown => GlobalGameState::QuittingClinicWard,
            PlantShopToDowntown => GlobalGameState::QuittingPlantShop,
            SewersToDowntown => GlobalGameState::QuittingSewers,
            CompoundToDowntown => GlobalGameState::QuittingCompound,
            CompoundToTower => GlobalGameState::QuittingCompound,
            TwinpeaksApartmentToDowntown => {
                GlobalGameState::QuittingTwinpeaksApartment
            }
            MallToDowntown => GlobalGameState::QuittingMall,
            TowerToCompound => GlobalGameState::QuittingCompoundTower,
        }
    }
}

impl<'w, 's> TransitionParams<'w, 's> {
    /// Goes to the next state according to the transition.
    /// Uses a default loading screen setting - a random atlas for 1s.
    pub fn begin(&mut self, transition: GlobalGameStateTransition) {
        self.begin_with_settings(transition, LoadingScreenSettings {
            atlas: Some(common_loading_screen::LoadingScreenAtlas::random()),
            stare_at_loading_screen_for_at_least: Some(from_millis(1000)),
            ..default()
        })
    }

    /// Goes to the next state according to the transition.
    pub fn begin_with_settings(
        &mut self,
        transition: GlobalGameStateTransition,
        loading_screen_settings: LoadingScreenSettings,
    ) {
        self.cmd.insert_resource(loading_screen_settings);

        self.next_loading_screen_state
            .set(common_loading_screen::start_state());

        *self.transition = transition;
        self.next_state.set(transition.from_state());
    }
}
