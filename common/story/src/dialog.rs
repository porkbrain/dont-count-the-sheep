//! Dialog is a cyclic directed graph of two kinds of nodes:
//! - Vocative nodes, which are the actual dialog lines.
//! - Guard nodes, which are nodes that mutate game state and serve as
//!   middleware in the dialog

mod deser;
mod guard;
mod list;

use bevy::{
    ecs::{
        reflect::ReflectResource,
        system::{Commands, Resource},
        world::World,
    },
    log::{error, trace, warn},
    reflect::Reflect,
    utils::{default, hashbrown::HashMap},
};
pub use list::DialogRoot;
use serde::{Deserialize, Serialize};

use self::guard::{GuardCmd, GuardSystem};
use crate::Character;

/// Can be used on components and resources to schedule commands.
pub type CmdFn = Box<dyn FnOnce(&mut Commands) + Send + Sync + 'static>;

/// Dialog backend.
/// It is a state machine that can be advanced.
/// It controls a fleet of guards that can read and write game state and allow
/// the dialog logic to be stateful.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Dialog {
    pub(crate) graph: DialogGraph,
    pub(crate) current_node: NodeName,
    pub(crate) branching: Branching,
    #[reflect(ignore)]
    pub(crate) guard_systems: HashMap<NodeName, GuardSystem>,
    /// When dialog is finished, run these commands.
    #[reflect(ignore)]
    pub(crate) when_finished: Vec<CmdFn>,
}

#[derive(Reflect, Debug, Default)]
pub(crate) enum Branching {
    #[default]
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

/// TODO: globally unique, must contain some kind of file name space.
#[derive(
    Debug, Deserialize, Serialize, Reflect, Clone, Hash, PartialEq, Eq,
)]
pub(crate) enum NodeName {
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
        /// TODO: give option
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
    WaitUntilNextTick,
    /// You can check the new [`Dialog::current_node`].
    Transition,
    /// The dialog won't advance until the player makes a choice.
    AwaitingPlayerChoice,
    /// The dialog is over.
    /// BE scheduled despawn of the [`Dialog`] resource and all guards.
    ScheduledDespawn,
}

impl Dialog {
    /// Schedule a command to run when the dialog is finished.
    /// As many commands as you want can be scheduled.
    pub fn on_finished(mut self, fun: CmdFn) -> Self {
        self.when_finished.push(fun);
        self
    }

    /// This method should be called by FE repeatedly until a node changes or
    /// all choice branches are evaluated.
    pub(crate) fn advance(&mut self, cmd: &mut Commands) -> AdvanceOutcome {
        match &self.current_node {
            NodeName::EndDialog => {
                self.despawn(cmd);
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

                    AdvanceOutcome::WaitUntilNextTick
                }
            },
        }
    }

    pub(crate) fn current_node_info(&self) -> &Node {
        &self.graph.nodes.get(&self.current_node).unwrap()
    }

    pub(crate) fn transition_to(
        &mut self,
        cmd: &mut Commands,
        node_name: NodeName,
    ) {
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

    pub(crate) fn spawn(self, cmd: &mut Commands) {
        cmd.insert_resource(self);
    }

    pub(crate) fn despawn(&mut self, cmd: &mut Commands) {
        cmd.remove_resource::<Self>();

        for (node_name, guard_system) in self.guard_systems.drain() {
            cmd.run_system_with_input(
                guard_system.entity,
                GuardCmd::Despawn(node_name.clone()),
            );
            let system_id = guard_system.entity;
            cmd.add(move |w: &mut World| {
                if let Err(e) = w.remove_system(system_id) {
                    error!("Error removing guard system: {e:?}");
                }
            });
        }

        for fun in self.when_finished.drain(..) {
            fun(cmd);
        }
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
                    return AdvanceOutcome::WaitUntilNextTick;
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
        let next_node = &dialog.graph.nodes.get(node_name).unwrap();
        assert_eq!(Character::Winnie, next_node.who);

        match &next_node.kind {
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

impl DialogGraph {
    /// Create a new dialog resource.
    /// It can then be associated with a FE to spawn the dialog.
    pub fn into_dialog_resource(self) -> Dialog {
        self.into()
    }
}

impl std::fmt::Debug for Dialog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialog")
            .field("graph", &self.graph)
            .field("guard_systems", &self.guard_systems)
            .field("current_node", &self.current_node)
            .field("branching", &self.branching)
            .finish()
    }
}

impl From<DialogGraph> for Dialog {
    fn from(graph: DialogGraph) -> Self {
        Self {
            current_node: graph.root.clone(),
            graph,
            guard_systems: default(),
            branching: default(),
            when_finished: default(),
        }
    }
}

impl Default for Dialog {
    /// Implemented because we want to reflect resource in Bevy.
    fn default() -> Self {
        Self {
            graph: DialogGraph {
                nodes: {
                    let mut node_map = HashMap::default();
                    node_map.insert(
                        NodeName::EndDialog,
                        Node {
                            who: Character::Winnie,
                            name: NodeName::EndDialog,
                            kind: NodeKind::Guard {
                                kind: GuardKind::EndDialog,
                                params: default(),
                            },
                            next: Vec::new(),
                        },
                    );
                    node_map
                },
                root: NodeName::EndDialog,
            },
            guard_systems: default(),
            current_node: NodeName::EndDialog,
            branching: default(),
            when_finished: default(),
        }
    }
}
