//! Example dialog.

use bevy::prelude::{AssetServer, Commands};
use common_story_derive::*;

use super::{AsChoice, AsSequence, Step};
use crate::{
    portrait_dialog::{aaatargets::DialogTargetChoice, spawn},
    Character,
};

/// This is how the dialog starts.
#[derive(Portrait)]
pub struct Example1;
impl AsSequence for Example1 {
    fn sequence() -> Vec<Step> {
        vec![
            Step::Text {
                speaker: Character::Winnie,
                content: "First short",
            },
            Step::Text {
                speaker: Character::Phoebe,
                content: "Second longer text almost a sentence in its length.",
            },
            Step::Text {
                speaker: Character::Marie,
                content: "I'm just here to test the dialog system.",
            },
            Step::Text {
                speaker: Character::Master,
                content: "Oh, I see. Well, I hope you enjoy it!",
            },
            Step::Text {
                speaker: Character::Otter,
                content: "I will, thank you!",
            },
            Step::Text {
                speaker: Character::Unnamed,
                content: "I'm just here to test the dialog system.",
            },
            Step::Choice {
                between: vec![
                    DialogTargetChoice::Example2,
                    DialogTargetChoice::Example3,
                    DialogTargetChoice::Example4,
                ],
            },
        ]
    }
}

pub(super) struct Example2;
impl AsChoice for Example2 {
    fn choice() -> &'static str {
        "tell me more about all the characters there are"
    }
}
impl AsSequence for Example2 {
    fn sequence() -> Vec<Step> {
        vec![
            Step::Text {
                speaker: Character::Cat,
                content: "I'm just here to test the dialog system.",
            },
            Step::Text {
                speaker: Character::Emil,
                content: "I'm just here to test the dialog system.",
            },
            Step::Text {
                speaker: Character::Pooper,
                content: "Haf haf\nYummy bones give me bones",
            },
            Step::Text {
                speaker: Character::Redhead,
                content: "I'm just here to test the dialog system.",
            },
            Step::Text {
                speaker: Character::Winnie,
                content: include_str!("example_lorem.txt"),
            },
            Step::Choice {
                between: vec![
                    DialogTargetChoice::Example3,
                    DialogTargetChoice::Example4,
                ],
            },
        ]
    }
}

pub(super) struct Example3;
impl AsChoice for Example3 {
    fn choice() -> &'static str {
        "inquire about the weather"
    }
}
impl AsSequence for Example3 {
    fn sequence() -> Vec<Step> {
        vec![
            Step::Text {
                speaker: Character::Winnie,
                content: "Wat be da weather mah G",
            },
            Step::Text {
                speaker: Character::Phoebe,
                content: "It appears to be raining right now.",
            },
            Step::Text {
                speaker: Character::Phoebe,
                content: "On us...",
            },
            Step::Choice {
                between: vec![DialogTargetChoice::Example4],
            },
        ]
    }
}

pub(super) struct Example4;
impl AsChoice for Example4 {
    fn choice() -> &'static str {
        "subtly ask for candy"
    }
}
impl AsSequence for Example4 {
    fn sequence() -> Vec<Step> {
        vec![
            Step::Text {
                speaker: Character::Winnie,
                content: "Haev u got sum candy?",
            },
            Step::Text {
                speaker: Character::Phoebe,
                content: "I only have protein.",
            },
            Step::Text {
                speaker: Character::Winnie,
                content: "M'key",
            },
        ]
    }
}
