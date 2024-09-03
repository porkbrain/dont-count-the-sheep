//! Cutscene useful for elevators in the game.
//! The player jumps into it and gets asked where to go with provided dialog.

use std::{iter, str::FromStr};

use bevy_grid_squared::{GridDirection, Square};
use common_assets::audio::CALL_ELEVATOR;
use common_loading_screen::{LoadingScreenAtlas, LoadingScreenSettings};
use common_store::{DialogStore, GlobalStore};
use common_story::Character;
use common_visuals::{
    AtlasAnimation, AtlasAnimationEnd, AtlasAnimationStep, AtlasAnimationTimer,
    BeginAtlasAnimation, EASE_IN_OUT,
};
use rscn::EntityDescription;
use top_down::layout::LAYOUT;

use crate::{
    cutscene::{self, CutsceneStep, IntoCutscene},
    dialog::{self, DialogGraph},
    prelude::*,
    top_down::actor::player::TakeAwayPlayerControl,
};

/// For the animation of stepping out of the elevator.
pub const STEP_TIME_ON_EXIT_ELEVATOR: Duration = from_millis(750);

/// Typically started when player near the elevator and presses the interaction
/// button.
struct EnterAnElevator {
    /// The player entity.
    /// Is actor.
    player: Entity,
    /// The elevator entity.
    /// Is sprite atlas.
    elevator: Entity,
    /// The camera entity.
    /// Will be moved to center on the player.
    camera: Entity,
    /// Get position player should jump onto.
    point_in_elevator: Vec2,
    /// What dialog to show the player.
    dialog: dialog::DialogRef,
    /// Return true if the player should exit the elevator.
    is_cancelled: fn(&GlobalStore) -> bool,
    /// If the player decided not to exit the elevator, what to do?
    /// Presumably transition to another scene.
    on_took_the_elevator: fn(&mut Commands, &GlobalStore),
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
            // take over camera manual control
            ClaimManualMainCameraControl,
            // move camera to the player
            BeginMovingEntity {
                who: camera,
                to: cutscene::Destination::Entity(player),
                over: from_millis(1000),
                animation_curve: Some(*EASE_IN_OUT),
            },
            // open the elevator door
            InsertAtlasAnimationTimerTo {
                entity: elevator,
                duration: from_millis(150),
                mode: TimerMode::Repeating,
            },
            Sleep(from_millis(1250)),
            StartPlayingAudio {
                asset_path: CALL_ELEVATOR.into(),
            },
            // jump into the elevator
            BeginSimpleWalkTo {
                with: player,
                // hop up
                square: in_elevator + Square::new(0, 1),
                planned: None,
                step_time: None,
            },
            WaitUntilActorAtRest(player),
            SetActorFacingDirection(player, GridDirection::Bottom),
            BeginSimpleWalkTo {
                with: player,
                // and down
                square: in_elevator,
                planned: None,
                step_time: None,
            },
            WaitUntilActorAtRest(player),
            Sleep(from_millis(300)),
            // ask player where to go
            BeginPortraitDialog(dialog),
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
                    ReleaseManualMainCameraControl,
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
                    ReturnPlayerControl(player),
                ]),
                // else transition
                Box::new(vec![
                    ScheduleCommands(change_global_state),
                    StartLoadingScreen {
                        settings: Some(LoadingScreenSettings {
                            atlas: Some(LoadingScreenAtlas::random()),
                            stare_at_loading_screen_for_at_least: Some(
                                from_millis(1500),
                            ),
                            ..default()
                        }),
                    },
                ]),
            ),
        ]
    }
}

