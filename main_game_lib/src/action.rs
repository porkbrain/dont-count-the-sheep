//! Global actions are accessible as a resource.

use leafwing_input_manager::{
    input_map::InputMap,
    user_input::{InputKind, UserInput},
    Actionlike,
};

use crate::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum GlobalAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveUpLeft,
    MoveUpRight,
    MoveDownLeft,
    MoveDownRight,

    Interact,
    Cancel,
}

pub fn interaction_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.pressed(GlobalAction::Interact)
    }
}

pub fn interaction_just_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.just_pressed(GlobalAction::Interact)
    }
}

pub fn move_action_pressed(
) -> impl FnMut(Res<ActionState<GlobalAction>>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>| {
        action_state.pressed(GlobalAction::MoveUp)
            || action_state.pressed(GlobalAction::MoveDown)
            || action_state.pressed(GlobalAction::MoveLeft)
            || action_state.pressed(GlobalAction::MoveRight)
            || action_state.pressed(GlobalAction::MoveUpLeft)
            || action_state.pressed(GlobalAction::MoveUpRight)
            || action_state.pressed(GlobalAction::MoveDownLeft)
            || action_state.pressed(GlobalAction::MoveDownRight)
    }
}

impl GlobalAction {
    pub(crate) fn input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        for action in GlobalAction::variants() {
            for input in GlobalAction::default_keyboard_input(action) {
                input_map.insert(input, action);
            }
        }

        input_map
    }

    fn default_keyboard_input(action: GlobalAction) -> Vec<UserInput> {
        use InputKind::Keyboard as Kbd;
        use KeyCode as Key;
        use UserInput::{Chord, Single};

        match action {
            Self::MoveDown => {
                vec![Single(Kbd(Key::S)), Single(Kbd(Key::Down))]
            }
            Self::MoveLeft => {
                vec![Single(Kbd(Key::A)), Single(Kbd(Key::Left))]
            }
            Self::MoveRight => {
                vec![Single(Kbd(Key::D)), Single(Kbd(Key::Right))]
            }
            Self::MoveUp => {
                vec![Single(Kbd(Key::W)), Single(Kbd(Key::Up))]
            }
            Self::MoveDownLeft => vec![
                Chord(vec![Kbd(Key::S), Kbd(Key::A)]),
                Chord(vec![Kbd(Key::Down), Kbd(Key::Left)]),
            ],
            Self::MoveDownRight => vec![
                Chord(vec![Kbd(Key::S), Kbd(Key::D)]),
                Chord(vec![Kbd(Key::Down), Kbd(Key::Right)]),
            ],
            Self::MoveUpLeft => vec![
                Chord(vec![Kbd(Key::W), Kbd(Key::A)]),
                Chord(vec![Kbd(Key::Up), Kbd(Key::Left)]),
            ],
            Self::MoveUpRight => vec![
                Chord(vec![Kbd(Key::W), Kbd(Key::D)]),
                Chord(vec![Kbd(Key::Up), Kbd(Key::Right)]),
            ],
            Self::Interact => {
                vec![Single(Kbd(Key::Space)), Single(Kbd(Key::Return))]
            }
            Self::Cancel => {
                vec![Single(Kbd(Key::Escape))]
            }
        }
    }
}
