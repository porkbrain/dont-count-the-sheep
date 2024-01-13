use crate::prelude::*;

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
