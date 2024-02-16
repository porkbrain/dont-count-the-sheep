//! Game state management.

use crate::prelude::*;

/// Provides control for the game states.
/// Each scene can add whatever state it needs to this enum.
/// Transitions between states are controlled by the
/// [`GlobalGameStateTransitionStack`]. It defines what transitions are allowed.
#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect)]
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

    /// Sets up the downtown scene in the background.
    DowntownLoading,
    /// Player is at downtown.
    AtDowntown,
    /// Despawn downtown game resources.
    DowntownQuitting,

    /// Performs all necessary cleanup and exits the game.
    Exit,

    /// A state for development purposes.
    #[cfg(feature = "dev-playground")]
    InDevPlayground,
}

/// What are the allowed transitions between game states?
#[derive(Debug, Reflect, Clone, Copy, Eq, PartialEq)]
pub enum GlobalGameStateTransition {
    /// Restart the game
    MeditationQuittingToMeditationLoading,
    /// Exit back to the apartment
    MeditationQuittingToApartment,

    /// Play the meditation minigame
    ApartmentQuittingToMeditationLoading,
    /// Quit the game
    ApartmentQuittingToExit,
    /// Go to downtown
    ApartmentQuittingToDowntownLoading,
}

/// Certain states have multiple allowed transitions.
/// The tip of the stack must always match the current state.
#[derive(Resource, Debug, Default, Reflect)]
pub struct GlobalGameStateTransitionStack {
    stack: Vec<GlobalGameStateTransition>,
}
impl GlobalGameStateTransitionStack {
    /// Expect a transition.
    pub fn push(&mut self, transition: GlobalGameStateTransition) {
        self.stack.push(transition);
    }

    /// Given state that's ready to transition, where should we go next?
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
            (Some(ApartmentQuittingToDowntownLoading), ApartmentQuitting) => {
                Some(DowntownLoading)
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
