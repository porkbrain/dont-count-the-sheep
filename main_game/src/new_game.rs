use bevy::ecs::{
    schedule::NextState,
    system::{Res, ResMut},
};
use common_store::{DialogStore, GlobalStore};
use common_story::{dialog, Character};
use main_game_lib::state::GlobalGameState;

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
        );

    next_state.set(GlobalGameState::LoadingBuilding1PlayerFloor);
}
