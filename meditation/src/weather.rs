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
    commands.spawn((
        controls::Normal::default(),
        Velocity::default(),
        AngularVelocity::default(), // for animation
        sprite::Transition::default(),
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/weather/atlas.png"),
                Vec2::new(consts::SPRITE_WIDTH, consts::SPRITE_HEIGHT),
                consts::SPRITE_ATLAS_COLS,
                consts::SPRITE_ATLAS_ROWS,
                Some(consts::SPRITE_ATLAS_PADDING),
                None,
            )),
            sprite: TextureAtlasSprite {
                index: sprite::Kind::default().index(),
                ..default()
            },
            ..default()
        },
    ));
}

#[derive(Event, Clone, Copy)]
pub(crate) enum ActionEvent {
    StartLoadingSpecial,
    LoadedSpecial {
        // fired or canceled?
        fired: bool,
    },
    Dipped,
    DashedAgainstVelocity {
        /// dashed in this direction while velocity was in the opposite
        towards: Direction,
    },
}
