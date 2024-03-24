//! Dialog is a cyclic directed graph of two kinds of nodes:
//! - Vocative nodes, which are the actual dialog lines.
//! - Guard nodes, which are nodes that mutate game state and serve as
//!   middleware in the dialog
//!
//! This module exports a backend that can be used to implement dialog in a
//! game.
//! It has no systems, only a resource that when coupled with a frontend
//! advances the dialog state.
//! See the [`fe`] module for frontends.
//!
//! Then there's the [`DialogRoot`] enum.
//! It's auto generated from all dialog .toml files.
//! Each file is represented as a variant of this enum by taking the file stem
//! and converting it to PascalCase.
//! You can use the specific variant to spawn dialog BE and FE which starts the
//! dialog.

mod deser;
pub mod fe;
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
use common_store::{DialogStore, GlobalStore};
pub use list::DialogRoot;

use self::guard::{GuardCmd, GuardKind, GuardSystem};
use crate::Character;

/// Use [`Dialog::on_finished`] to schedule commands to run when the dialog is
/// finished.
pub type CmdFn = Box<dyn FnOnce(&mut Commands) + Send + Sync + 'static>;

/// Namespace represents a dialog toml file with relative path from the root
/// of the dialog directory.
/// Each dialog file has a unique name.
pub type Namespace = DialogRoot;

/// Dialog backend.
///
/// It is a state machine that can be advanced.
/// It controls a fleet of guards that can read and write game state and allow
/// the dialog logic to be stateful.
///
/// Spawn it via [`DialogRoot`], associate it with a frontend (see [`fe`]
/// module.)
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Dialog {
    graph: DialogGraph,
    current_node: NodeName,
    branching: Branching,
    #[reflect(ignore)]
    guard_systems: HashMap<NodeName, GuardSystem>,
    /// When dialog is finished, run these commands.
    #[reflect(ignore)]
    when_finished: Vec<CmdFn>,
}

/// Node name uniquely identifies a node across all dialogs.
/// This is achieved by having namespaces (represent files) and node names or
/// auto-generated node names.
#[derive(Debug, Reflect, Clone, Hash, PartialEq, Eq, Default)]
pub enum NodeName {
    /// This node has been explicitly named in the dialog file.
    /// If a node is explicitly named, other nodes can refer to it.
    /// It also enabled persistent state after the dialog is over.
    Explicit(Namespace, String),
    /// If a node does not have an explicit name in the dialog file, we
    /// generate one for it.
    /// Since a dialog file is just a vector of node definitions, we use the
    /// index of the node in the vector as the auto-generated name.
    Auto(Namespace, usize),
    /// There's always exactly one root node in each dialog file.
    /// However, by merging multiple dialog files, we can have multiple root
    /// nodes in a graph.
    NamespaceRoot(Namespace),
    /// The root of the _whole dialog_.
    /// This is not the same as the root of a dialog file.
    /// There's always exactly one root node in the whole dialog.
    /// When we _emerge_, we go to this root.
    /// The root node points to one or more namespace roots.
    #[default]
    Root,
    /// Special node that marks the end of the dialog.
    /// When we reach this node, the dialog is despawned.
    EndDialog,
}

/// The dialog asset that can be started.
/// Since dialogs can be stateful, state is lazy loaded.
/// The state is managed by systems called guards.
#[derive(Debug, Reflect, Default)]
pub struct DialogGraph {
    root: NodeName,
    nodes: HashMap<NodeName, Node>,
}

#[derive(Debug, Reflect)]
struct Node {
    name: NodeName,
    who: Character,
    kind: NodeKind,
    next: Vec<NodeName>,
}

#[derive(Debug, Reflect)]
enum NodeKind {
    Guard {
        /// Guard states are persisted across dialog sessions if
        /// - the node has a [`NodeName::Explicit`]
        ///
        /// Otherwise the state is discarded after the dialog is over.
        kind: GuardKind,
        #[allow(dead_code)]
        #[reflect(ignore)]
        params: HashMap<String, toml::Value>,
    },
    Vocative {
        /// The dialog line to print.
        /// TODO: https://github.com/porkbrain/dont-count-the-sheep/issues/95
        line: String,
    },
    /// A node that does nothing.
    Blank,
}

#[derive(Reflect, Debug, Default)]
enum Branching {
    #[default]
    None,
    Single(NodeName),
    Choice(Vec<BranchStatus>),
}

