//! Weather is an entity that is controlled by the player.
//! It's called weather because it has to follow the climate entity around the
//! screen but is somewhat free.
//! Reminds me of the analogy made by Niel deGrasse Tyson.

pub(crate) mod anim;
mod consts;
pub(crate) mod controls;

use crate::prelude::*;

pub(crate) fn spawn(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        controls::Normal::default(),
        Velocity::default(),
        AngularVelocity::default(), // for animation
        SpriteBundle {
            texture: asset_server.load("textures/weather/default.png"),
            ..default()
        },
    ));
}

pub(crate) mod event {
    use bevy::prelude::Event;

    #[derive(Event)]
    pub(crate) struct StartLoadingSpecial;

    #[derive(Event)]
    pub(crate) struct LoadedSpecial {
        // fired or canceled?
        pub(crate) fired: bool,
    }
}
