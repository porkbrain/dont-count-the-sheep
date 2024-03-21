//! Dialog is a cyclic directed graph of two kinds of nodes:
//! - Vocative nodes, which are the actual dialog lines.
//! - Guard nodes, which are nodes that mutate game state and serve as
//!   middleware in the dialog
//!
//! This module exports a backend that can be used to implement dialog in a
//! game. It has no systems, only a resource that when coupled with a frontend
//! advances the dialog state.

mod deser;
mod guard;
mod list;

use bevy::{
    ecs::{
        reflect::ReflectResource,
        system::{CommandQueue, Commands, Resource},
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
    // namespace -> DialogGraph
    pub(crate) graph: DialogGraph,
    // GlobalNodeName
    current_node: LocalNodeName,
    pub(crate) branching: Branching,
    #[reflect(ignore)]
    // GlobalNodeName -> GuardSystem
    pub(crate) guard_systems: HashMap<LocalNodeName, GuardSystem>,
    /// When dialog is finished, run these commands.
    #[reflect(ignore)]
    pub(crate) when_finished: Vec<CmdFn>,
}

#[derive(Reflect, Debug, Default)]
pub(crate) enum Branching {
    #[default]
    None,
    // GlobalNodeName
    Single(LocalNodeName),
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
#[derive(Debug, Reflect)]
pub struct DialogGraph {
    pub(crate) nodes: HashMap<LocalNodeName, Node>,
}

#[derive(Debug, Reflect)]
pub(crate) struct Node {
    pub(crate) name: LocalNodeName,
    pub(crate) who: Character,
    pub(crate) kind: NodeKind,
    pub(crate) next: Vec<LocalNodeName>,
}

/// TODO: globally unique, must contain some kind of file name space.
#[derive(Debug, Reflect, Clone, Hash, PartialEq, Eq)]
pub(crate) enum GlobalNodeName {
    Explicit(&'static str, String),
    Auto(&'static str, usize),
    NamespaceRoot(&'static str),
    Root,
    EndDialog,
    Emerge,
}

/// [`DialogGraph`] local.
/// Does not contain namespace info.
#[derive(Debug, Reflect, Clone, Hash, PartialEq, Eq)]
pub(crate) enum LocalNodeName {
    /// Matches [`GlobalNodeName::Explicit`] except the namespace.
    Explicit(String),
    /// Matches [`GlobalNodeName::Auto`] except the namespace.
    Auto(usize),
    /// Matches [`GlobalNodeName::NamespaceRoot`] except the namespace.
    Root,
    EndDialog,
    Emerge,
}

#[derive(Debug, Reflect)]
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

    pub(crate) fn current_node(&self) -> &LocalNodeName {
        &self.current_node
    }

    /// This method should be called by FE repeatedly until a node changes or
    /// all choice branches are evaluated.
    pub(crate) fn advance(&mut self, cmd: &mut Commands) -> AdvanceOutcome {
        match &self.current_node {
            LocalNodeName::EndDialog => {
                self.despawn(cmd);
                AdvanceOutcome::ScheduledDespawn
            }
            LocalNodeName::Emerge => {
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
                        cmd.add(kind.register_system_cmd(node_name.clone()));
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
        node_name: LocalNodeName,
    ) {
        if let LocalNodeName::Explicit(node_name) = &self.current_node {
            //
        }

        self.branching =
            Branching::new(cmd, &node_name, &self.graph, &self.guard_systems);
        self.current_node = node_name;
    }

    pub(crate) fn spawn(self, cmd: &mut Commands) {
        cmd.insert_resource(self);
    }

    pub(crate) fn despawn(&mut self, cmd: &mut Commands) {
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

        // must be added last because guards depend on this resource
        cmd.remove_resource::<Self>();
    }

    fn transition_or_offer_player_choice_if_all_ready(
        &mut self,
        cmd: &mut Commands,
    ) -> AdvanceOutcome {
        match &self.branching {
            Branching::None => {
                warn!("NextNodes::None, emerging");
                self.transition_to(cmd, LocalNodeName::Emerge);
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
                    self.transition_to(cmd, LocalNodeName::Emerge);
                    AdvanceOutcome::Transition
                }
            }
        }
    }
}

impl Branching {
    fn new(
        cmd: &mut Commands,
        from: &LocalNodeName,
        graph: &DialogGraph,
        guard_systems: &HashMap<LocalNodeName, GuardSystem>,
    ) -> Self {
        let next_nodes = &graph.nodes.get(from).unwrap().next;
        trace!("Branching for {from:?}: {next_nodes:?}");

        if next_nodes.is_empty() {
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
                            cmd,
                            graph,
                            guard_systems,
                            next_branch_index,
                            next_node_name,
                        )
                    })
                    .collect(),
            )
        }
    }

    /// This method can be only used when the [`Dialog`] resource is not yet
    /// inserted.
    /// It's used to init the guards.
    /// Once in dialog, use [`Branching::new`] instead.
    /// That method uses the guard cache to avoid spawning the same guard twice.
    fn init(cmd: &mut CommandQueue, graph: &DialogGraph) -> Self {
        let next_nodes = &graph.nodes.get(&LocalNodeName::Root).unwrap().next;
        trace!("Branching from root: {next_nodes:?}");

        if next_nodes.is_empty() {
            Branching::None
        } else if next_nodes.len() == 1 {
            Branching::Single(next_nodes[0].clone())
        } else {
            Branching::Choice(
                next_nodes
                    .iter()
                    .enumerate()
                    .map(|(next_branch_index, next_node_name)| {
                        BranchStatus::init(
                            cmd,
                            graph,
                            next_branch_index,
                            next_node_name,
                        )
                    })
                    .collect(),
            )
        }
    }
}

impl BranchStatus {
    fn new(
        cmd: &mut Commands,
        graph: &DialogGraph,
        guard_systems: &HashMap<LocalNodeName, GuardSystem>,
        branch_index: usize,
        node_name: &LocalNodeName,
    ) -> Self {
        let next_node = &graph.nodes.get(node_name).unwrap();
        assert_eq!(
            Character::Winnie,
            next_node.who,
            "Only Winnie can branch ({:?})",
            next_node
        );

        match &next_node.kind {
            NodeKind::Vocative { line } => {
                //  TODO: this node is ready to be shown as an option
                Self::OfferAsChoice(line.clone())
            }
            NodeKind::Guard { kind, .. } => {
                if let Some(guard_system) = guard_systems.get(node_name) {
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
                    cmd.add(kind.register_system_cmd(node_name.clone()));
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

    /// See [`Branching::init`]
    fn init(
        cmd: &mut CommandQueue,
        graph: &DialogGraph,
        branch_index: usize,
        node_name: &LocalNodeName,
    ) -> Self {
        let next_node = &graph.nodes.get(node_name).unwrap();
        assert_eq!(
            Character::Winnie,
            next_node.who,
            "Only Winnie can branch ({:?})",
            next_node
        );

        match &next_node.kind {
            NodeKind::Vocative { line } => {
                //  TODO: this node is ready to be shown as an option
                Self::OfferAsChoice(line.clone())
            }
            NodeKind::Guard { kind, .. } => {
                trace!("Registering guard system {kind:?} for {node_name:?}");
                cmd.push(kind.register_system_cmd(node_name.clone()));
                cmd.push(GuardCmd::PlayerChoice {
                    node_name: node_name.clone(),
                    next_branch_index: branch_index,
                });

                // we need to evaluate the guard
                Self::Pending
            }
        }
    }
}

impl DialogGraph {
    /// Create a new dialog resource.
    /// It can then be associated with a FE to spawn the dialog.
    ///
    /// We accept command queue instead of commands because the guards spawned
    /// by this method depend on the [`Dialog`] resource.
    /// Therefore, the [`Dialog`] resource must be inserted before the guards
    /// are spawned.
    /// So apply the provided queue after inserting the [`Dialog`] resource
    /// with relevant FE method.
    pub fn into_dialog_resource(self, cmd: &mut CommandQueue) -> Dialog {
        let branching = Branching::init(cmd, &self);
        Dialog {
            current_node: LocalNodeName::Root,
            graph: self,
            guard_systems: default(),
            branching,
            when_finished: default(),
        }
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

impl Default for Dialog {
    /// Implemented because we want to reflect resource in Bevy.
    fn default() -> Self {
        Self {
            graph: DialogGraph { nodes: default() },
            guard_systems: default(),
            current_node: LocalNodeName::Root,
            branching: default(),
            when_finished: default(),
        }
    }
}
