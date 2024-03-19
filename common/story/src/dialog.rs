//! Dialog is a cyclic directed graph of two kinds of nodes:
//! - Vocative nodes, which are the actual dialog lines.
//! - Guard nodes, which are nodes that mutate game state and serve as
//!   middleware in the dialog

mod deser;
mod guard;
mod list;

use bevy::{
    ecs::system::{Commands, ResMut, Resource},
    log::{trace, warn},
    reflect::Reflect,
    utils::hashbrown::HashMap,
};
pub use list::DialogRoot;
use serde::{Deserialize, Serialize};

use self::guard::{GuardCmd, GuardSystem};
use crate::Character;

// user option if dialog has more than 1 next node and the who is Winnie
// TODO: validate that if an option all who are winnie

/// TODO
#[derive(Resource, Reflect, Debug)]
pub struct Dialog {
    graph: DialogGraph,
    guard_systems: HashMap<NodeName, GuardSystem>,
    current_node: NodeName,
    next_nodes: NextNodes,
}

#[derive(Reflect, Debug)]
enum NextNodes {
    None,
    Single(NodeName),
    Choice(Vec<(NodeName, NextNodeStatus)>),
}

#[derive(Reflect, Debug)]
enum NextNodeStatus {
    Pending,
    OfferAsChoice(String),
    /// This branch is exhausted, presumably some guard decided to stop it.
    Stop,
}

/// The dialog asset that can be started.
/// Since dialogs can be stateful, state is lazy loaded.
#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct DialogGraph {
    pub(crate) nodes: HashMap<NodeName, Node>,
    pub(crate) root: NodeName,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub(crate) struct Node {
    pub(crate) name: NodeName,
    pub(crate) who: Character,
    pub(crate) kind: NodeKind,
    pub(crate) next: Vec<NodeName>,
}

#[derive(
    Debug, Deserialize, Serialize, Reflect, Clone, Hash, PartialEq, Eq,
)]
pub(crate) enum NodeName {
    // todo: globally unique
    // TODO: we could store just the hash of the string if we wanted to allow
    // copy, but is worse for debugging
    Explicit(String),
    Auto(usize),
    EndDialog,
    Emerge,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub(crate) enum NodeKind {
    Guard {
        /// Guard states are persisted across dialog sessions if
        /// - the node has a [`NodeName::Explicit`]
        ///
        /// Otherwise the state is discarded after the dialog is over.
        kind: GuardKind,
        #[reflect(ignore)]
        params: HashMap<String, toml::Value>,
    },
    Vocative {
        /// The dialog line to print.
        line: String,
    },
}

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Reflect,
    strum::EnumString,
    strum::Display,
    Clone,
    Copy,
)]
pub(crate) enum GuardKind {
    EndDialog,
    Emerge,
    ExhaustiveAlternatives,
    ReachLastAlternative,
}

fn advance(mut cmd: Commands, mut dialog: ResMut<Dialog>) {
    // TODO: finish rendering all text

    match &dialog.current_node {
        NodeName::EndDialog => {
            todo!()
        }
        NodeName::Emerge => {
            todo!()
        }
        node_name => {
            let node = dialog.graph.nodes.get(node_name).unwrap();

            match &node.kind {
                NodeKind::Vocative { .. } => {
                    dialog.try_transition_or_offer_player_choice(&mut cmd);
                }
                NodeKind::Guard { kind, .. } => {
                    if let Some(guard_system) =
                        dialog.guard_systems.get(&node.name)
                    {
                        cmd.run_system_with_input(
                            guard_system.entity,
                            GuardCmd::TryTransition(node_name.clone()),
                        );
                    } else {
                        trace!(
                            "Registering guard system {kind:?} \
                            for node {node_name:?}"
                        );
                        kind.register_system(&mut cmd, node_name.clone());
                        // TODO: perhaps we can actually schedule transition
                        // here
                    }
                }
            }
        }
    }
}

impl From<String> for NodeName {
    fn from(s: String) -> Self {
        match s.as_str() {
            "_end_dialog" => NodeName::EndDialog,
            "_emerge" => NodeName::Emerge,
            _ => NodeName::Explicit(s),
        }
    }
}

impl Dialog {
    fn transition_to(&mut self, cmd: &mut Commands, node_name: NodeName) {
        let next_nodes = &self.graph.nodes.get(&node_name).unwrap().next;

        self.next_nodes = if next_nodes.is_empty() {
            NextNodes::None
        } else if next_nodes.len() == 1 {
            NextNodes::Single(next_nodes[0].clone())
        } else {
            NextNodes::Choice(
                next_nodes
                    .iter()
                    .map(|next_node_name| {
                        let status =
                            NextNodeStatus::new(&self, cmd, next_node_name);

                        (next_node_name.clone(), status)
                    })
                    .collect(),
            )
        };

        self.current_node = node_name;
    }

    fn try_transition_or_offer_player_choice(&mut self, cmd: &mut Commands) {
        match &self.next_nodes {
            NextNodes::None => {
                warn!("NextNodes::None, emerging");
                self.transition_to(cmd, NodeName::Emerge);
            }
            NextNodes::Single(next_node) => {
                self.transition_to(cmd, next_node.clone());
            }
            NextNodes::Choice(next_nodes) => {
                let any_pending = next_nodes.iter().any(|(_, status)| {
                    matches!(status, NextNodeStatus::Pending)
                });

                if any_pending {
                    return;
                }

                let mut choices =
                    next_nodes.iter().filter_map(|(node_name, status)| {
                        match status {
                            NextNodeStatus::OfferAsChoice(text) => {
                                Some((node_name, text))
                            }
                            NextNodeStatus::Stop => None,
                            NextNodeStatus::Pending => unreachable!(),
                        }
                    });

                if let Some((first_choice_node_name, first_choice_text)) =
                    choices.next()
                {
                    if let Some(second_choice) = choices.next() {
                        todo!()
                    } else {
                        self.transition_to(cmd, first_choice_node_name.clone())
                    }
                } else {
                    warn!("NextNodes::Choice stopped all branches, emerging");
                    self.transition_to(cmd, NodeName::Emerge);
                }
            }
        }
    }
}

impl NextNodeStatus {
    fn new(
        dialog: &Dialog,
        cmd: &mut Commands,
        next_node_name: &NodeName,
    ) -> Self {
        let next_node_kind =
            &dialog.graph.nodes.get(next_node_name).unwrap().kind;

        match next_node_kind {
            NodeKind::Vocative { line } => {
                //  TODO: this node is ready to be shown as an option
                Self::OfferAsChoice(line.clone())
            }
            NodeKind::Guard { kind, .. } => {
                if let Some(guard_system) =
                    dialog.guard_systems.get(next_node_name)
                {
                    cmd.run_system_with_input(
                        guard_system.entity,
                        GuardCmd::PlayerChoice(next_node_name.clone()),
                    )
                } else {
                    trace!(
                        "Registering guard system {kind:?} \
                        node {next_node_name:?}"
                    );
                    kind.register_system(cmd, next_node_name.clone());
                    // TODO: schedule player choice cmd
                }

                // we need to evaluate the guard
                Self::Pending
            }
        }
    }
}
