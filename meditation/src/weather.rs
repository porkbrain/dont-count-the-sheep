//! Weather is an entity that is controlled by the player.
//! It's called weather because it has to follow the climate entity around the
//! screen but is somewhat free.
//! Reminds me of the analogy made by Niel deGrasse Tyson.

pub(crate) mod anim;
pub(crate) mod arrow;
pub(crate) mod consts;
pub(crate) mod controls;
mod sprite;

use crate::{control_mode, prelude::*};

#[derive(Component)]
pub(crate) struct Weather;

#[derive(Component)]
pub(crate) struct WeatherBody;

#[derive(Component)]
pub(crate) struct WeatherFace;

#[derive(Event, Clone, Copy)]
pub(crate) enum ActionEvent {
    StartLoadingSpecial {
        /// Where was the weather when the special was started.
        from_translation: Vec2,
    },
    FiredSpecial,
    Dipped,
    DashedAgainstVelocity {
        /// dashed in this direction while velocity was in the opposite
        towards: MotionDirection,
    },
}

/// 1. spriteless parent which commands the movement
/// 2. body sprite, child of parent
/// 3. face sprite, child of parent
/// 4. spark effect is hidden by default and shown when special is fired
/// 5. arrow is hidden by default and shown when weather is off screen
pub(crate) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    //
    // 1.
    //
    let parent = commands
        .spawn((
            Weather,
            control_mode::Normal::default(),
            Velocity::default(),
            AngularVelocity::default(), // for animation
            sprite::Transition::default(),
            SpatialBundle {
                transform: consts::DEFAULT_TRANSFORM,
                ..default()
            },
        ))
        .id();
    //
    // 2.
    //
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
    //
    // 3.
    //
    let face = commands
        .spawn((
            WeatherFace,
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("textures/weather/face_atlas.png"),
                    Vec2::new(
                        consts::FACE_SPRITE_WIDTH,
                        consts::FACE_SPRITE_HEIGHT,
                    ),
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
    //
    // 4.
    //
    commands.spawn((
        anim::SparkEffect,
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
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
    //
    // 5.
    //
    commands.spawn((
        arrow::Arrow,
        SpriteBundle {
            texture: asset_server.load("textures/weather/arrow.png"),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::WEATHER_ARROW,
            )),
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}
