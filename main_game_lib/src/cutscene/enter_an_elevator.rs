//! Cutscene useful for elevators in the game.
//! The player jumps into it and gets asked where to go with provided dialog.

use bevy_grid_squared::{GridDirection, Square};
use common_store::GlobalStore;
use common_story::dialog;
use common_visuals::EASE_IN_OUT;
use top_down::layout::LAYOUT;

use crate::{
    cutscene::{self, CutsceneStep, IntoCutscene},
    prelude::*,
};

/// For the animation of stepping out of the elevator.
pub const STEP_TIME_ON_EXIT_ELEVATOR: Duration = from_millis(750);

/// Typically started when player near the elevator and presses the interaction
/// button.
pub struct EnterAnElevator {
    /// The player entity.
    /// Is actor.
    pub player: Entity,
    /// The elevator entity.
    /// Is sprite atlas.
    pub elevator: Entity,
    /// The camera entity.
    /// Will be moved to center on the player.
    pub camera: Entity,
    /// Get position player should jump onto.
    pub point_in_elevator: Vec2,
    /// What dialog to show the player.
    pub dialog: dialog::TypedNamespace,
    /// Return true if the player should exit the elevator.
    pub is_cancelled: fn(&GlobalStore) -> bool,
    /// If the player decided not to exit the elevator, what to do?
    /// Presumably transition to another scene.
    pub on_took_the_elevator: fn(&mut Commands, &GlobalStore),
}

impl IntoCutscene for EnterAnElevator {
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
            is_cancelled: cancel_and_exit_elevator,
            on_took_the_elevator: change_global_state,
            dialog,
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
            BeginPortraitDialog(dialog.into()),
            WaitForPortraitDialogToEnd,
            Sleep(from_millis(300)),
            IfTrueThisElseThat(
                cancel_and_exit_elevator,
                // then step out of the elevator
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
                // else transition to downtown
                Box::new(vec![ScheduleCommands(change_global_state)]),
            ),
        ]
    }
}
