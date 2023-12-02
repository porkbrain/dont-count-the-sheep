//! Weather is an entity that is controlled by the player.
//! It's called weather because it has to follow the climate entity around the
//! screen but is somewhat free.
//! Reminds me of the analogy made by Niel deGrasse Tyson.

pub(crate) mod anim;
mod consts;
pub(crate) mod controls;
mod sprite;

use crate::prelude::*;

pub(crate) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/weather/atlas.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(35.0, 35.0),
        consts::SPRITE_ATLAS_COLS,
        consts::SPRITE_ATLAS_ROWS,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn((
        controls::Normal::default(),
        Velocity::default(),
        AngularVelocity::default(), // for animation
        sprite::Transition::default(),
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(sprite::Kind::default().index()),
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
