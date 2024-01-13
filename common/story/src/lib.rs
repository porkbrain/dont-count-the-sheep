#![feature(round_char_boundary)]
#![feature(trivial_bounds)]

use bevy::reflect::Reflect;

pub mod portrait_dialog;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
enum Character {
    /// Main playable character
    Winnie,
    /// The Princess
    Phoebe,
    /// The Widow
    Marie,
    /// The cult head
    Master,

    Redhead,
    Bolt,
    Capy,
    Cat,
    Emil,
    Pooper,
    Unnamed,
    Otter,
}

impl Character {
    fn portrait_asset_path(&self) -> &'static str {
        use common_assets::paths::portraits::*;

        match self {
            Character::Winnie => WINNIE,
            Character::Phoebe => PHOEBE,
            Character::Marie => MARIE,
            Character::Bolt => BOLT,
            Character::Capy => CAPY,
            Character::Cat => CAT,
            Character::Emil => EMIL,
            Character::Master => MASTER,
            Character::Pooper => POOPER,
            Character::Redhead => REDHEAD,
            Character::Unnamed => UNNAMED,
            Character::Otter => OTTER,
        }
    }
}