/// Spawns the cutscene for entering an elevator.
pub fn spawn(
    cmd: &mut Commands,
    dialogs: &mut Assets<DialogGraph>,

    player: Entity,
    elevator: Entity,
    camera: Entity,
    point_in_elevator: Vec2,
    // LOCALIZATION
    choices: &[(GlobalGameStateTransition, &str)],
) {
    assert!(!choices.is_empty());

    const NAMESPACE: dialog::TypedNamespace =
        dialog::TypedNamespace::InElevator;
    const EXIT_ELEVATOR_NODE_NAME: &str = "exit_elevator";

    trace!("Enter an elevator with choices: {choices:?}");

    // Runtime dialog, we can be certain there is not going to be a memory leak
    // because new strong handle is allocated when inserting the dialog.
    // And this handle is dropped when cutscene finishes.
    let dialog_graph = {
        let mut g = DialogGraph::new_subgraph(dialog::Node {
            name: dialog::NodeName::NamespaceRoot(NAMESPACE.into()),
            who: Character::Winnie,
            // LOCALIZATION
            kind: dialog::NodeKind::Vocative {
                line: "Let's see, I'm going to ...".to_owned(),
            },
            next: choices
                .iter()
                .map(|(transition, _)| {
                    dialog::NodeName::Explicit(
                        NAMESPACE.into(),
                        transition.to_string(),
                    )
                })
                .chain(iter::once(dialog::NodeName::Explicit(
                    NAMESPACE.into(),
                    EXIT_ELEVATOR_NODE_NAME.to_owned(),
                )))
                .collect(),
        });

        for (transition, line) in choices {
            g.insert_node(dialog::Node {
                name: dialog::NodeName::Explicit(
                    NAMESPACE.into(),
                    transition.to_string(),
                ),
                who: Character::Winnie,
                kind: dialog::NodeKind::Vocative {
                    line: line.to_string(),
                },
                next: vec![dialog::NodeName::EndDialog],
            });
        }

        g.insert_node(dialog::Node {
            name: dialog::NodeName::Explicit(
                NAMESPACE.into(),
                EXIT_ELEVATOR_NODE_NAME.to_owned(),
            ),
            who: Character::Winnie,
            // LOCALIZATION
            kind: dialog::NodeKind::Vocative {
                line: "exit the elevator".to_owned(),
            },
            next: vec![dialog::NodeName::EndDialog],
        });

        g
    };
    let dialog_strong_handle = dialogs.add(dialog_graph);

    fn did_choose_to_cancel_and_exit_the_elevator(store: &GlobalStore) -> bool {
        match store.get_last_dialog::<dialog::Namespace>() {
            None => true,
            Some((_, node)) => {
                GlobalGameStateTransition::from_str(&node).is_err()
            }
        }
    }

    fn on_took_the_elevator(cmd: &mut Commands, store: &GlobalStore) {
        // get current state, from that infer quitting state
        cmd.add(|w: &mut World| {
            let current_top_down_scene = **w
                .get_resource::<State<WhichTopDownScene>>()
                .expect("Elevator can only be used in top down scenes");

            // SAFETY: always present
            let mut next_state =
                w.get_resource_mut::<NextState<GlobalGameState>>().unwrap();
            next_state.set(GlobalGameState::LeavingTopDownScene(
                current_top_down_scene,
            ));
        });

        match store
            .get_last_dialog::<dialog::Namespace>()
            .map(|(_, node)| GlobalGameStateTransition::from_str(&node))
        {
            Some(Ok(transition)) => {
                trace!("Enter an elevator transition: {transition}");
                cmd.insert_resource(transition);
            }
            Some(Err(e)) => {
                panic!("Expected last dialog to be a transition, but got: {e}");
            }
            None => {
                panic!("Expected some last dialog when taking the elevator")
            }
        }
    }

    cutscene::enter_an_elevator::EnterAnElevator {
        player,
        elevator,
        camera,
        point_in_elevator,
        dialog: dialog_strong_handle.into(),
        is_cancelled: did_choose_to_cancel_and_exit_the_elevator,
        on_took_the_elevator,
    }
    .spawn(cmd);
}

/// A helper that can be ran when the elevator is spawned.
///
/// It sets the current frame of the elevator to the open state (last frame)
/// and scheduled a close animation.
///
/// Then, when finished, it sets the elevator back to normal.
/// Also, removes player's [`TakeAwayPlayerControl`] component.
/// Make sure to insert it, otherwise the player could interact with the
/// elevator and it'd be a mess.
pub fn start_with_open_elevator_and_close_it(
    cmd: &mut Commands,
    player: Entity,
    elevator_entity: Entity,
    elevator: &mut EntityDescription,
) {
    trace!("Setting elevator to last frame");
    let last_frame = elevator
        .atlas_animation
        .as_ref()
        .expect("Elevator must have AtlasAnimation")
        .last;
    // set the last frame as the current index
    elevator
        .texture_atlas
        .as_mut()
        .expect("Elevator must have TextureAtlas")
        .index = last_frame;
    // start the animation as soon as we are in running state
    fn is_in_running_global_state(w: &World, _: Entity) -> bool {
        w.get_resource::<State<InTopDownScene>>()
            .is_some_and(|scene| scene.is_running())
    }
    cmd.entity(elevator_entity).insert(
        common_visuals::BeginAtlasAnimation::run(
            is_in_running_global_state,
            from_millis(150),
            Some(from_millis(1500)),
        ),
    );

    let a = elevator
        .atlas_animation
        .as_mut()
        .expect("Elevator must have AtlasAnimation");
    // animation runs in reverse
    a.play = AtlasAnimationStep::Backward;
    // on last frame, put everything back to normal
    a.on_last_frame =
        AtlasAnimationEnd::run(Box::new(move |cmd, who, atlas, _| {
            // animation finished in reverse
            debug_assert_eq!(0, atlas.index);

            // allow the player to move again
            cmd.entity(player).remove::<TakeAwayPlayerControl>();

            cmd.entity(who).add(|mut e: EntityWorldMut| {
                trace!("Setting elevator animation back to normal");
                // no anim running
                e.remove::<AtlasAnimationTimer>();
                e.remove::<BeginAtlasAnimation>();

                let mut a = e.get_mut::<AtlasAnimation>().unwrap();
                // back to normal
                a.on_last_frame = AtlasAnimationEnd::RemoveTimer;
                a.play = AtlasAnimationStep::Forward;
            });
        }));
}
