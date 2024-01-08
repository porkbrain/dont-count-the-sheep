//! Hoshi is an entity that is controlled by the player.

mod anim;
mod arrow;
pub(crate) mod consts;
mod controls;
mod mode;
mod sprite;

use bevy::render::view::RenderLayers;
use bevy_magic_light_2d::gi::types::LightOccluder2D;
pub(crate) use controls::loading_special;

use self::consts::*;
use crate::{cameras::OBJ_RENDER_LAYER, prelude::*};

#[derive(Event, Clone, Copy)]
pub(crate) enum ActionEvent {
    StartLoadingSpecial {
        /// Where was the Hoshi when the special was started.
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

#[derive(Component)]
pub(crate) struct Hoshi;
#[derive(Component)]
struct HoshiBody;
#[derive(Component)]
struct HoshiFace;

/// Any entity spawned by this plugin has this component.
/// Useful for despawning.
#[derive(Component)]
struct HoshiEntity;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionEvent>()
            .add_systems(
                OnEnter(GlobalGameState::MeditationLoading),
                (spawn, arrow::spawn),
            )
            .add_systems(OnExit(GlobalGameState::MeditationQuitting), despawn)
            .add_systems(
                Update,
                (
                    anim::rotate,
                    arrow::point_arrow,
                    anim::sprite_loading_special,
                    controls::normal,
                    controls::loading_special,
                    anim::update_camera_on_special.after(controls::normal),
                    anim::sprite
                        .after(controls::normal)
                        .after(controls::loading_special),
                )
                    .run_if(in_state(GlobalGameState::MeditationInGame)),
            );
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

/// 1. spriteless parent which commands the movement
/// 2. body sprite, child of parent
/// 3. face sprite, child of parent
/// 4. spark effect is hidden by default and shown when special is fired
/// 5. setup camera state which is affected by going into special
fn spawn(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
) {
    debug!("Spawning Hoshi entities");

    //
    // 1.
    //
    let parent = commands
        .spawn((
            Hoshi,
            HoshiEntity,
            mode::Normal::default(),
            Velocity::default(),
            AngularVelocity::default(), // for animation
            sprite::Transition::default(),
            SpatialBundle {
                transform: DEFAULT_TRANSFORM,
                ..default()
            },
        ))
        .insert(LightOccluder2D {
            h_size: OCCLUDER_SIZE,
        })
        .id();
    //
    // 2.
    //
    let body = commands
        .spawn((
            HoshiBody,
            RenderLayers::layer(OBJ_RENDER_LAYER),
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load(assets::BODY_ATLAS),
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
            HoshiFace,
            RenderLayers::layer(OBJ_RENDER_LAYER),
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load(assets::FACE_ATLAS),
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
        HoshiEntity,
        RenderLayers::layer(OBJ_RENDER_LAYER),
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
                asset_server.load(assets::SPARK_ATLAS),
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
    commands.spawn((HoshiEntity, anim::CameraState::default()));
}

fn despawn(mut commands: Commands, entities: Query<Entity, With<HoshiEntity>>) {
    debug!("Spawning Hoshi entities");

    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}