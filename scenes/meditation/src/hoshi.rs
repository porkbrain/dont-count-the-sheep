//! Hoshi is an entity that is controlled by the player.
//!
//! While despawning is taken care of by [Plugin], spawning is done by calling
//! the [spawn] function.
//! That's because spawning is done via a Godot spawner.

mod anim;
mod arrow;
mod camera;
pub(crate) mod consts;
mod controls;
mod mode;
mod sprite;

use bevy::{math::uvec2, render::view::RenderLayers};
use common_loading_screen::LoadingScreenState;
use common_visuals::camera::render_layer;
use controls::HoshiControlsSystemSet;

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
                OnExit(LoadingScreenState::WaitForSignalToFinish),
                camera::spawn
                    .run_if(in_state(GlobalGameState::LoadingMeditation)),
            )
            .add_systems(
                OnEnter(GlobalGameState::LoadingMeditation),
                arrow::spawn,
            )
            .add_systems(
                OnExit(GlobalGameState::QuittingMeditation),
                (despawn, arrow::despawn),
            )
            .add_systems(
                Update,
                (controls::normal, controls::loading_special)
                    .in_set(HoshiControlsSystemSet),
            )
            .add_systems(
                Update,
                (
                    arrow::point_arrow,
                    anim::rotate,
                    anim::sprite_loading_special,
                    anim::sprite,
                    camera::zoom_on_special,
                    camera::follow_hoshi,
                )
                    .after(HoshiControlsSystemSet)
                    .run_if(in_state(GlobalGameState::InGameMeditation)),
            );
    }
}

/// Used to spawn Hoshi.
///
/// 1. spriteless parent which commands the movement
/// 2. body sprite, child of parent
/// 3. face sprite, child of parent
/// 4. spark effect is hidden by default and shown when special is fired
pub(crate) fn spawn(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    translation: Vec2,
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
                transform: Transform {
                    translation: translation.extend(zindex::HOSHI),
                    rotation: Quat::from_array([0.0, 0.0, 0.0, 1.0]),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                },
                ..default()
            },
        ))
        .id();
    //
    // 2.
    //
    let body = cmd
        .spawn((
            HoshiBody,
            RenderLayers::layer(render_layer::OBJ),
            SpriteBundle {
                texture: asset_server.load(assets::BODY_ATLAS),
                ..default()
            },
            TextureAtlas {
                index: sprite::BodyKind::default().index(),
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    uvec2(BODY_WIDTH as u32, BODY_HEIGHT as u32),
                    BODY_ATLAS_COLS as u32,
                    BODY_ATLAS_ROWS as u32,
                    Some(BODY_ATLAS_PADDING),
                    None,
                )),
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
            SpriteBundle {
                texture: asset_server.load(assets::FACE_ATLAS),
                ..default()
            },
            TextureAtlas {
                index: sprite::FaceKind::default().index(),
                layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                    uvec2(FACE_SPRITE_WIDTH as u32, FACE_SPRITE_HEIGHT as u32),
                    FACE_ATLAS_COLS as u32,
                    FACE_ATLAS_ROWS as u32,
                    Some(FACE_ATLAS_PADDING),
                    None,
                )),
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
            on_last_frame: AtlasAnimationEnd::RemoveTimerAndHideAndReset,
            last: SPARK_FRAMES - 1,
            ..default()
        },
        SpriteBundle {
            texture: asset_server.load(assets::SPARK_ATLAS),
            visibility: Visibility::Hidden,
            ..default()
        },
        TextureAtlas {
            index: 0,
            layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                UVec2::splat(SPARK_SIDE as u32),
                SPARK_FRAMES as u32,
                1,
                None,
                None,
            )),
        },
    ));
}

fn despawn(mut cmd: Commands, entities: Query<Entity, With<HoshiEntity>>) {
    debug!("Spawning Hoshi entities");

    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
