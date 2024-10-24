mod dialog_for_npc;
mod exhaustive_alternatives;
mod notify;
mod reach_last_alternative;
mod visit_once;

use bevy::{
    ecs::{
        system::{Commands, In, Local, Res, ResMut, SystemId},
        world::{Command, World},
    },
    log::{error, warn},
    reflect::Reflect,
    utils::default,
};
use common_store::{DialogStore, GlobalStore};
use serde::de::DeserializeOwned;

use super::{BranchStatus, Branching, Dialog, NodeKind, NodeName};

#[derive(Reflect, Debug, Clone, Copy)]
pub(crate) struct GuardSystem {
    pub(crate) kind: GuardKind,
    pub(crate) entity: SystemId<GuardCmd>,
}

#[derive(Debug, Clone)]
pub(crate) enum GuardCmd {
    /// Will try change the current node of the dialog is ready.
    ///
    /// For guard with async ops, such as displaying UI with animations,
    /// this command might not result in transition.
    TryTransition(NodeName),
    /// We want to show player choices in dialog.
    /// This command says: in the [`Dialog::branching`]'s [`Branching::Choice`]
    /// vector, at the specified index, give us string that we should show
    /// to the player as a choice.
    /// It's possible that the guard will decide to stop the current branch
    /// with [`BranchStatus::Stop`].
    PlayerChoice {
        node_name: NodeName,
        next_branch_index: usize,
    },
    /// The dialog is being despawned, save the state if necessary.
    ///
    /// # Important
    /// The command for despawning the system comes immediately after this
    /// guard cmd.
    Despawn(NodeName),
}

#[derive(Debug, Reflect, strum::EnumString, strum::Display, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
#[allow(missing_docs)]
pub enum GuardKind {
    ExhaustiveAlternatives,
    ReachLastAlternative,
    VisitOnce,

    AddDialogToNpc,
    RemoveDialogFromNpc,

    /// Pushes a notification to the player.
    Notify,
}

impl GuardKind {
    pub(crate) fn register_system_cmd(
        self,
        node_name: NodeName,
    ) -> impl Command {
        move |w: &mut World| {
            let entity = match self {
                Self::ExhaustiveAlternatives => {
                    w.register_system(exhaustive_alternatives::system)
                }
                Self::ReachLastAlternative => {
                    w.register_system(reach_last_alternative::system)
                }
                Self::VisitOnce => w.register_system(visit_once::system),
                Self::AddDialogToNpc => w.register_system(dialog_for_npc::add),
                Self::RemoveDialogFromNpc => {
                    w.register_system(dialog_for_npc::remove)
                }
                Self::Notify => w.register_system(notify::system),
            };
            if let Some(mut dialog) = w.get_resource_mut::<Dialog>() {
                dialog
                    .guard_systems
                    .insert(node_name, GuardSystem { entity, kind: self });
            } else {
                error!(
                    "Trying to add a guard {self} to a \
                    world without Dialog resource"
                );
            }
        }
    }
}

impl GuardKind {
    fn load_state<T>(self, store: &GlobalStore, guard_cmd: &GuardCmd) -> T
    where
        T: Default + DeserializeOwned,
    {
        match guard_cmd {
            GuardCmd::TryTransition(NodeName::Explicit(
                namespace,
                node_name,
            ))
            | GuardCmd::PlayerChoice {
                node_name: NodeName::Explicit(namespace, node_name),
                ..
            } => {
                let from_store = store
                    .guard_state(self, (namespace, node_name))
                    .get()
                    .and_then(|v| serde_json::from_value(v).ok());
                from_store.unwrap_or(default())
            }
            _ => default(),
        }
    }
}

impl Command for GuardCmd {
    fn apply(self, w: &mut World) {
        if let Some(dialog) = w.get_resource::<Dialog>() {
            if let Some(GuardSystem { entity, kind }) =
                dialog.guard_systems.get(self.node_name()).cloned()
            {
                if let Err(e) = w.run_system_with_input(entity, self.clone()) {
                    error!("Error running {self:?} for {kind}: {e:?}",);
                }
            }
        } else {
            error!("Dialog not found when trying to run {self:?}");
        }
    }
}

impl GuardCmd {
    fn node_name(&self) -> &NodeName {
        match self {
            Self::TryTransition(node_name)
            | Self::PlayerChoice { node_name, .. }
            | Self::Despawn(node_name) => node_name,
        }
    }
}

impl Dialog {
    fn pass_guard_player_choice(
        &mut self,
        cmd: &mut Commands,
        node_name: NodeName,
        next_branch_index: usize,
    ) {
        let next_nodes = &self.graph.nodes.get(&node_name).unwrap().next;
        assert_eq!(1, next_nodes.len());

        let next_node_name = &next_nodes[0];
        let next_node_kind =
            &self.graph.nodes.get(next_node_name).unwrap().kind;

        let next_node_choice = match next_node_kind {
            NodeKind::Blank => {
                warn!("{node_name:?}: Next node {next_node_name:?} is blank");
                BranchStatus::Stop
            }
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
        };

        if let Branching::Choice(branches) = &mut self.branching {
            branches[next_branch_index] = next_node_choice;
        };
    }
}
