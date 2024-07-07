use bevy::{
    ecs::system::{Res, ResMut},
    state::state::NextState,
};
use common_store::{DialogStore, GlobalStore};
use common_story::Character;
use main_game_lib::{dialog, state::GlobalGameState};

pub(crate) fn on_enter(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    store: Res<GlobalStore>,
) {
    store
        .add_dialog_to_npc(
            Character::Marie,
            dialog::TypedNamespace::MarieBlabbering,
        )
        .add_dialog_to_npc(Character::Bolt, dialog::TypedNamespace::BoltIsMean)
        .add_dialog_to_npc(
            Character::GingerCat,
            dialog::TypedNamespace::MrGoodWater,
        )
        .add_dialog_to_npc(
            Character::Cooper,
            dialog::TypedNamespace::InitialCooper,
        )
        .add_dialog_to_npc(
            Character::Samizdat,
            dialog::TypedNamespace::InitialSamizdat,
        )
        .add_dialog_to_npc(
            Character::Otter,
            dialog::TypedNamespace::InitialOtter,
        )
        .add_dialog_to_npc(
            Character::Phoebe,
            dialog::TypedNamespace::InitialPhoebe,
        );

    next_state.set(GlobalGameState::LoadingBuilding1PlayerFloor);
}
