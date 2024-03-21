use bevy_grid_squared::{GridDirection, Square};
use common_loading_screen::LoadingScreenSettings;
use common_store::{DialogStore, GlobalStore};
use common_story::dialog::DialogRoot;
use common_visuals::EASE_IN_OUT;
use main_game_lib::{
    cutscene::{self, CutsceneStep, IntoCutscene},
    state::GlobalGameStateTransition as Ggst,
};

use crate::{consts::*, prelude::*};

/// When near the elevator and presses the interaction button, start this
/// cutscene.
pub(super) struct EnterTheElevator {
    pub(super) player: Entity,
    pub(super) elevator: Entity,
    pub(super) camera: Entity,
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
        } = self;

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
                square: Square::new(-57, -19),
                planned: Some((Square::new(-57, -20), GridDirection::Bottom)),
                step_time: None,
            },
            WaitUntilActorAtRest(player),
            Sleep(from_millis(300)),
            // ask player where to go
            BeginPortraitDialog(DialogRoot::EnterTheApartmentElevator),
            WaitForPortraitDialogToEnd,
            Sleep(from_millis(300)),
            IfTrueThisElseThat(
                did_choose_to_leave,
                // then transition to downtown
                Box::new(vec![ChangeGlobalState {
                    to: GlobalGameState::ApartmentQuitting,
                    with: Ggst::ApartmentQuittingToDowntownLoading,
                    loading_screen: Some(LoadingScreenSettings { ..default() }),
                    // this is already done in this scene's smooth exit sys
                    change_loading_screen_state_to_start: false,
                }]),
                // else step out of the elevator
                Box::new(vec![
                    BeginSimpleWalkTo {
                        with: player,
                        square: Square::new(-57, -22),
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

fn did_choose_to_leave(store: &GlobalStore) -> bool {
    todo!()
    // store.was_this_the_last_dialog(TakeTheElevatorToGroundFloor)
}
