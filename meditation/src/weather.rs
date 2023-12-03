//! Weather is an entity that is controlled by the player.
//! It's called weather because it has to follow the climate entity around the
//! screen but is somewhat free.
//! Reminds me of the analogy made by Niel deGrasse Tyson.

pub(crate) mod anim;
mod consts;
pub(crate) mod controls;
mod sprite;

use crate::prelude::*;

use self::anim::SparkEffect;

pub(crate) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    let parent = commands
        .spawn((
            controls::Normal::default(),
            Velocity::default(),
            AngularVelocity::default(), // for animation
            sprite::Transition::default(),
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    zindex::WEATHER,
                )),
                ..default()
            },
        ))
        .id();

    let body = commands
        .spawn((
            WeatherBody,
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("textures/weather/body_atlas.png"),
                    Vec2::new(consts::BODY_WIDTH, consts::BODY_HEIGHT),
                    consts::BODY_ATLAS_COLS,
                    consts::BODY_ATLAS_ROWS,
                    Some(consts::BODY_ATLAS_PADDING),
                    None,
                )),
                sprite: TextureAtlasSprite {
                    index: sprite::BodyKind::default().index(),
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(body);

    let face = commands
        .spawn((
            WeatherFace,
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("textures/weather/face_atlas.png"),
                    Vec2::new(consts::FACE_WIDTH, consts::FACE_HEIGHT),
                    consts::FACE_ATLAS_COLS,
                    consts::FACE_ATLAS_ROWS,
                    Some(consts::FACE_ATLAS_PADDING),
                    None,
                )),
                sprite: TextureAtlasSprite {
                    index: sprite::FaceKind::default().index(),
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(face);

    commands.spawn((
        SparkEffect,
        Animation {
            on_last_frame: AnimationEnd::Custom(Box::new(
                |entity,
                 _animation,
                 _timer,
                 atlas,
                 visibility,
                 commands,
                 _time| {
                    *visibility = Visibility::Hidden;
                    commands.entity(entity).remove::<AnimationTimer>();
                    atlas.index = 0;
                },
            )),
            first: 0,
            last: consts::SPARK_FRAMES - 1,
        },
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/weather/spark_atlas.png"),
                Vec2::splat(consts::SPARK_SIDE),
                consts::SPARK_FRAMES,
                1,
                None,
                None,
            )),
            sprite: TextureAtlasSprite::new(0),
            ..default()
        },
    ));
}

#[derive(Component)]
pub(crate) struct WeatherBody;

#[derive(Component)]
pub(crate) struct WeatherFace;

#[derive(Event, Clone, Copy)]
pub(crate) enum ActionEvent {
    StartLoadingSpecial,
    FiredSpecial,
    Dipped,
    DashedAgainstVelocity {
        /// dashed in this direction while velocity was in the opposite
        towards: MotionDirection,
    },
}
