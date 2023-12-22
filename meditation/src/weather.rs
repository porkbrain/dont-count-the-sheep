//! Weather is an entity that is controlled by the player.
//! It's called weather because it has to follow the climate entity around the
//! screen but is somewhat free.
//! Reminds me of the analogy made by Niel deGrasse Tyson.

pub(crate) mod anim;
pub(crate) mod arrow;
pub(crate) mod consts;
pub(crate) mod controls;
mod sprite;

use bevy_magic_light_2d::gi::types::{LightOccluder2D, OmniLightSource2D};

use crate::{control_mode, prelude::*};

use self::consts::*;

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
        at_translation: Vec2,
    },
    Jumped,
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
/// 6. setup camera state which is affected by going into special
pub(crate) fn spawn(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
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
                transform: DEFAULT_TRANSFORM,
                ..default()
            },
        ))
        .insert(LightOccluder2D {
            h_size: Vec2::new(15.0, 15.0), // TODO
        })
        .insert(OmniLightSource2D {
            intensity: 0.25,
            color: Color::hex("#fff4b3").unwrap(), // TODO
            falloff: Vec3::new(10.0, 10.0, 0.05),
            ..default()
        })
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
                    vec2(BODY_WIDTH, BODY_HEIGHT),
                    BODY_ATLAS_COLS,
                    BODY_ATLAS_ROWS,
                    Some(BODY_ATLAS_PADDING),
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
                    vec2(FACE_SPRITE_WIDTH, FACE_SPRITE_HEIGHT),
                    FACE_ATLAS_COLS,
                    FACE_ATLAS_ROWS,
                    Some(FACE_ATLAS_PADDING),
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
            last: SPARK_FRAMES - 1,
        },
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/weather/spark_atlas.png"),
                Vec2::splat(SPARK_SIDE),
                SPARK_FRAMES,
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
    arrow::spawn(&mut commands, &asset_server);
    //
    // 6.
    //
    commands.spawn(anim::CameraState::default());
}
