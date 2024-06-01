//! Global actions are accessible as a resource.

#![deny(missing_docs)]

use bevy::prelude::*;
pub use leafwing_input_manager::{self, action_state::ActionState};
use leafwing_input_manager::{
    axislike::{DualAxis, VirtualDPad},
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
            .add_plugins(InputManagerPlugin::<GlobalAction>::default());

        #[cfg(feature = "devtools")]
        {
            use leafwing_input_manager::action_state::ActionData;

            app.register_type::<GlobalAction>()
                .register_type::<ActionState<GlobalAction>>()
                .register_type::<ActionData>();
        }
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

    /// WASD, arrow keys, controller dpad or left stick.
    Move,

    /// When held, the player is in an inspect mode.
    /// This is mainly relevant for actions of gathering information about the
    /// world.
    Inspect,

    /// Numeric input for zero.
    NumZero,
    /// Numeric input for one.
    NumOne,
    /// Numeric input for two.
    NumTwo,
    /// Numeric input for three.
    NumThree,
    /// Numeric input for four.
    NumFour,
    /// Numeric input for five.
    NumFive,
    /// Numeric input for six.
    NumSix,
    /// Numeric input for seven.
    NumSeven,
    /// Numeric input for eight.
    NumEight,
    /// Numeric input for nine.
    NumNine,
}

/// You can get this action from a [`GlobalAction::Move`].
/// It is a discrete form of the "analog" dual axis input.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, EnumIter)]
pub enum MovementAction {
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
}

/// Runs a system if cancel action is being held.
pub fn cancel_just_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_pressed(&GlobalAction::Cancel)
    }
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
        action_state.pressed(&GlobalAction::Move)
    }
}

/// Any numeric key is being held.
pub fn numeric_key_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        GlobalAction::numerical()
            .into_iter()
            .any(|action| action_state.pressed(&action))
    }
}

/// Any movement action was just pressed.
pub fn move_action_just_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_pressed(&GlobalAction::Move)
    }
}

impl GlobalAction {
    /// Returns all numeric actions from zero to nine.
    pub fn numerical() -> Vec<Self> {
        vec![
            Self::NumZero,
            Self::NumOne,
            Self::NumTwo,
            Self::NumThree,
            Self::NumFour,
            Self::NumFive,
            Self::NumSix,
            Self::NumSeven,
            Self::NumEight,
            Self::NumNine,
        ]
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
        use GamepadButtonType::*;
        use InputKind::{GamepadButton as GPad, PhysicalKey as Kbd};
        use KeyCode::*;
        use UserInput::Single;

        match action {
            Self::Interact => {
                vec![
                    Single(Kbd(Space)),
                    Single(Kbd(Enter)),
                    Single(GPad(South)), // A
                ]
            }
            Self::Cancel => {
                vec![
                    Single(Kbd(Escape)),
                    Single(GPad(North)), // Y
                    Single(GPad(GamepadButtonType::Select)),
                ]
            }
            Self::Move => {
                vec![
                    Single(InputKind::DualAxis(DualAxis::left_stick())),
                    UserInput::VirtualDPad(VirtualDPad::wasd()),
                    UserInput::VirtualDPad(VirtualDPad::dpad()),
                    UserInput::VirtualDPad(VirtualDPad::arrow_keys()),
                ]
            }
            Self::Inspect => vec![
                Single(Kbd(AltLeft)),
                Single(Kbd(AltRight)),
                Single(GPad(LeftTrigger)),
                Single(GPad(LeftTrigger2)),
            ],
            Self::NumZero => vec![Single(Kbd(Digit0))],
            Self::NumOne => vec![Single(Kbd(Digit1))],
            Self::NumTwo => vec![Single(Kbd(Digit2))],
            Self::NumThree => vec![Single(Kbd(Digit3))],
            Self::NumFour => vec![Single(Kbd(Digit4))],
            Self::NumFive => vec![Single(Kbd(Digit5))],
            Self::NumSix => vec![Single(Kbd(Digit6))],
            Self::NumSeven => vec![Single(Kbd(Digit7))],
            Self::NumEight => vec![Single(Kbd(Digit8))],
            Self::NumNine => vec![Single(Kbd(Digit9))],
        }
    }
}

/// Extends [`ActionState`] with methods specific to this game.
pub trait ActionStateExt {
    /// Returns the movement action if the action state is in a movement state.
    fn movement_action(&self) -> Option<MovementAction>;
}

impl ActionStateExt for ActionState<GlobalAction> {
    fn movement_action(&self) -> Option<MovementAction> {
        let axis_pair = self.axis_pair(&GlobalAction::Move)?;
        from_dual_axis(axis_pair.xy())
    }
}

impl MovementAction {
    /// ↖↑↗
    pub fn is_in_up_direction(self) -> bool {
        matches!(self, Self::MoveUp | Self::MoveUpLeft | Self::MoveUpRight)
    }

    /// ↘↓↙
    pub fn is_in_down_direction(self) -> bool {
        matches!(
            self,
            Self::MoveDown | Self::MoveDownLeft | Self::MoveDownRight
        )
    }

    /// ↖←↙
    pub fn is_in_left_direction(self) -> bool {
        matches!(self, Self::MoveLeft | Self::MoveUpLeft | Self::MoveDownLeft)
    }

    /// ↗→↘
    pub fn is_in_right_direction(self) -> bool {
        matches!(
            self,
            Self::MoveRight | Self::MoveUpRight | Self::MoveDownRight
        )
    }
}

fn from_dual_axis(left_stick: Vec2) -> Option<MovementAction> {
    // Generated by chat-gpt based on my description and example of the output
    // enum.

    use MovementAction::*;

    // Define a small threshold to avoid noise in the analog stick
    let threshold = 0.1;

    // Check if the stick is within the dead zone
    if left_stick.x.abs() < threshold && left_stick.y.abs() < threshold {
        return None;
    }

    // Calculate the angle in radians
    let angle = left_stick.y.atan2(left_stick.x);

    // Determine the action based on the angle
    let action = if angle >= -3.0 * std::f32::consts::PI / 8.0
        && angle < -std::f32::consts::PI / 8.0
    {
        MoveDownRight // ↘
    } else if angle >= -std::f32::consts::PI / 8.0
        && angle < std::f32::consts::PI / 8.0
    {
        MoveRight // →
    } else if angle >= std::f32::consts::PI / 8.0
        && angle < 3.0 * std::f32::consts::PI / 8.0
    {
        MoveUpRight // ↗
    } else if angle >= 3.0 * std::f32::consts::PI / 8.0
        && angle < 5.0 * std::f32::consts::PI / 8.0
    {
        MoveUp // ↑
    } else if angle >= 5.0 * std::f32::consts::PI / 8.0
        && angle < 7.0 * std::f32::consts::PI / 8.0
    {
        MoveUpLeft // ↖
    } else if angle >= 7.0 * std::f32::consts::PI / 8.0
        || angle < -7.0 * std::f32::consts::PI / 8.0
    {
        MoveLeft // ←
    } else if angle >= -7.0 * std::f32::consts::PI / 8.0
        && angle < -5.0 * std::f32::consts::PI / 8.0
    {
        MoveDownLeft // ↙
    } else {
        MoveDown // ↓
    };

    Some(action)
}
