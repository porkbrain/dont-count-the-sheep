//! Systems related to the player.

use bevy::prelude::*;
use bevy_grid_squared::{direction::Direction as GridDirection, Square};
use common_action::GlobalAction;
use common_layout::{IntoMap, SquareKind};
use leafwing_input_manager::action_state::ActionState;

use super::{Actor, ActorTarget};

/// The entity that the player controls.
/// Bound it with [`Actor`] to allow movement.
#[derive(Component, Reflect)]
pub struct Player;

/// Use keyboard to move around the player.
pub fn move_around<T: IntoMap>(
    map: Res<common_layout::Map<T>>,
    controls: Res<ActionState<GlobalAction>>,

    mut player: Query<&mut Actor, With<Player>>,
) {
    use GridDirection::*;

    let next_steps = match controls.get_pressed().last() {
        Some(GlobalAction::MoveUp) => [Top, TopLeft, TopRight],
        Some(GlobalAction::MoveDown) => [Bottom, BottomLeft, BottomRight],
        Some(GlobalAction::MoveLeft) => [Left, TopLeft, BottomLeft],
        Some(GlobalAction::MoveRight) => [Right, TopRight, BottomRight],
        Some(GlobalAction::MoveUpLeft) => [TopLeft, Top, Left],
        Some(GlobalAction::MoveUpRight) => [TopRight, Top, Right],
        Some(GlobalAction::MoveDownLeft) => [BottomLeft, Bottom, Left],
        Some(GlobalAction::MoveDownRight) => [BottomRight, Bottom, Right],
        _ => {
            return;
        }
    };

    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    if player
        .walking_to
        .as_ref()
        .and_then(|to| to.planned)
        .is_some()
    {
        return;
    }

    // exhaustive match in case of future changes
    let is_available = |square: Square| match map.get(&square) {
        None => T::contains(square),
        Some(SquareKind::None | SquareKind::Zone(_)) => true,
        Some(SquareKind::Object | SquareKind::Wall) => false,
    };

    let plan_from = player.current_square();

    let target = next_steps.iter().copied().find_map(|direction| {
        let target = plan_from.neighbor(direction);
        is_available(target).then_some((target, direction))
    });

    player.step_time = player.character.default_step_time();
    if let Some((target_square, direction)) = target {
        player.direction = direction;

        if let Some(walking_to) = &mut player.walking_to {
            debug_assert!(walking_to.planned.is_none());
            walking_to.planned = Some((target_square, direction));
        } else {
            player.walking_to = Some(ActorTarget::new(target_square));
        }
    } else {
        // Cannot move anywhere, but would like to? At least direction the
        // sprite towards the attempted direction.

        player.direction = next_steps[0];
    }
}
