//! NPC logic.
//!
//! Behavior trees are run with [`drive_behavior`] system.

pub mod behaviors;

use std::{
    ops::{AddAssign, Not},
    time::{Duration, Instant},
};

use bevy::prelude::*;
use bevy_grid_squared::Square;
use common_ext::QueryExt;
use common_store::{DialogStore, GlobalStore};

use super::{ActorOrCharacter, BeginDialogEvent};
use crate::{
    dialog::{self, StartDialogWhenLoaded},
    top_down::{
        inspect_and_interact::ReadyForInteraction, layout::ZoneTile, Actor,
        ActorTarget, Player, TileMap, TopDownScene,
    },
};

const MIN_WAIT_BETWEEN_PATHFINDING_RETRY: Duration = Duration::from_millis(250);

/// Describes state of an NPC that's positioned in the current map.
/// As opposed to just an abstract simulation, this NPC is actively moving and
/// can be interacted with.
#[derive(Component, Reflect, Default)]
pub struct NpcInTheMap {
    /// Queue of squares that the NPC is planning to visit or has already
    /// visited.
    /// That's determined by the index.
    /// Populated by the pathfinding algorithm.
    /// Empty if the NPC has no further plans.
    /// Is cleared when the index reaches the end.
    planned_path: Vec<Square>,
    /// Where are we right now in the planned path queue.
    /// Is zero if empty path and also if there's only one square in the path.
    planned_path_index: usize,
}

/// Run path finding algorithm for given entity to given square.
#[derive(Event, Reflect)]
pub struct PlanPathEvent(Entity, Square);

/// Carries information about the output of work.
#[derive(Debug, Reflect, Clone, Copy)]
pub enum BehaviorResult {
    /// Everything is awesome when you're part of a team.
    Ok,
    /// The last behavior node failed.
    Failed,
}

/// A behavior tree can be attached to an NPC to drive its actions.
#[derive(Debug, Component)]
pub struct BehaviorTree {
    /// The zeroth element is the root node.
    /// The last element is the currently visited node.
    /// The first tuple element is the number of visits to the node.
    visiting_stack: Vec<(usize, BehaviorNode)>,
    /// Carries information about the last result of a node.
    /// This is none in the beginning or if unset by some composite or
    /// decorator node.
    last_result: Option<BehaviorResult>,
}

/// A behavior tree node.
/// The tree is traversed depth-first.
///
/// There are composite and decorative nodes and then there are leaf nodes.
/// Leaf nodes perform actual work on the world.
///
/// Nodes can carry state.
/// When a node is visited for the first time, it is cloned and pushed onto the
/// stack.
#[derive(Debug, Clone)]
pub enum BehaviorNode {
    /// Composite node that runs one child after another.
    /// Exits with failed if any child fails, otherwise succeeds.
    Sequence(Vec<BehaviorNode>),
    /// Repeats the child until it fails.
    Repeat(Box<BehaviorNode>),
    /// Inverts the result of the child.
    Invert(Box<BehaviorNode>),
    /// Regardless of whether child fails or not, this node always succeeds.
    Infallible(Box<BehaviorNode>),

    /// Leaf node that performs an action.
    Leaf(BehaviorLeaf),
    /// If the child node does not finish in time, this node fails.
    LeafWithTimeout(BehaviorLeaf, Timer),
}

/// Different actions that can be performed by a leaf node.
#[derive(Debug, Reflect, Clone)]
pub enum BehaviorLeaf {
    /// Does nothing, runs forever.
    Idle,
    /// Drives the pathfinding algorithm to find a path to the given square and
    /// the NPC to move along it.
    FindPath {
        /// The square to find a path to.
        to: Square,
        /// When did we last try to find a path.
        /// Prevents spamming the pathfinding algorithm too often.
        last_attempt: Option<Instant>,
    },
}

/// An NPC with this component will not further execute its behavior tree.
///
/// This is for example inserted when the NPC enters a dialog.
#[derive(Component, Reflect)]
pub struct BehaviorPaused;

