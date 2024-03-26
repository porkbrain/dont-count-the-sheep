//! Things that player can encounter in this scene.

use crate::prelude::*;

#[derive(Event, Reflect, Clone, strum::EnumString)]
pub enum DowntownAction {}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, _: &mut App) {}
}
