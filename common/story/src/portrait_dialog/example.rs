use bevy::prelude::{AssetServer, Commands};
use common_story_derive::*;

use super::{DialogFragment, Step};
use crate::{
    portrait_dialog::{aaatargets::DialogTarget, spawn},
    Character,
};

#[derive(Portrait)]
pub struct Example1;
impl DialogFragment for Example1 {
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
            Step::Text {
                speaker: Character::Bolt,
                content: "I'm just here to test the dialog system.",
            },
            Step::Text {
                speaker: Character::Capy,
                content: "I'm just here to test the dialog system.",
            },
            Step::GoTo {
                story_point: DialogTarget::Example2,
            },
        ]
    }
}

pub(super) struct Example2;
impl DialogFragment for Example2 {
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
                between: vec![DialogTarget::Example3, DialogTarget::Example4],
            },
        ]
    }
}

pub(super) struct Example3;
impl DialogFragment for Example3 {
    fn choice() -> &'static str {
        "inquire about the weather"
    }

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
            Step::GoTo {
                story_point: DialogTarget::Example4,
            },
        ]
    }
}

pub(super) struct Example4;
impl DialogFragment for Example4 {
    fn choice() -> &'static str {
        "subtly ask for candy"
    }

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
