use bevy::{
    ecs::{
        system::{Commands, In, Local, Res, ResMut, SystemId},
        world::World,
    },
    log::warn,
    reflect::Reflect,
};
use common_store::{DialogStore, GlobalStore};

use super::{Dialog, GuardKind, NextNodeStatus, NextNodes, NodeKind, NodeName};

#[derive(Reflect, Debug)]
pub(crate) struct GuardSystem {
    pub(crate) entity: SystemId<GuardCmd>,
}

impl GuardKind {
    pub(crate) fn register_system(
        self,
        cmd: &mut Commands,
        node_name: NodeName,
    ) {
        cmd.add(move |w: &mut World| {
            let entity = match self {
                Self::ExhaustiveAlternatives => {
                    w.register_system(exhaustive_alternatives)
                }
                _ => todo!(),
            };
            if let Some(mut dialog) = w.get_resource_mut::<Dialog>() {
                dialog
                    .guard_systems
                    .insert(node_name, GuardSystem { entity });
            } else {
                warn!(
                    "Trying to add a guard {self} to a \
                    world without Dialog resource"
                );
            }
        });
    }
}
pub(crate) enum GuardCmd {
    /// Will change the current node of the dialog is ready.
    ///
    /// For guard with async ops, such as displaying UI with animations,
    /// this command might not result in transition.
    TryTransition(NodeName),
    /// We want to show player choices in dialog.
    /// This command says: in the [`Dialog::next_nodes`] array, at the
    /// specified index, give us string that we should show to the player
    /// as a choice.
    /// It's possible that the guard will decide to stop the current branch
    /// with [`NextNode::Stop`].
    PlayerChoice(NodeName),
    /// The dialog is being despawned, save the state if necessary.
    Despawn(NodeName),
}

fn exhaustive_alternatives(
    In(guard_cmd): In<GuardCmd>,
    mut state: Local<Option<usize>>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
) {
    if state.is_none() {
        match &guard_cmd {
            GuardCmd::TryTransition(NodeName::Explicit(node_name))
            | GuardCmd::PlayerChoice(NodeName::Explicit(node_name)) => {
                let from_store = store
                    .guard_state(GuardKind::ExhaustiveAlternatives, node_name)
                    .get()
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                state.replace(from_store.unwrap_or(0))
            }
            _ => state.replace(0),
        };
    }
    let state = state.as_mut().unwrap();

    match guard_cmd {
        GuardCmd::TryTransition(node_name) => {
            debug_assert_eq!(node_name, dialog.current_node);

            let next_node = dialog
                .graph
                .nodes
                .get(&node_name)
                .unwrap()
                .next // get next nodes
                .get(*state) // which one is next to show (if any)
                .cloned()
                .inspect(|_| *state += 1) // next time show the next one
                .unwrap_or(NodeName::Emerge); // all shown, stop
            dialog.transition_to(&mut cmd, next_node);
        }
        GuardCmd::PlayerChoice(node_name) => {
            let next_node_choice = dialog
                .graph
                .nodes
                .get(&node_name)
                .unwrap()
                .next // get next nodes
                .get(*state) // which one is next to show (if any)
                .map(|next_node_name| {
                    let next_node_kind =
                        &dialog.graph.nodes.get(next_node_name).unwrap().kind;
                    if let NodeKind::Vocative { line } = &next_node_kind {
                        // TODO: perhaps another property for choice
                        NextNodeStatus::OfferAsChoice(line.clone())
                    } else {
                        panic!(
                            "Expected vocative node, got \
                            {next_node_kind:?} ({next_node_name:?})"
                        )
                    }
                })
                .unwrap_or(NextNodeStatus::Stop); // all shown, stop offering this

            if let NextNodes::Choice(next_nodes) = &mut dialog.next_nodes {
                next_nodes
                    .iter_mut()
                    .find(|(name, _)| name == &node_name)
                    .unwrap()
                    .1 = next_node_choice;
            }
        }
        GuardCmd::Despawn(NodeName::Explicit(node_name)) => {
            store
                .guard_state(GuardKind::ExhaustiveAlternatives, node_name)
                .set((*state).into());
        }
        GuardCmd::Despawn(_) => {
            //
        }
    }
}
