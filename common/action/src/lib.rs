//! Global actions are accessible as a resource.

#![deny(missing_docs)]

use bevy::prelude::*;
pub use leafwing_input_manager::{self, action_state::ActionState};
use leafwing_input_manager::{
    action_state::ActionData,
    input_map::InputMap,
    plugin::InputManagerPlugin,
    user_input::{InputKind, UserInput},
    Actionlike,
};
use strum::{EnumIter, IntoEnumIterator};

/// Registers necessary types, inserts resources and adds the dependent
/// [`InputManagerPlugin`].
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionState<GlobalAction>>()
            .insert_resource(GlobalAction::input_map())
            .add_plugins(InputManagerPlugin::<GlobalAction>::default())
            .register_type::<GlobalAction>()
            .register_type::<ActionState<GlobalAction>>()
            .register_type::<ActionData>();
    }
}

/// These actions are used throughout the game.
#[derive(
    Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, EnumIter,
)]
#[non_exhaustive]
pub enum GlobalAction {
    /// Ubiquitous action, used for interacting with the world.
    Interact,
    /// Go to menu etc.
    Cancel,

    /// Going only up.
    MoveUp,
    /// Going only down.
    MoveDown,
    /// Going only left.
    MoveLeft,
    /// Going only right.
    MoveRight,
    /// Going both up and left.
    /// Overwrites `MoveUp` and `MoveLeft`.
    MoveUpLeft,
    /// Going both up and right.
    /// Overwrites `MoveUp` and `MoveRight`.
    MoveUpRight,
    /// Going both down and left.
    /// Overwrites `MoveDown` and `MoveLeft`.
    MoveDownLeft,
    /// Going both down and right.
    /// Overwrites `MoveDown` and `MoveRight`.
    MoveDownRight,

    /// When held, the player is in an inspect mode.
    /// This is mainly relevant for actions of gathering information about the
    /// world.
    Inspect,
}

/// Runs a system if inspect action is being held.
pub fn inspect_just_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_pressed(&GlobalAction::Inspect)
    }
}

/// Runs a system if inspect action is being held.
pub fn inspect_pressed() -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.pressed(&GlobalAction::Inspect)
    }
}

/// Runs a system if inspect action was just released.
pub fn inspect_just_released(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_released(&GlobalAction::Inspect)
    }
}

/// Runs a system if interaction with the world is being held.
pub fn interaction_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.pressed(&GlobalAction::Interact)
    }
}

/// Runs a system if interaction with the world was just pressed.
pub fn interaction_just_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_pressed(&GlobalAction::Interact)
    }
}

/// Any movement action is being held.
pub fn move_action_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.pressed(&GlobalAction::MoveUp)
            || action_state.pressed(&GlobalAction::MoveDown)
            || action_state.pressed(&GlobalAction::MoveLeft)
            || action_state.pressed(&GlobalAction::MoveRight)
            || action_state.pressed(&GlobalAction::MoveUpLeft)
            || action_state.pressed(&GlobalAction::MoveUpRight)
            || action_state.pressed(&GlobalAction::MoveDownLeft)
            || action_state.pressed(&GlobalAction::MoveDownRight)
    }
}

/// Any movement action was just pressed.
pub fn move_action_just_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_pressed(&GlobalAction::MoveUp)
            || action_state.just_pressed(&GlobalAction::MoveDown)
            || action_state.just_pressed(&GlobalAction::MoveLeft)
            || action_state.just_pressed(&GlobalAction::MoveRight)
            || action_state.just_pressed(&GlobalAction::MoveUpLeft)
            || action_state.just_pressed(&GlobalAction::MoveUpRight)
            || action_state.just_pressed(&GlobalAction::MoveDownLeft)
            || action_state.just_pressed(&GlobalAction::MoveDownRight)
    }
}

impl GlobalAction {
    /// Whether the action is a directional input for movement.
    /// That's one of
    /// - [`Self::MoveUp`]
    /// - [`Self::MoveDown`]
    /// - [`Self::MoveLeft`]
    /// - [`Self::MoveRight`]
    /// - [`Self::MoveUpLeft`]
    /// - [`Self::MoveUpRight`]
    /// - [`Self::MoveDownLeft`]
    /// - [`Self::MoveDownRight`]
    pub fn is_directional(self) -> bool {
        matches!(
            self,
            Self::MoveUp
                | Self::MoveDown
                | Self::MoveLeft
                | Self::MoveRight
                | Self::MoveUpLeft
                | Self::MoveUpRight
                | Self::MoveDownLeft
                | Self::MoveDownRight
        )
    }

    fn input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        for action in GlobalAction::iter() {
            for input in GlobalAction::default_keyboard_input(action) {
                input_map.insert(action, input);
            }
        }

        input_map
    }

    fn default_keyboard_input(action: GlobalAction) -> Vec<UserInput> {
        use InputKind::PhysicalKey as Kbd;
        use KeyCode::*;
        use UserInput::{Chord, Single};

        match action {
            Self::Interact => {
                vec![Single(Kbd(Space)), Single(Kbd(Enter))]
            }
            Self::Cancel => {
                vec![Single(Kbd(Escape))]
            }
            Self::MoveDown => {
                vec![Single(Kbd(KeyS)), Single(Kbd(ArrowDown))]
            }
            Self::MoveLeft => {
                vec![Single(Kbd(KeyA)), Single(Kbd(ArrowLeft))]
            }
            Self::MoveRight => {
                vec![Single(Kbd(KeyD)), Single(Kbd(ArrowRight))]
            }
            Self::MoveUp => {
                vec![Single(Kbd(KeyW)), Single(Kbd(ArrowUp))]
            }
            Self::MoveDownLeft => vec![
                Chord(vec![Kbd(KeyS), Kbd(KeyA)]),
                Chord(vec![Kbd(ArrowDown), Kbd(ArrowLeft)]),
            ],
            Self::MoveDownRight => vec![
                Chord(vec![Kbd(KeyS), Kbd(KeyD)]),
                Chord(vec![Kbd(ArrowDown), Kbd(ArrowRight)]),
            ],
            Self::MoveUpLeft => vec![
                Chord(vec![Kbd(KeyW), Kbd(KeyA)]),
                Chord(vec![Kbd(ArrowUp), Kbd(ArrowLeft)]),
            ],
            Self::MoveUpRight => vec![
                Chord(vec![Kbd(KeyW), Kbd(KeyD)]),
                Chord(vec![Kbd(ArrowUp), Kbd(ArrowRight)]),
            ],
            Self::Inspect => vec![Single(Kbd(AltLeft))],
        }
    }
}