/// Runs all unpaused behavior trees.
pub fn drive_behavior(
    mut cmd: Commands,
    time: Res<Time>,
    mut plan_path: EventWriter<PlanPathEvent>,

    mut trees: Query<(Entity, &mut BehaviorTree), Without<BehaviorPaused>>,
    actors: Query<(&Actor, &NpcInTheMap)>,
) {
    for (tree_entity, mut tree) in trees.iter_mut() {
        let Some((_visit, leaf)) = tree.unfold_into_leaf(&time) else {
            cmd.entity(tree_entity).remove::<BehaviorTree>();
            continue;
        };

        use BehaviorLeaf::*;
        match leaf {
            Idle => {}
            FindPath { to, last_attempt } => {
                if let Some(last_attempt) = last_attempt {
                    if last_attempt.elapsed()
                        < MIN_WAIT_BETWEEN_PATHFINDING_RETRY
                    {
                        continue;
                    }
                }

                let Ok((actor, npc_in_the_map)) = actors.get(tree_entity)
                else {
                    // NPC is only virtual or does not exist,
                    // we don't actually have to move it
                    tree.leaf_finished(BehaviorResult::Ok);
                    continue;
                };

                let current_square = actor.current_square();

                if current_square == *to {
                    trace!(
                        "NPC {:?} reached target square {to:?}",
                        actor.character
                    );
                    tree.leaf_finished(BehaviorResult::Ok);
                    continue;
                }

                // nowhere planned and nowhere walking, yet not at target
                if npc_in_the_map.planned_path.is_empty()
                    && actor.walking_to.is_still()
                {
                    plan_path.send(PlanPathEvent(tree_entity, *to));
                    *last_attempt = Some(Instant::now());
                }
            }
        }
    }
}

/// Plans a path for an NPC within the map.
/// If no path can be found, the planned path is set to an empty vector.
///
/// Condition this to run only on new event.
pub fn plan_path<T: TopDownScene>(
    map: Res<TileMap<T>>,
    mut events: EventReader<PlanPathEvent>,

    mut actors: Query<(Entity, &Actor, &mut NpcInTheMap)>,
) where
    T::LocalTileKind: ZoneTile<Successors = T::LocalTileKind>,
{
    for PlanPathEvent(entity, target_square) in events.read() {
        let Ok((actor_entity, actor, mut npc_in_the_map)) =
            actors.get_mut(*entity)
        else {
            continue;
        };

        trace!("Searching for path to {target_square:?}");
        npc_in_the_map.planned_path_index = 0;
        npc_in_the_map.planned_path = map
            .find_partial_path(
                actor_entity,
                actor.current_square(),
                *target_square,
            )
            .unwrap_or_default(); // no path
        trace!("Found path of len {}", npc_in_the_map.planned_path.len());
    }
}

/// Takes tiles from the planned path and sets it as the actor's goal path.
///
/// We only do this if the behavior tree is not paused.
/// E.g. when the NPC enters a dialog, we don't want it to move.
pub fn run_path<T: TopDownScene>(
    map: Res<TileMap<T>>,

    mut actors: Query<
        (Entity, &mut Actor, &mut NpcInTheMap),
        Without<BehaviorPaused>,
    >,
) {
    for (actor_entity, mut actor, mut npc_in_the_map) in actors.iter_mut() {
        if npc_in_the_map.planned_path.is_empty() {
            continue;
        }

        match actor.walking_to.target_mut() {
            Some(target) if target.planned.is_some() => continue,

            // there's next square but we can even add a new plan
            Some(target) => {
                let Some(planned_square) = npc_in_the_map.next_planned_square()
                else {
                    continue;
                };
                if !map.is_walkable(planned_square, actor_entity) {
                    // we'll need to replan
                    npc_in_the_map.reset_path();
                    continue;
                }
                let Some(direction) =
                    target.square.direction_to(planned_square)
                else {
                    continue;
                };

                target.planned = Some((planned_square, direction));
            }
            // not moving, make their bones
            None => {
                let Some(planned_square) = npc_in_the_map.next_planned_square()
                else {
                    continue;
                };
                if !map.is_walkable(planned_square, actor_entity) {
                    // we'll need to replan
                    npc_in_the_map.reset_path();
                    continue;
                }
                let Some(direction) =
                    actor.walking_from.direction_to(planned_square)
                else {
                    continue;
                };

                actor.walking_to = ActorTarget::new(planned_square).into();
                actor.direction = direction;
            }
        }
    }
}

