use super::*;

pub(super) fn system(
    In(guard_cmd): In<GuardCmd>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
    notifications: ResMut<NotificationFifo>,
) {
    match guard_cmd {
        GuardCmd::TryTransition(node_name) => {
            debug_assert_eq!(node_name, dialog.current_node);

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
