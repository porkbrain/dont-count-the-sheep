//! The background of the game comprises a starry sky and a shooting star.

use bevy::{math::uvec2, render::view::RenderLayers};
use common_visuals::camera::render_layer;

use crate::prelude::*;

const TWINKLE_DURATION: Duration = from_millis(250);
const TWINKLE_CHANCE_PER_SECOND: f32 = 1.0 / 8.0;
const TWINKLE_COUNT: usize = 4;

const SHOOTING_STAR_CHANCE_PER_SECOND: f32 = 1.0 / 10.0;
const SHOOTING_STAR_FRAMES: usize = 4;
const SHOOTING_STAR_FRAME_TIME: Duration = from_millis(50);
const SHOOTING_STAR_WIDTH: u32 = 35;
const SHOOTING_STAR_HEIGHT: u32 = 35;
const SHOOTING_STAR_POSITION: Vec2 = vec2(-180.0, 50.0);

pub(crate) struct Plugin;

/// Identifies the background entities.
/// Useful for despawning.
#[derive(Component)]
struct BackgroundEntity;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::LoadingMeditation), spawn)
            .add_systems(OnExit(GlobalGameState::QuittingMeditation), despawn);
    }
}

fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    cmd.spawn((
        BackgroundEntity,
        RenderLayers::layer(render_layer::BG),
        SpriteBundle {
            texture: asset_server.load(assets::BACKGROUND_DEFAULT),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::MAIN_BACKGROUND,
            )),
            ..default()
        },
    ));

    spawn_twinkles(&mut cmd, &asset_server);
    spawn_shooting_star(&mut cmd, &asset_server, &mut texture_atlases);
}

fn despawn(mut cmd: Commands, bg: Query<Entity, With<BackgroundEntity>>) {
    for entity in bg.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

fn spawn_twinkles(cmd: &mut Commands, asset_server: &Res<AssetServer>) {
    for i in 1..=TWINKLE_COUNT {
        cmd.spawn((
            BackgroundEntity,
            RenderLayers::layer(render_layer::OBJ),
            Flicker::new(TWINKLE_CHANCE_PER_SECOND, TWINKLE_DURATION),
            SpriteBundle {
                texture: asset_server.load(assets::twinkle(i)),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    zindex::TWINKLES,
                )),
                ..default()
            },
        ));
    }
}

fn spawn_shooting_star(
    cmd: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let animation = AtlasAnimation {
        // we schedule it at random
        on_last_frame: AtlasAnimationEnd::RemoveTimerAndHideAndReset,
        last: SHOOTING_STAR_FRAMES - 1,
        ..default()
    };
    cmd.spawn((
        BackgroundEntity,
        RenderLayers::layer(render_layer::OBJ),
        BeginAtlasAnimation {
            cond: common_visuals::BeginAtlasAnimationCond::AtRandom(
                SHOOTING_STAR_CHANCE_PER_SECOND,
            ),
            frame_time: SHOOTING_STAR_FRAME_TIME,
            ..default()
        },
        SpriteBundle {
            texture: asset_server.load(assets::SHOOTING_STAR_ATLAS),
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(
                SHOOTING_STAR_POSITION.extend(zindex::SHOOTING_STARS),
            ),
            ..default()
        },
        TextureAtlas {
            index: animation.first,
            layout: texture_atlases.add(TextureAtlasLayout::from_grid(
                uvec2(SHOOTING_STAR_WIDTH, SHOOTING_STAR_HEIGHT),
                SHOOTING_STAR_FRAMES as u32,
                1,
                None,
                None,
            )),
        },
        animation,
    ));
}
