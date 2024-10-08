use std::str::FromStr;

use common_story::Character;

use super::*;
use crate::dialog::list::Namespace;

pub(super) fn add(
    In(guard_cmd): In<GuardCmd>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
) {
    handle_guard_cmd_as(
        guard_cmd,
        GuardKind::AddDialogToNpc,
        // ...
        &mut cmd,
        &mut dialog,
        &store,
    );
}

pub(super) fn remove(
    In(guard_cmd): In<GuardCmd>,

    mut cmd: Commands,
    mut dialog: ResMut<Dialog>,
    store: Res<GlobalStore>,
) {
    handle_guard_cmd_as(
        guard_cmd,
        GuardKind::RemoveDialogFromNpc,
        // ...
        &mut cmd,
        &mut dialog,
        &store,
    );
}

fn handle_guard_cmd_as(
    guard_cmd: GuardCmd,
    kind: GuardKind,
    cmd: &mut Commands,
    dialog: &mut Dialog,
    store: &GlobalStore,
) {
    match guard_cmd {
        GuardCmd::TryTransition(node_name) => {
            debug_assert_eq!(node_name, dialog.current_node);
            let node = &dialog.graph.nodes.get(&node_name).unwrap();

            let NodeKind::Guard { params, .. } = &node.kind else {
                panic!(
                    "{node_name:?} ({kind}): Expected node kind to be Guard"
                );
            };

            let npc = params
                .get("npc")
                .and_then(|v| v.as_str())
                .map(|npc| Character::from_str(npc).expect("NPC doesn't exist"))
                .unwrap_or(node.who);
            debug_assert_ne!(Character::Winnie, npc);

            let namespace = params
                .get("file_path")
                .and_then(|file_path| file_path.as_str())
                .map(|s| Namespace::from(s.to_owned()))
                .unwrap_or_else(|| {
                    panic!("No file_path string in params {params:#?}")
                });

            match kind {
                GuardKind::AddDialogToNpc => {
                    store.add_dialog_to_npc(npc, namespace);
                }
                GuardKind::RemoveDialogFromNpc => {
                    store.remove_dialog_from_npc(npc, namespace);
                }
                _ => unreachable!(),
            }

            let next_nodes = &node.next;
            assert_eq!(1, next_nodes.len());
            let next_node_name = next_nodes[0].clone();
            dialog.transition_to(cmd, store, next_node_name);
        }
        GuardCmd::PlayerChoice {
            node_name,
            next_branch_index,
        } => dialog.pass_guard_player_choice(cmd, node_name, next_branch_index),
        GuardCmd::Despawn(_) => {
            //
        }
    }
}
