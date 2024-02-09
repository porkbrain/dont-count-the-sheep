//! Systems related to the player.

use bevy::prelude::*;
use bevy_grid_squared::GridDirection;
use common_action::GlobalAction;
use common_ext::QueryExt;
use leafwing_input_manager::action_state::ActionState;

use super::{Actor, ActorTarget};
use crate::layout::{IntoMap, TileMap};

/// The entity that the player controls.
/// Bound it with [`Actor`] to allow movement.
///
/// This information is duplicated.
/// It's also stored on the [`Actor`] component as a flag.
#[derive(Component, Reflect)]
pub struct Player;

/// Use keyboard to move around the player.
pub fn move_around<T: IntoMap>(
    map: Res<TileMap<T>>,
    controls: Res<ActionState<GlobalAction>>,

    mut player: Query<(Entity, &mut Actor), With<Player>>,
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

    let Some((player_entity, mut player)) = player.get_single_mut_or_none()
    else {
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

    let plan_from = player.current_square();

    let target = next_steps.iter().copied().find_map(|direction| {
        let target = plan_from.neighbor(direction);
        map.is_walkable(target, player_entity)
            .then_some((target, direction))
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