#[derive(Reflect, Debug)]
enum BranchStatus {
    /// Guards can be async.
    /// They will eventually transition this status into another variant.
    Pending,
    OfferAsChoice(String),
    /// This branch is exhausted, presumably some guard decided to stop it.
    Stop,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AdvanceOutcome {
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

#[derive(Debug)]
struct BranchPending;

impl Dialog {
    /// Schedule a command to run when the dialog is finished.
    /// As many commands as you want can be scheduled.
    pub fn on_finished(mut self, fun: CmdFn) -> Self {
        self.when_finished.push(fun);
        self
    }

    /// If there are no choices to be made by the player, this method returns
    /// [`None`]`. If there are choices, but not yet ready, this method
    /// returns [`Some(Err(Pending))`].
    fn get_choices(
        &self,
    ) -> Option<Result<Vec<(&NodeName, &str)>, BranchPending>> {
        if let Branching::Choice(branches) = &self.branching {
            let node = self.current_node_info();
            let branches = branches
                .iter()
                .enumerate()
                .filter_map(|(branch_index, status)| match status {
                    BranchStatus::OfferAsChoice(text) => {
                        let node_name = &node.next[branch_index];

                        Some(Ok((node_name, text.as_str())))
                    }
                    BranchStatus::Stop => None,
                    BranchStatus::Pending => Some(Err(BranchPending)),
                })
                .collect::<Result<Vec<_>, _>>();

            if let Ok(branches) = branches {
                if branches.len() > 1 {
                    Some(Ok(branches))
                } else {
                    // zero or one branch is not a choice
                    None
                }
            } else {
                Some(Err(BranchPending))
            }
        } else {
            None
        }
    }

    /// This method should be called by FE repeatedly until a node changes or
    /// all choice branches are evaluated.
    fn advance(
        &mut self,
        cmd: &mut Commands,
        store: &GlobalStore,
    ) -> AdvanceOutcome {
        if NodeName::EndDialog == self.current_node {
            self.despawn(cmd);
            return AdvanceOutcome::ScheduledDespawn;
        }

        match &self.graph.nodes.get(&self.current_node).unwrap().kind {
            NodeKind::Blank | NodeKind::Vocative { .. } => {
                self.transition_or_offer_player_choice_if_all_ready(cmd, store)
            }
            NodeKind::Guard { kind, .. } => {
                let node_name = self.current_node.clone();
                if let Some(guard_system) = self.guard_systems.get(&node_name) {
                    cmd.run_system_with_input(
                        guard_system.entity,
                        GuardCmd::TryTransition(node_name),
                    );
                } else {
                    trace!("Registering guard {kind:?} for node {node_name:?}");
                    cmd.add(kind.register_system_cmd(node_name.clone()));
                    cmd.add(GuardCmd::TryTransition(node_name));
                }

                AdvanceOutcome::WaitUntilNextTick
            }
        }
    }

    fn current_node_info(&self) -> &Node {
        self.graph.nodes.get(&self.current_node).unwrap()
    }

    fn transition_to(
        &mut self,
        cmd: &mut Commands,
        store: &GlobalStore,
        node_name: NodeName,
    ) {
        if let Some((namespace, node_name)) =
            node_name.as_namespace_and_node_name_str()
        {
            store.insert_dialog((namespace, node_name));
        }

        self.current_node = node_name.clone();
        self.branching =
            Branching::new(cmd, &node_name, &self.graph, &self.guard_systems)
    }

    fn spawn(self, cmd: &mut Commands) {
        cmd.insert_resource(self);
    }

    fn despawn(&mut self, cmd: &mut Commands) {
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
        store: &GlobalStore,
    ) -> AdvanceOutcome {
        match &self.branching {
            Branching::None => {
                self.transition_to(
                    cmd,
                    store,
                    if self.current_node == NodeName::Root {
                        error!("NextNodes::None in the root");
                        NodeName::EndDialog
                    } else {
                        warn!("NextNodes::None, emerging");
                        NodeName::Root
                    },
                );
                AdvanceOutcome::Transition
            }
            Branching::Single(next_node) => {
                self.transition_to(cmd, store, next_node.clone());
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
                        self.transition_to(cmd, store, first_choice_node_name);
                        AdvanceOutcome::Transition
                    } else {
                        AdvanceOutcome::AwaitingPlayerChoice
                    }
                } else {
                    warn!("NextNodes::Choice stopped all branches, emerging");
                    self.transition_to(cmd, store, NodeName::Root);
                    AdvanceOutcome::Transition
                }
            }
        }
    }
}

