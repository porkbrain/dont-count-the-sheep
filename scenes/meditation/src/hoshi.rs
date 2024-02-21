//! Hoshi is an entity that is controlled by the player.

mod anim;
mod arrow;
pub(crate) mod consts;
mod controls;
mod mode;
mod sprite;

use bevy::render::view::RenderLayers;
use bevy_magic_light_2d::gi::types::LightOccluder2D;
use common_visuals::camera::render_layer;
pub(crate) use controls::loading_special;

use self::consts::*;
use crate::prelude::*;

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
}

/// 1. spriteless parent which commands the movement
/// 2. body sprite, child of parent
/// 3. face sprite, child of parent
/// 4. spark effect is hidden by default and shown when special is fired
/// 5. setup camera state which is affected by going into special
fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    debug!("Spawning Hoshi entities");

    //
    // 1.
    //
    let parent = cmd
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
    let body = cmd
        .spawn((
            HoshiBody,
            RenderLayers::layer(render_layer::OBJ),
            SpriteSheetBundle {
                texture: asset_server.load(assets::BODY_ATLAS),
                atlas: TextureAtlas {
                    index: sprite::BodyKind::default().index(),
                    layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                        vec2(BODY_WIDTH, BODY_HEIGHT),
                        BODY_ATLAS_COLS,
                        BODY_ATLAS_ROWS,
                        Some(BODY_ATLAS_PADDING),
                        None,
                    )),
                },
                ..default()
            },
        ))
        .id();
    cmd.entity(parent).add_child(body);
    //
    // 3.
    //
    let face = cmd
        .spawn((
            HoshiFace,
            RenderLayers::layer(render_layer::OBJ),
            SpriteSheetBundle {
                texture: asset_server.load(assets::FACE_ATLAS),
                atlas: TextureAtlas {
                    index: sprite::FaceKind::default().index(),
                    layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                        vec2(FACE_SPRITE_WIDTH, FACE_SPRITE_HEIGHT),
                        FACE_ATLAS_COLS,
                        FACE_ATLAS_ROWS,
                        Some(FACE_ATLAS_PADDING),
                        None,
                    )),
                },
                ..default()
            },
        ))
        .id();
    cmd.entity(parent).add_child(face);
    //
    // 4.
    //
    cmd.spawn((
        anim::SparkEffect,
        HoshiEntity,
        RenderLayers::layer(render_layer::OBJ),
        AtlasAnimation {
            on_last_frame: AtlasAnimationEnd::Custom(Box::new(
                |entity,
                 _animation,
                 _timer,
                 atlas,
                 visibility,
                 commands,
                 _time| {
                    *visibility = Visibility::Hidden;
                    commands.entity(entity).remove::<AtlasAnimationTimer>();
                    atlas.index = 0;
                },
            )),
            last: SPARK_FRAMES - 1,
            ..default()
        },
        SpriteSheetBundle {
            texture: asset_server.load(assets::SPARK_ATLAS),
            atlas: TextureAtlas {
                index: 0,
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    Vec2::splat(SPARK_SIDE),
                    SPARK_FRAMES,
                    1,
                    None,
                    None,
                )),
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
    //
    // 5.
    //
    cmd.spawn((HoshiEntity, anim::CameraState::default()));
}

fn despawn(mut cmd: Commands, entities: Query<Entity, With<HoshiEntity>>) {
    debug!("Spawning Hoshi entities");

    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
