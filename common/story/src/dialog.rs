//! Dialog is a cyclic directed graph of two kinds of nodes:
//! - Vocative nodes, which are the actual dialog lines.
//! - Guard nodes, which are nodes that mutate game state and serve as
//!   middleware in the dialog

mod deser;
mod guard;
mod list;

use bevy::{
    ecs::system::{Commands, Resource},
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
    pub(crate) graph: DialogGraph,
    pub(crate) guard_systems: HashMap<NodeName, GuardSystem>,
    pub(crate) current_node: NodeName,
    pub(crate) branching: Branching,
}

#[derive(Reflect, Debug)]
pub(crate) enum Branching {
    None,
    Single(NodeName),
    Choice(Vec<BranchStatus>),
}

#[derive(Reflect, Debug)]
pub(crate) enum BranchStatus {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum AdvanceOutcome {
    /// Some operations are still pending, try again later.
    /// This will happen with guards.
    /// Most guards will be ready in the next tick, but an async guard could
    /// potentially take as much time as it needs for e.g. a UI animation.
    ///
    /// # Important
    /// Don't call [`Dialog::advance`] before verifying that the current node
    /// has indeed not changed.
    CheckAgainAfterApplyDeferred,
    /// You can check the new [`Dialog::current_node`].
    Transition,
    /// The dialog won't advance until the player makes a choice.
    AwaitingPlayerChoice,
    /// The dialog is over.
    /// BE scheduled despawn of the [`Dialog`] resource and all guards.
    ScheduledDespawn,
}

impl Dialog {
    /// This method should be called by FE repeatedly until a node changes or
    /// all choice branches are evaluated.
    pub(crate) fn advance(&mut self, cmd: &mut Commands) -> AdvanceOutcome {
        match &self.current_node {
            NodeName::EndDialog => {
                todo!();
                AdvanceOutcome::ScheduledDespawn
            }
            NodeName::Emerge => {
                todo!()
            }
            node_name => match &self.graph.nodes.get(node_name).unwrap().kind {
                NodeKind::Vocative { .. } => {
                    self.transition_or_offer_player_choice_if_all_ready(cmd)
                }
                NodeKind::Guard { kind, .. } => {
                    if let Some(guard_system) =
                        self.guard_systems.get(node_name)
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
                        kind.register_system(cmd, node_name.clone());
                        cmd.add(GuardCmd::TryTransition(node_name.clone()));
                    }

                    AdvanceOutcome::CheckAgainAfterApplyDeferred
                }
            },
        }
    }

    pub(crate) fn current_node_info(&self) -> &Node {
        &self.graph.nodes.get(&self.current_node).unwrap()
    }

    fn transition_to(&mut self, cmd: &mut Commands, node_name: NodeName) {
        let next_nodes = &self.graph.nodes.get(&node_name).unwrap().next;

        self.current_node = node_name;
        self.branching = if next_nodes.is_empty() {
            Branching::None
        } else if next_nodes.len() == 1 {
            Branching::Single(next_nodes[0].clone())
        } else {
            Branching::Choice(
                next_nodes
                    .iter()
                    .enumerate()
                    .map(|(next_branch_index, next_node_name)| {
                        BranchStatus::new(
                            &self,
                            cmd,
                            next_branch_index,
                            next_node_name,
                        )
                    })
                    .collect(),
            )
        };
    }

    fn transition_or_offer_player_choice_if_all_ready(
        &mut self,
        cmd: &mut Commands,
    ) -> AdvanceOutcome {
        match &self.branching {
            Branching::None => {
                warn!("NextNodes::None, emerging");
                self.transition_to(cmd, NodeName::Emerge);
                AdvanceOutcome::Transition
            }
            Branching::Single(next_node) => {
                self.transition_to(cmd, next_node.clone());
                AdvanceOutcome::Transition
            }
            Branching::Choice(branches) => {
                let any_pending = branches
                    .iter()
                    .any(|status| matches!(status, BranchStatus::Pending));

                if any_pending {
                    // must be re-evaluated again next tick
                    return AdvanceOutcome::CheckAgainAfterApplyDeferred;
                }

                let mut choices = branches.iter().enumerate().filter_map(
                    |(branch_index, status)| match status {
                        BranchStatus::OfferAsChoice(text) => {
                            Some((branch_index, text))
                        }
                        BranchStatus::Stop => None,
                        BranchStatus::Pending => unreachable!(),
                    },
                );

                if let Some((first_choice_branch_index, _)) = choices.next() {
                    if choices.next().is_none() {
                        let first_choice_node_name = self
                            .graph
                            .nodes
                            .get(&self.current_node)
                            .unwrap()
                            .next
                            .get(first_choice_branch_index)
                            .unwrap()
                            .clone();
                        self.transition_to(cmd, first_choice_node_name);
                        AdvanceOutcome::Transition
                    } else {
                        AdvanceOutcome::AwaitingPlayerChoice
                    }
                } else {
                    warn!("NextNodes::Choice stopped all branches, emerging");
                    self.transition_to(cmd, NodeName::Emerge);
                    AdvanceOutcome::Transition
                }
            }
        }
    }
}

impl BranchStatus {
    fn new(
        dialog: &Dialog,
        cmd: &mut Commands,
        branch_index: usize,
        node_name: &NodeName,
    ) -> Self {
        let next_node_kind = &dialog.graph.nodes.get(node_name).unwrap().kind;

        match next_node_kind {
            NodeKind::Vocative { line } => {
                //  TODO: this node is ready to be shown as an option
                Self::OfferAsChoice(line.clone())
            }
            NodeKind::Guard { kind, .. } => {
                if let Some(guard_system) = dialog.guard_systems.get(node_name)
                {
                    cmd.run_system_with_input(
                        guard_system.entity,
                        GuardCmd::PlayerChoice {
                            node_name: node_name.clone(),
                            next_branch_index: branch_index,
                        },
                    )
                } else {
                    trace!(
                        "Registering guard system {kind:?} for {node_name:?}"
                    );
                    kind.register_system(cmd, node_name.clone());
                    cmd.add(GuardCmd::PlayerChoice {
                        node_name: node_name.clone(),
                        next_branch_index: branch_index,
                    });
                }

                // we need to evaluate the guard
                Self::Pending
            }
        }
    }
}
