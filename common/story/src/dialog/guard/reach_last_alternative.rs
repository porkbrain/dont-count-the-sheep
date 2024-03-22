use super::*;

const KIND: GuardKind = GuardKind::ReachLastAlternative;

pub(super) fn system(
    In(guard_cmd): In<GuardCmd>,
    mut state: Local<Option<usize>>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
) {
    // TODO: dedup
    if state.is_none() {
        match &guard_cmd {
            GuardCmd::TryTransition(NodeName::Explicit(
                namespace,
                node_name,
            ))
            | GuardCmd::PlayerChoice {
                node_name: NodeName::Explicit(namespace, node_name),
                ..
            } => {
                let from_store = store
                    .guard_state(KIND, (namespace, node_name))
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

            let next_nodes = &dialog.graph.nodes.get(&node_name).unwrap().next;

            let next_node = next_nodes
                .get(*state) // which one is next to show (if any)
                .cloned()
                .inspect(|_| *state += 1) // next time show the next one
                // keep showing the last one
                .unwrap_or_else(|| next_nodes.last().unwrap().clone());
            dialog.transition_to(&mut cmd, &store, next_node);
        }
        GuardCmd::PlayerChoice {
            node_name,
            next_branch_index,
        } => {
            let next_nodes = &dialog.graph.nodes.get(&node_name).unwrap().next;

            let next_node_name = next_nodes
                .get(*state)
                .unwrap_or_else(|| next_nodes.last().unwrap());
            let next_node_kind =
                &dialog.graph.nodes.get(next_node_name).unwrap().kind;

            let next_node_choice = match next_node_kind {
                NodeKind::Blank => BranchStatus::Stop,
                NodeKind::Vocative { line } => {
                    // TODO: perhaps another property for choice
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
            };

            if let Branching::Choice(branches) = &mut dialog.branching {
                branches[next_branch_index] = next_node_choice;
            };
        }
        GuardCmd::Despawn(NodeName::Explicit(namespace, node_name)) => {
            store
                .guard_state(KIND, (namespace, node_name))
                .set((*state).into());
        }
        GuardCmd::Despawn(_) => {
            //
        }
    }
}
