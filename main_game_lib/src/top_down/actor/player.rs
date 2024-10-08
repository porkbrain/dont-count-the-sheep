//! Systems related to the player.

use bevy::prelude::*;
use bevy_grid_squared::{GridDirection, Square};
use common_action::{ActionStateExt, GlobalAction, MovementAction};
use common_ext::QueryExt;
use leafwing_input_manager::action_state::ActionState;

use super::{Actor, ActorTarget};
use crate::top_down::layout::TileMap;

/// The entity that the player controls.
/// Bound it with [`Actor`] to allow movement.
///
/// This information is duplicated.
/// It's also stored on the [`Actor`] component as a flag.
#[derive(Component, Reflect)]
pub struct Player;

/// If the [`Player`] has this component, the player cannot perform movement
/// actions.
#[derive(Component, Reflect)]
pub struct TakeAwayPlayerControl;

/// Use keyboard to move around the player.
pub fn move_around(
    map: Res<TileMap>,
    controls: Res<ActionState<GlobalAction>>,

    mut player: Query<
        (Entity, &mut Actor),
        (With<Player>, Without<TakeAwayPlayerControl>),
    >,
) {
    // there must be some user action
    let Some(action) = controls.movement_action() else {
        return;
    };
    // that leads to a movement command
    let Some((primary_steps, alternative_steps)) =
        to_direction_commands(action)
    else {
        return;
    };
    // there must be someone to move
    let Some((player_entity, mut player)) = player.get_single_mut_or_none()
    else {
        return;
    };
    // who doesn't yet have all the movement planned
    if player
        .walking_to
        .target()
        .and_then(|to| to.planned)
        .is_some()
    {
        return;
    }

    player.step_time = player.character.default_step_time();

    let plan_from = player.current_square();

    // given a position and options, find the first walkable square
    let find_target = |from: Square, options: &[_]| {
        options.iter().copied().find_map(|direction| {
            let target = from.neighbor(direction);
            map.is_walkable(target, player_entity)
                .then_some((target, direction))
        })
    };

    // either we find a walkable square in the primary directions
    // or else we consider the alternative directions
    let target = find_target(plan_from, primary_steps).or_else(|| {
        // if we can't move in the primary directions, try the fallback
        let (target_square, _) = find_target(plan_from, alternative_steps?)?;
        // but the fallback has to satisfy 2 conditions:

        // 1. next step can be made with primary directions
        let (square_after_that, direction_after_that) =
            find_target(target_square, primary_steps)?;

        // 2. next step won't move us back to where we are now
        if target_square == square_after_that {
            None
        } else {
            // we use the upcoming direction to avoid flickering
            Some((target_square, direction_after_that))
        }
    });

    if let Some((target_square, direction)) = target {
        player.direction = direction;

        if let Some(walking_to) = &mut player.walking_to.target_mut() {
            debug_assert!(walking_to.planned.is_none());
            walking_to.planned = Some((target_square, direction));
        } else {
            player.walking_to = ActorTarget::new(target_square).into();
        }
    } else {
        // Cannot move anywhere, but would like to? At least direction the
        // sprite towards the attempted direction.

        player.direction = primary_steps[0];
    }
}

/// Convert a global action to a list of directions to move in.
///
/// Some actions have secondary directions to consider if the primary ones
/// are blocked.
///
/// See the [`move_around`] logic to understand how secondary directions are
/// used.
fn to_direction_commands(
    action: MovementAction,
) -> Option<(&'static [GridDirection], Option<&'static [GridDirection]>)> {
    use GridDirection::*;

    let steps: (&[_], Option<&[_]>) = match action {
        MovementAction::MoveUp => (&[Top, TopLeft, TopRight], None),
        MovementAction::MoveDown => (&[Bottom, BottomLeft, BottomRight], None),
        MovementAction::MoveLeft => (&[Left, TopLeft, BottomLeft], None),
        MovementAction::MoveRight => (&[Right, TopRight, BottomRight], None),

        // These diagonal movement combined actions can end up getting stuck
        // in diagonal corners, for example:
        //
        // ```
        //   --
        //   p|
        // ```
        //
        // Here, the player `p` is stuck in the top right corner if we only
        // consider Up, Right, UpRight, so let's also consider UpLeft and
        // BottomRight squares.
        // This is analogous for the other diagonal movements.
        MovementAction::MoveUpLeft => {
            (&[TopLeft, Top, Left], Some(&[TopRight, BottomLeft]))
        }
        MovementAction::MoveUpRight => {
            (&[TopRight, Top, Right], Some(&[TopLeft, BottomRight]))
        }
        MovementAction::MoveDownLeft => {
            (&[BottomLeft, Bottom, Left], Some(&[TopLeft, BottomRight]))
        }
        MovementAction::MoveDownRight => {
            (&[BottomRight, Bottom, Right], Some(&[TopRight, BottomLeft]))
        }
    };

    Some(steps)
}
