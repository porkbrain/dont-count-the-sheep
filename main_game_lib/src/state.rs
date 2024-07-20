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

    LoadingTopDownScene(WhichTopDownScene),
    RunningTopDownScene(WhichTopDownScene),
    LeavingTopDownScene(WhichTopDownScene),

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

    /// Performs all necessary cleanup and exits the game.
    Exit,
}

/// Will be present as a resource if the game is in any top-down scene which
/// is our 2D game's most ubiquitous scene kind.
/// We use [`ComputedStates`] for this.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct InTopDownScene(pub TopDownSceneState);

/// What is the current state of the top-down scene?
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TopDownSceneState {
    /// Scene is being prepared.
    /// This entails loading assets and setting up the scene.
    Loading,
    /// Scene is running.
    /// The player can interact with the scene.
    Running,
    /// Scene is being cleaned up.
    /// This entails fading out the scene and unloading assets.
    Leaving,
}

/// All the named top-down scenes.
///
/// Is also present if the game is in a top-down scene using the
/// [`ComputedStates`].
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display, strum::AsRefStr,
)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[allow(missing_docs)]
pub enum WhichTopDownScene {
    Building1PlayerFloor,
    Building1Basement1,
    Building1Basement2,
    Clinic,
    ClinicWard,
    PlantShop,
    Sewers,
    TwinpeaksApartment,
    Mall,
    Meditation,
    Downtown,
    Compound,
    CompoundTower,
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

/// Helper to check if the state is in specific top down scene loading state.
pub fn in_scene_loading_state(
    scene: WhichTopDownScene,
) -> impl FnMut(Option<Res<State<GlobalGameState>>>) -> bool + Clone {
    in_state(GlobalGameState::LoadingTopDownScene(scene))
}

/// Helper to check if the state is in specific top down scene running state.
pub fn in_scene_running_state(
    scene: WhichTopDownScene,
) -> impl FnMut(Option<Res<State<GlobalGameState>>>) -> bool + Clone {
    in_state(GlobalGameState::RunningTopDownScene(scene))
}

/// Helper to check if the state is in specific top down scene leaving state.
pub fn in_scene_leaving_state(
    scene: WhichTopDownScene,
) -> impl FnMut(Option<Res<State<GlobalGameState>>>) -> bool + Clone {
    in_state(GlobalGameState::LeavingTopDownScene(scene))
}

/// Helper to check if the state is in _any_ top down scene loading state.
pub fn in_top_down_loading_state(
) -> impl FnMut(Option<Res<State<InTopDownScene>>>) -> bool + Clone {
    in_state(InTopDownScene(TopDownSceneState::Loading))
}

/// Helper to check if the state is in _any_ top down scene running state.
pub fn in_top_down_running_state(
) -> impl FnMut(Option<Res<State<InTopDownScene>>>) -> bool + Clone {
    in_state(InTopDownScene(TopDownSceneState::Running))
}

/// Helper to check if the state is in _any_ top down scene leaving state.
pub fn in_top_down_leaving_state(
) -> impl FnMut(Option<Res<State<InTopDownScene>>>) -> bool + Clone {
    in_state(InTopDownScene(TopDownSceneState::Leaving))
}

impl GlobalGameStateTransition {
    /// We expect the transition to start at this state.
    pub fn from_state(self) -> GlobalGameState {
        use GlobalGameStateTransition::*;
        use WhichTopDownScene::*;

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
            | Building1PlayerFloorToDowntown => Building1PlayerFloor.leaving(),
            Building1Basement1ToBasement2
            | Building1Basement1ToPlayerFloor
            | Building1Basement1ToDowntown => Building1Basement1.leaving(),
            Building1Basement2ToBasement1 => Building1Basement2.leaving(),
            DowntownToBuilding1PlayerFloor
            | DowntownToClinic
            | DowntownToClinicWard
            | DowntownToCompound
            | DowntownToMall
            | DowntownToPlantShop
            | DowntownToSewers
            | DowntownToTwinpeaksApartment => Downtown.leaving(),
            ClinicToDowntown => Clinic.leaving(),
            ClinicWardToDowntown => ClinicWard.leaving(),
            PlantShopToDowntown => PlantShop.leaving(),
            SewersToDowntown => Sewers.leaving(),
            CompoundToDowntown => Compound.leaving(),
            CompoundToTower => Compound.leaving(),
            TwinpeaksApartmentToDowntown => TwinpeaksApartment.leaving(),
            MallToDowntown => Mall.leaving(),
            TowerToCompound => CompoundTower.leaving(),
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

impl ComputedStates for InTopDownScene {
    type SourceStates = Option<GlobalGameState>;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            Some(GlobalGameState::LoadingTopDownScene(_)) => {
                Some(InTopDownScene(TopDownSceneState::Loading))
            }
            Some(GlobalGameState::RunningTopDownScene(_)) => {
                Some(InTopDownScene(TopDownSceneState::Running))
            }
            Some(GlobalGameState::LeavingTopDownScene(_)) => {
                Some(InTopDownScene(TopDownSceneState::Leaving))
            }
            _ => None,
        }
    }
}

impl ComputedStates for WhichTopDownScene {
    type SourceStates = Option<GlobalGameState>;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            Some(
                GlobalGameState::LoadingTopDownScene(scene)
                | GlobalGameState::RunningTopDownScene(scene)
                | GlobalGameState::LeavingTopDownScene(scene),
            ) => Some(scene),
            _ => None,
        }
    }
}

impl InTopDownScene {
    /// Is the scene in the [`TopDownSceneState::Running`] state?
    pub fn is_running(self) -> bool {
        matches!(self.0, TopDownSceneState::Running)
    }
}

impl WhichTopDownScene {
    /// Instance of the scene in the loading state.
    pub fn loading(self) -> GlobalGameState {
        GlobalGameState::LoadingTopDownScene(self)
    }

    /// Instance of the scene in the running state.
    pub fn running(self) -> GlobalGameState {
        GlobalGameState::RunningTopDownScene(self)
    }

    /// Instance of the scene in the leaving state.
    pub fn leaving(self) -> GlobalGameState {
        GlobalGameState::LeavingTopDownScene(self)
    }
}
