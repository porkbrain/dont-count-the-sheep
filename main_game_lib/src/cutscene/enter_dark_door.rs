//! Cut scene in which the player goes into door and slowly fades out the
//! sprite so it's as if they are entering a dark room.

use bevy_grid_squared::{sq, GridDirection};
use common_loading_screen::LoadingScreenSettings;
use common_story::Character;

use crate::{
    cutscene::{CutsceneStep, IntoCutscene},
    prelude::*,
    top_down::layout::LAYOUT,
};

/// Cut scene in which the player goes into door and slowly fades out the
/// sprite so it's as if they are entering a dark room.
pub struct EnterDarkDoor {
    /// The player entity.
    /// Is actor.
    pub player: Entity,
    /// We animate it to second frame to simulate door opening.
    pub door: Entity,
    /// Position at the tip of the door.
    /// If the player is here, they can walk up to the door.
    pub door_entrance: Vec2,
    /// The global game state to change to.
    pub change_global_state_to: GlobalGameState,
    /// Using this transition.
    pub transition: GlobalGameStateTransition,
    /// What loading screen to start.
    pub loading_screen: LoadingScreenSettings,
}

impl IntoCutscene for EnterDarkDoor {
    fn sequence(self) -> Vec<CutsceneStep> {
        use CutsceneStep::*;
        let Self {
            player,
            // has atlas animation component, see layout where it's spawned
            door,
            door_entrance,
            change_global_state_to,
            transition,
            loading_screen,
        } = self;

        let walk_towards = LAYOUT.world_pos_to_square(door_entrance);

        // walking over several tiles
        let walk_time = Character::Winnie.slow_step_time() * 6;

        vec![
            TakeAwayPlayerControl(player),
            SetActorFacingDirection(player, GridDirection::Top),
            // open the elevator door
            InsertAtlasAnimationTimerTo {
                entity: door,
                duration: from_millis(50),
                mode: TimerMode::Repeating,
            },
            Sleep(from_millis(400)),
            BeginSimpleWalkTo {
                with: player,
                square: walk_towards + sq(0, 3),
                planned: None,
                step_time: Some(walk_time),
            },
            Sleep(from_millis(50)),
            // fades out the player sprite
            BeginColorInterpolation {
                who: player,
                to: Color::NONE,
                over: 3 * walk_time / 4,
                animation_curve: None,
            },
            WaitUntilActorAtRest(player),
            Sleep(from_millis(150)),
            ChangeGlobalState {
                to: change_global_state_to,
                with: transition,
                change_loading_screen_state_to_start: true,
                loading_screen: Some(loading_screen),
            },
        ]
    }
}
