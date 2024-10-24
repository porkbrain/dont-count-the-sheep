use bevy::log::trace;

use super::*;

const KIND: GuardKind = GuardKind::VisitOnce;

pub(super) fn system(
    In(guard_cmd): In<GuardCmd>,
    mut state: Local<Option<bool>>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
) {
    let state = state.get_or_insert_with(|| {
        let state = KIND.load_state(&store, &guard_cmd);
        trace!("Loading state: {state}");
        state
    });

    match guard_cmd {
        GuardCmd::TryTransition(node_name) => {
            debug_assert_eq!(node_name, dialog.current_node);
            let next_nodes = &dialog.graph.nodes.get(&node_name).unwrap().next;
            debug_assert_eq!(
                next_nodes.len(),
                1,
                "visit_once guard can only have one next node"
            );

            let next_node = next_nodes
                .first()
                .cloned()
                .inspect(|_| {
                    trace!("Next time branch won't show");
                    *state = true
                })
                .expect("visit_once must have one next node");
            dialog.transition_to(&mut cmd, &store, next_node);
        }
        GuardCmd::PlayerChoice {
            node_name,
            next_branch_index,
        } => {
            trace!("Player choice with state: {state}");
            let next_node_choice = if *state {
                BranchStatus::Stop
            } else {
                let next_nodes =
                    &dialog.graph.nodes.get(&node_name).unwrap().next; // get next nodes
                debug_assert_eq!(
                    next_nodes.len(),
                    1,
                    "visit_once guard can only have one next node"
                );

                let next_node_name = next_nodes.first().unwrap();

                let next_node_kind =
                    &dialog.graph.nodes.get(next_node_name).unwrap().kind;

                match next_node_kind {
                    NodeKind::Blank => BranchStatus::Stop,
                    NodeKind::Vocative { line } => {
                        BranchStatus::OfferAsChoice(line.clone())
                    }
                    NodeKind::Guard { .. } => {
                        // evaluate next guard
                        cmd.add(GuardCmd::PlayerChoice {
                            node_name: next_node_name.clone(),
                            next_branch_index,
                        });
                        return;
                    }
                }
            };

            if let Branching::Choice(branches) = &mut dialog.branching {
                branches[next_branch_index] = next_node_choice;
            };
        }
        GuardCmd::Despawn(NodeName::Explicit(namespace, node_name)) => {
            trace!("Storing state for {namespace}::{node_name}: {state}");
            store
                .guard_state(KIND, (namespace, node_name))
                .set((*state).into());
        }
        GuardCmd::Despawn(_) => {
            //
        }
    }
}