impl Branching {
    fn new(
        cmd: &mut Commands,
        from: &NodeName,
        graph: &DialogGraph,
        guard_systems: &HashMap<NodeName, GuardSystem>,
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
        let next_nodes = &graph.nodes.get(&NodeName::Root).unwrap().next;
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
        guard_systems: &HashMap<NodeName, GuardSystem>,
        branch_index: usize,
        next_node_name: &NodeName,
    ) -> Self {
        let next_node = &graph.nodes.get(next_node_name).unwrap();
        assert_eq!(
            Character::Winnie,
            next_node.who,
            "Only Winnie can branch ({:?})",
            next_node
        );

        match &next_node.kind {
            NodeKind::Blank => Self::Stop,
            NodeKind::Vocative { line } => {
                //  TODO: https://github.com/porkbrain/dont-count-the-sheep/issues/95
                Self::OfferAsChoice(line.clone())
            }
            NodeKind::Guard { kind, .. } => {
                if let Some(guard_system) = guard_systems.get(next_node_name) {
                    cmd.run_system_with_input(
                        guard_system.entity,
                        GuardCmd::PlayerChoice {
                            node_name: next_node_name.clone(),
                            next_branch_index: branch_index,
                        },
                    )
                } else {
                    trace!(
                        "Registering guard system {kind:?} for {next_node_name:?}"
                    );
                    cmd.add(kind.register_system_cmd(next_node_name.clone()));
                    cmd.add(GuardCmd::PlayerChoice {
                        node_name: next_node_name.clone(),
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
        node_name: &NodeName,
    ) -> Self {
        let next_node = &graph.nodes.get(node_name).unwrap();
        assert_eq!(
            Character::Winnie,
            next_node.who,
            "Only Winnie can branch ({:?})",
            next_node
        );

        match &next_node.kind {
            NodeKind::Blank => Self::Stop,
            NodeKind::Vocative { line } => {
                //  TODO: https://github.com/porkbrain/dont-count-the-sheep/issues/95
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
    ///
    /// # Panics
    /// The dialog panics if it's not a root graph.
    /// Run [`DialogGraph::into_root_graph`].
    #[must_use]
    pub fn into_dialog_resource(self, cmd: &mut CommandQueue) -> Dialog {
        assert!(!self.is_subgraph());
        let branching = Branching::init(cmd, &self);
        Dialog {
            current_node: NodeName::Root,
            graph: self,
            guard_systems: default(),
            branching,
            when_finished: default(),
        }
    }

    /// A subgraph is not ready to be spawned as a dialog.
    /// Either call [] or attach it to an existing root graph with []
    pub fn is_subgraph(&self) -> bool {
        self.root != NodeName::Root
    }

    /// What node names are present in the graph.
    pub fn node_names(&self) -> impl Iterator<Item = &NodeName> {
        self.nodes.keys()
    }

    /// Convert a subgraph into a root graph by inserting the root node and
    /// pointing the root to this subgraph's root.
    ///
    /// Optionally, provide a line that's shown when the dialog starts.
    /// Otherwise the root node will be blank.
    /// That might look awkward if there are two or more subgraphs attached to
    /// the root.
    #[must_use]
    pub fn into_root_graph(mut self, root_line: Option<String>) -> Self {
        assert!(self.is_subgraph());
        let namespace_root = self.root;
        let who = self.nodes.get(&namespace_root).unwrap().who;

        self.nodes.insert(
            NodeName::Root,
            Node {
                name: NodeName::Root,
                who,
                kind: if let Some(line) = root_line {
                    NodeKind::Vocative { line }
                } else {
                    NodeKind::Blank
                },
                next: vec![namespace_root],
            },
        );

        self.root = NodeName::Root;
        self
    }

    /// Attach a subgraph to a node in the graph.
    /// The subgraph will be added to the next nodes of the `to` arg node.
    pub fn attach(&mut self, other: Self, to: NodeName) {
        assert!(other.is_subgraph());
        assert!(!self.nodes.contains_key(&other.root));
        assert!(self.nodes.contains_key(&to));

        let other_root = other.root;
        self.nodes.get_mut(&to).unwrap().next.push(other_root);
        self.nodes.extend(other.nodes);
    }
}

impl NodeName {
    const NAMESPACE_ROOT: &'static str = "_root";

    /// Get the namespace and node name.
    /// Only works for [`NodeName::Explicit`] and [`NodeName::NamespaceRoot`].
    pub fn as_namespace_and_node_name_str(&self) -> Option<(Namespace, &str)> {
        match self {
            Self::Explicit(namespace, node_name) => {
                Some((*namespace, node_name))
            }
            Self::NamespaceRoot(namespace) => {
                Some((*namespace, Self::NAMESPACE_ROOT))
            }
            _ => None,
        }
    }

    fn from_namespace_and_node_name_str(
        namespace: Namespace,
        node_name: String,
    ) -> Self {
        match node_name.as_str() {
            "_end_dialog" => Self::EndDialog,
            "_emerge" => Self::Root,
            Self::NAMESPACE_ROOT => Self::NamespaceRoot(namespace),
            _ => Self::Explicit(namespace, node_name),
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
            graph: default(),
            guard_systems: default(),
            current_node: NodeName::Root,
            branching: default(),
            when_finished: default(),
        }
    }
}
