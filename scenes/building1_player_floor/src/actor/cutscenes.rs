use bevy_grid_squared::{GridDirection, Square};
use common_loading_screen::LoadingScreenSettings;
use common_store::{DialogStore, GlobalStore};
use common_story::dialog;
use common_visuals::EASE_IN_OUT;
use main_game_lib::{
    cutscene::{self, CutsceneStep, IntoCutscene},
    state::GlobalGameStateTransition as Ggst,
};
use top_down::layout::LAYOUT;

use crate::prelude::*;

/// When near the elevator and presses the interaction button, start this
/// cutscene.
pub(super) struct EnterTheElevator {
    pub(super) player: Entity,
    pub(super) elevator: Entity,
    pub(super) camera: Entity,
    /// Get position of entity with [`Point`] component and [`Name`] that
    /// matches "InElevator"
    pub(super) point_in_elevator: Vec2,
}

impl IntoCutscene for EnterTheElevator {
    fn has_letterboxing() -> bool {
        true
    }

    fn sequence(self) -> Vec<CutsceneStep> {
        use CutsceneStep::*;
        let Self {
            player,
            // has atlas animation component, see layout where it's spawned
            elevator,
            camera,
            point_in_elevator,
        } = self;

        let in_elevator = LAYOUT.world_pos_to_square(point_in_elevator);

        vec![
            TakeAwayPlayerControl(player),
            SetActorFacingDirection(player, GridDirection::Top),
            // move camera to the player
            BeginMovingEntity {
                who: camera,
                to: cutscene::Destination::Entity(player),
                over: from_millis(1000),
                animation_curve: Some(EASE_IN_OUT.clone()),
            },
            // open the elevator door
            InsertAtlasAnimationTimerTo {
                entity: elevator,
                duration: from_millis(150),
                mode: TimerMode::Repeating,
            },
            Sleep(from_millis(1250)),
            // jump into the elevator
            BeginSimpleWalkTo {
                with: player,
                // hop up
                square: in_elevator + Square::new(0, 1),
                // and down
                planned: Some((in_elevator, GridDirection::Bottom)),
                step_time: None,
            },
            WaitUntilActorAtRest(player),
            Sleep(from_millis(300)),
            // ask player where to go
            BeginPortraitDialog(
                dialog::TypedNamespace::EnterTheApartmentElevator.into(),
            ),
            WaitForPortraitDialogToEnd,
            Sleep(from_millis(300)),
            IfTrueThisElseThat(
                did_choose_to_leave,
                // then transition to downtown
                Box::new(vec![ChangeGlobalState {
                    to: GlobalGameState::ApartmentQuitting,
                    with: Ggst::ApartmentToDowntown,
                    loading_screen: Some(LoadingScreenSettings {
                        atlas: Some(
                            common_loading_screen::LoadingScreenAtlas::random(),
                        ),
                        stare_at_loading_screen_for_at_least: Some(
                            from_millis(2000),
                        ),
                        ..default()
                    }),
                    // this is already done in this scene's smooth exit sys
                    change_loading_screen_state_to_start: false,
                }]),
                // else step out of the elevator
                Box::new(vec![
                    BeginSimpleWalkTo {
                        with: player,
                        square: in_elevator - Square::new(0, 2),
                        step_time: Some(STEP_TIME_ON_EXIT_ELEVATOR),
                        planned: None,
                    },
                    Sleep(from_millis(250)),
                    // closes the elevator door
                    ReverseAtlasAnimation(elevator),
                    InsertAtlasAnimationTimerTo {
                        entity: elevator,
                        duration: from_millis(150),
                        mode: TimerMode::Repeating,
                    },
                    WaitUntilAtlasAnimationEnds(elevator),
                    // get it ready for the next time this scene runs
                    ReverseAtlasAnimation(elevator),
                    WaitUntilActorAtRest(player),
                    // reset the camera
                    BeginMovingEntity {
                        who: camera,
                        to: cutscene::Destination::Position(default()),
                        over: from_millis(1000),
                        animation_curve: Some(EASE_IN_OUT.clone()),
                    },
                    ReturnPlayerControl(player),
                ]),
            ),
        ]
    }
}

const GROUND_FLOOR_NODE_NAME: &str = "ground_floor";

fn did_choose_to_leave(store: &GlobalStore) -> bool {
    store.was_this_the_last_dialog((
        dialog::TypedNamespace::EnterTheApartmentElevator,
        GROUND_FLOOR_NODE_NAME,
    ))
}

#[cfg(test)]
mod tests {
    use common_story::dialog::DialogGraph;

    use super::*;

    #[test]
    fn it_has_ground_floor_node() {
        let namespace = dialog::TypedNamespace::EnterTheApartmentElevator;

        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = format!(
            "{manifest}/../../main_game/assets/dialogs/{namespace}.toml"
        );
        let toml = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{path}: {e}"));

        let dialog = DialogGraph::subgraph_from_raw(namespace.into(), &toml);

        dialog
            .node_names()
            .filter_map(|node| {
                let (_, name) = node.as_namespace_and_node_name_str()?;
                Some(name.to_owned())
            })
            .find(|name| name.as_str() == GROUND_FLOOR_NODE_NAME)
            .expect("Ground floor not found");
    }
}
