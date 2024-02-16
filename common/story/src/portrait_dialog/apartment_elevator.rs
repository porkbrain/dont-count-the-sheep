//! Dialogs within the elevator of the building where Winnie's apartment is.

use bevy::reflect::TypePath;
use common_story_derive::Portrait;

use super::{AsChoice, AsSequence, Step};
use crate::{
    portrait_dialog::aaatargets::{DialogTargetChoice, DialogTargetGoto},
    Character,
};

/// Dialog that starts when the player enters the elevator.
#[derive(Portrait, TypePath)]
pub struct EnteredTheElevator;
impl AsSequence for EnteredTheElevator {
    fn sequence() -> Vec<Step> {
        vec![Step::Choice {
            speaker: Character::Winnie,
            content: "Let's see, I'm going to ...",
            between: vec![
                DialogTargetChoice::TakeTheElevatorToGroundFloor,
                DialogTargetChoice::TakeTheElevatorToFirstFloor,
                DialogTargetChoice::LeaveTheElevatorToStayOnCurrentFloor,
            ],
        }]
    }
}

#[derive(TypePath)]
pub(crate) struct TakeTheElevatorToFirstFloor;
impl AsChoice for TakeTheElevatorToFirstFloor {
    fn choice() -> &'static str {
        "the first floor"
    }
}
impl AsSequence for TakeTheElevatorToFirstFloor {
    fn sequence() -> Vec<Step> {
        // TODO: Say "Okey, still not working ..." if this is not the first time
        // "The neighbors surely are thrilled"

        vec![
            Step::text(
                Character::Winnie,
                "... can't ... just ...\ncan you work you ...",
            ),
            Step::text(Character::Winnie, "Arrgh!"),
            Step::text(
                Character::Winnie,
                "Yep, that one is definitely not working.",
            ),
            Step::GoTo {
                story_point: DialogTargetGoto::EnteredTheElevator,
            },
        ]
    }
}

/// The player is going to the ground floor, presumably out.
#[derive(TypePath)]
pub struct TakeTheElevatorToGroundFloor;
impl AsChoice for TakeTheElevatorToGroundFloor {
    fn choice() -> &'static str {
        "the ground floor"
    }
}
impl AsSequence for TakeTheElevatorToGroundFloor {
    fn sequence() -> Vec<Step> {
        vec![]
    }
}

#[derive(TypePath)]
pub(crate) struct LeaveTheElevatorToStayOnCurrentFloor;
impl AsChoice for LeaveTheElevatorToStayOnCurrentFloor {
    fn choice() -> &'static str {
        "leave the elevator"
    }
}
impl AsSequence for LeaveTheElevatorToStayOnCurrentFloor {
    fn sequence() -> Vec<Step> {
        vec![]
    }
}
