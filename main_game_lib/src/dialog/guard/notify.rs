use super::*;
use crate::hud::notification::{Notification, NotificationFifo};

const KIND: GuardKind = GuardKind::Notify;

pub(super) fn system(
    In(guard_cmd): In<GuardCmd>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
    mut notifications: ResMut<NotificationFifo>,
) {
    match guard_cmd {
        GuardCmd::TryTransition(node_name) => {
            debug_assert_eq!(node_name, dialog.current_node);

            let node = &dialog.graph.nodes.get(&node_name).unwrap();

            let NodeKind::Guard { params, .. } = &node.kind else {
                panic!(
                    "{node_name:?} ({KIND}): Expected node kind to be Guard"
                );
            };

            let message = params
                .get("message")
                .unwrap_or_else(|| {
                    panic!("{node_name:?} ({KIND}): Expected 'message' param")
                })
                .as_str()
                .expect("Expected 'message' param to be a string")
                .to_string();
            notifications.push(Notification::PlainText(message));

            let next_nodes = &dialog
                .graph
                .nodes
                .get(&node_name)
                .unwrap_or_else(|| {
                    panic!("Node {node_name:?} not found in graph")
                })
                .next;
            assert_eq!(
                1,
                next_nodes.len(),
                "Expected one next node for notify guard {node_name:?}"
            );
            let next_node = next_nodes.first().unwrap().clone();

            dialog.transition_to(&mut cmd, &store, next_node);
        }
        GuardCmd::PlayerChoice {
            node_name,
            next_branch_index,
        } => dialog.pass_guard_player_choice(
            &mut cmd,
            node_name,
            next_branch_index,
        ),
        GuardCmd::Despawn(_) => {
            //
        }
    }
}