/// NPCs close to the player are marked as ready for interaction.
///
/// This allows the player to [`begin_dialog`] as an interaction with the NPC
/// will emit the [`BeginDialogEvent`].
pub(crate) fn mark_nearby_as_ready_for_interaction(
    mut cmd: Commands,

    player: Query<&GlobalTransform, With<Player>>,
    actors: Query<(Entity, &GlobalTransform), (With<Actor>, Without<Player>)>,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };
    let player = player.translation().truncate();

    for (entity, transform) in actors.iter() {
        let distance = transform.translation().truncate().distance(player);

        if distance < 30.0 {
            cmd.entity(entity).insert(ReadyForInteraction);
        } else {
            cmd.entity(entity).remove::<ReadyForInteraction>();
        }
    }
}

/// When we start a dialog with an NPC, we insert [`BehaviorPaused`] to prevent
/// the NPC from further executing its behavior tree.
/// This also stops any movement scheduled in [`NpcInTheMap`].
///
/// This is preferable to clearing the path because:
/// 1. If we clear the path, we would need unambiguous ordering between this
///    system and the [`drive_behavior`] system. Otherwise the latter might see
///    an empty path but commands would not be applied yet, and a new path is
///    planned.
/// 2. We won't have to recompute when the dialog finishes.
pub(crate) fn begin_dialog(
    mut cmd: Commands,
    mut events: EventReader<BeginDialogEvent>,
    store: Res<GlobalStore>,

    mut player: Query<&mut Actor, With<Player>>,
    mut actors: Query<&mut Actor, Without<Player>>,
) {
    let Some(mut player) = player.get_single_mut_or_none() else {
        warn!("Cannot begin dialog without a player");
        return;
    };

    match events.read().last() {
        Some(BeginDialogEvent(ActorOrCharacter::Actor(entity))) => {
            let entity = *entity;

            let Ok(mut actor) = actors.get_mut(entity) else {
                // might've just despawned - e.g. walked away
                return;
            };

            let character = actor.character;

            let dialogs =
                store.list_dialogs_for_npc::<dialog::Namespace>(character);

            if dialogs.is_empty() {
                return;
            }

            let start_dialog = StartDialogWhenLoaded::portrait()
                .on_finished(Box::new(move |cmd: &mut Commands| {
                    trace!("Removing BehaviorPaused from {character}");
                    cmd.entity(entity).remove::<BehaviorPaused>();
                }))
                .add_namespaces(dialogs);
            cmd.insert_resource(start_dialog);

            {
                // stops the NPC from moving

                cmd.entity(entity).insert(BehaviorPaused);

                actor.remove_planned_step();
                actor.direction = actor
                    .current_square()
                    .direction_to(player.current_square())
                    .unwrap_or(actor.direction);

                player.remove_planned_step();
                player.direction = actor.direction.opposite();
            }
        }
        Some(BeginDialogEvent(ActorOrCharacter::Character(character))) => {
            let dialogs =
                store.list_dialogs_for_npc::<dialog::Namespace>(character);

            if dialogs.is_empty() {
                return;
            }

            let start_dialog =
                StartDialogWhenLoaded::portrait().add_namespaces(dialogs);
            cmd.insert_resource(start_dialog);

            player.remove_planned_step();
        }
        None => (),
    };
}

impl BehaviorLeaf {
    /// Creates a new find path leaf node.
    pub fn find_path_to(square: Square) -> Self {
        Self::FindPath {
            to: square,
            last_attempt: None,
        }
    }
}

impl BehaviorTree {
    /// Given root node, creates a new behavior tree.
    pub fn new(root: impl Into<BehaviorNode>) -> Self {
        Self {
            visiting_stack: vec![(0, root.into())],
            last_result: None,
        }
    }

