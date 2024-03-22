mod exhaustive_alternatives;
mod reach_last_alternative;

use bevy::{
    ecs::{
        system::{Command, Commands, In, Local, Res, ResMut, SystemId},
        world::World,
    },
    log::error,
    reflect::Reflect,
};
use common_store::{DialogStore, GlobalStore};

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
    /// This command says: in the [`Dialog::next_nodes`] array, at the
    /// specified index, give us string that we should show to the player
    /// as a choice.
    /// It's possible that the guard will decide to stop the current branch
    /// with [`NextNode::Stop`].
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
pub(crate) enum GuardKind {
    ExhaustiveAlternatives,
    ReachLastAlternative,
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
