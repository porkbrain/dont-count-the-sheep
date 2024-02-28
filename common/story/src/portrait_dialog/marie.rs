//! Dialogs with Marie.

use bevy::reflect::TypePath;
use common_story_derive::Portrait;

use super::{AsChoice, AsSequence, Step};
use crate::{portrait_dialog::aaatargets::DialogTargetChoice, Character};

/// Dialog that starts when the player enters the elevator.
#[derive(Portrait, TypePath)]
pub struct MarieBlabbering; // TODO: spoke before?
impl AsSequence for MarieBlabbering {
    fn sequence() -> Vec<Step> {
        vec![Step::Choice {
            speaker: Character::Marie,
            content: "My husband, he was one beautiful eucaryote.",
            between: vec![
                DialogTargetChoice::IamSorryForYourLoss,
                DialogTargetChoice::WhatHappenedToYourHusband,
            ],
        }]
    }
}

#[derive(TypePath)]
pub(crate) struct WhatHappenedToYourHusband; // TODO: asked before?
impl AsChoice for WhatHappenedToYourHusband {
    fn choice() -> &'static str {
        "Was? What happened to him?"
    }
}
impl AsSequence for WhatHappenedToYourHusband {
    fn sequence() -> Vec<Step> {
        // TODO: if asked before, start with "As I said"
        // and then pick a random answer
        vec![Step::Text {
            speaker: Character::Marie,
            content: "He was eaten by a pack of wild trucks.",
        }]
    }
}

#[derive(TypePath)]
pub(crate) struct IamSorryForYourLoss;
impl AsChoice for IamSorryForYourLoss {
    fn choice() -> &'static str {
        "I'm sorry for your loss."
    }
}
impl AsSequence for IamSorryForYourLoss {
    fn sequence() -> Vec<Step> {
        vec![
            Step::Text {
                speaker: Character::Marie,
                content: "And I am sorry for yours child.",
            },
            Step::Text {
                speaker: Character::Winnie,
                content: "Uhm... ok?",
            },
        ]
    }
}