    fn leaf_finished(&mut self, res: BehaviorResult) {
        trace!(
            "Leaf {:?} finished with {res:?}",
            self.visiting_stack.last()
        );
        self.visiting_stack.pop();
        self.last_result = Some(res);
    }

    /// Returns the visit number (first visit is 0) and the leaf node that can
    /// be mutated.
    /// Note that if this leaf is exited, but some node above wants to retry it,
    /// a new state will be created.
    fn unfold_into_leaf(
        &mut self,
        time: &Time,
    ) -> Option<(usize, &mut BehaviorLeaf)> {
        while let Some((visits, last_node)) = self.visiting_stack.last_mut() {
            let this_visit = *visits;
            let is_first_visit = this_visit == 0;
            let has_last_failed =
                matches!(self.last_result, Some(BehaviorResult::Failed));

            match last_node {
                BehaviorNode::Sequence(nodes) => {
                    if this_visit >= nodes.len() // all done or failed
                        || (!is_first_visit && has_last_failed)
                    {
                        self.visiting_stack.pop();
                        continue;
                    }

                    visits.add_assign(1);
                    let next_node = nodes[this_visit].clone();
                    self.visiting_stack.push((0, next_node));
                }
                BehaviorNode::Repeat(node) => {
                    if !is_first_visit && has_last_failed {
                        self.visiting_stack.pop();
                        continue;
                    }

                    visits.add_assign(1);
                    let node_to_repeat = (**node).clone();
                    self.visiting_stack.push((0, node_to_repeat));
                }
                BehaviorNode::Invert(node) => {
                    if !is_first_visit {
                        self.last_result = Some(
                            !self.last_result.expect("inverter to have result"),
                        );
                        self.visiting_stack.pop();
                        continue;
                    }

                    visits.add_assign(1);
                    let node_to_invert = (**node).clone();
                    self.visiting_stack.push((0, node_to_invert));
                }
                BehaviorNode::Infallible(node) => {
                    if !is_first_visit {
                        self.last_result = Some(BehaviorResult::Ok);
                        self.visiting_stack.pop();
                        continue;
                    }

                    visits.add_assign(1);
                    let infallible_node = (**node).clone();
                    self.visiting_stack.push((0, infallible_node));
                }

                // ~
                // we found a leaf, it's time to do some actual work
                // ~
                BehaviorNode::Leaf(_) => {
                    visits.add_assign(1);
                    break;
                }
                BehaviorNode::LeafWithTimeout(_, timer) => {
                    timer.tick(time.delta());

                    if timer.finished() {
                        self.leaf_finished(BehaviorResult::Failed);
                        continue;
                    }

                    visits.add_assign(1);
                    break;
                }
            }
        }

        match self.visiting_stack.last_mut() {
            Some((
                visit_number,
                BehaviorNode::Leaf(leaf)
                | BehaviorNode::LeafWithTimeout(leaf, _),
            )) => {
                // -1 because we already incremented
                Some((*visit_number - 1, leaf))
            }
            _ => None,
        }
    }
}

impl NpcInTheMap {
    /// Returns next planned square if any and increments the index.
    #[inline]
    fn next_planned_square(&mut self) -> Option<Square> {
        if self.planned_path.is_empty() {
            return None;
        }

        let next_square = self.planned_path[self.planned_path_index];
        self.planned_path_index += 1;

        if self.planned_path_index >= self.planned_path.len() {
            self.reset_path();
        }

        Some(next_square)
    }

    /// Removes the planned path.
    #[inline]
    pub fn reset_path(&mut self) {
        self.planned_path_index = 0;
        self.planned_path.clear();
    }
}

impl BehaviorNode {
    /// Converts this node into a boxed version.
    pub fn into_boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Not for BehaviorResult {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Ok => Self::Failed,
            Self::Failed => Self::Ok,
        }
    }
}

impl<T> From<T> for BehaviorTree
where
    T: Into<BehaviorNode>,
{
    fn from(tree: T) -> Self {
        Self::new(tree)
    }
}
