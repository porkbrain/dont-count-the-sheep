use bevy::ecs::{
    schedule::NextState,
    system::{Res, ResMut},
};
use common_store::{DialogStore, GlobalStore};
use common_story::{dialog::DialogRoot, Character};
use main_game_lib::state::GlobalGameState;

pub(crate) fn on_enter(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    store: Res<GlobalStore>,
) {
    store.add_dialog_to_npc(Character::Marie, DialogRoot::MarieBlabbering);
    store.add_dialog_to_npc(Character::Bolt, DialogRoot::BoltIsMean);

    next_state.set(GlobalGameState::ApartmentLoading);
}
