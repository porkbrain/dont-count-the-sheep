//! The background of the game comprises a starry sky and a shooting star.

use bevy::{math::vec3, render::view::RenderLayers};
use bevy_magic_light_2d::gi::types::OmniLightSource2D;

use crate::{
    cameras::{BackgroundLightScene, BG_RENDER_LAYER},
    prelude::*,
};

pub(crate) const COLOR: Color = PRIMARY_COLOR;
const STAR_LIGHT_COLOR: &str = "#dbcbff";

const TWINKLE_DURATION: Duration = from_millis(250);
const TWINKLE_CHANCE_PER_SECOND: f32 = 1.0 / 8.0;
const TWINKLE_COUNT: usize = 4;

const SHOOTING_STAR_CHANCE_PER_SECOND: f32 = 1.0 / 10.0;
const SHOOTING_STAR_FRAMES: usize = 4;
const SHOOTING_STAR_FRAME_TIME: Duration = from_millis(50);
const SHOOTING_STAR_WIDTH: f32 = 35.0;
const SHOOTING_STAR_HEIGHT: f32 = 35.0;
const SHOOTING_STAR_POSITION: Vec2 = vec2(-180.0, 50.0);

const GALAXY_LIGHT_INTENSITY: f32 = 0.25;

pub(crate) struct Plugin;

/// Identifies the background entities.
/// Useful for despawning.
#[derive(Component)]
struct BackgroundEntity;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::MeditationLoading), spawn)
            .add_systems(OnExit(GlobalGameState::MeditationQuitting), despawn);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    cmd.spawn((
        BackgroundEntity,
        RenderLayers::layer(BG_RENDER_LAYER),
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
    spawn_light_sources(&mut cmd);
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
            RenderLayers::layer(BG_RENDER_LAYER),
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
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    let animation = Animation {
        // we schedule it at random
        on_last_frame: AnimationEnd::RemoveTimer,
        first: 0,
        last: SHOOTING_STAR_FRAMES - 1,
    };
    cmd.spawn((
        BackgroundEntity,
        RenderLayers::layer(BG_RENDER_LAYER),
        BeginAnimationAtRandom {
            chance_per_second: SHOOTING_STAR_CHANCE_PER_SECOND,
            frame_time: SHOOTING_STAR_FRAME_TIME,
            ..default()
        },
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(assets::SHOOTING_STAR_ATLAS),
                vec2(SHOOTING_STAR_WIDTH, SHOOTING_STAR_HEIGHT),
                SHOOTING_STAR_FRAMES,
                1,
                None,
                None,
            )),
            sprite: TextureAtlasSprite::new(animation.first),
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(
                SHOOTING_STAR_POSITION.extend(zindex::SHOOTING_STARS),
            ),
            ..default()
        },
        animation,
    ));
}

/// Some stars emit light.
fn spawn_light_sources(cmd: &mut Commands) {
    // top right star
    cmd.spawn((
        BackgroundLightScene,
        SpatialBundle {
            transform: Transform::from_translation(vec3(-187.0, 122.0, 0.0)),
            ..default()
        },
        OmniLightSource2D {
            intensity: 0.5,
            color: Color::hex(STAR_LIGHT_COLOR).unwrap(),
            jitter_intensity: 0.5,
            falloff: Vec3::new(1.0, 1.0, 0.05),
            ..default()
        },
    ));

    // top left star
    cmd.spawn((
        BackgroundLightScene,
        SpatialBundle {
            transform: Transform::from_translation(vec3(235.0, 67.0, 0.0)),
            ..default()
        },
        OmniLightSource2D {
            intensity: 0.5,
            color: Color::hex(STAR_LIGHT_COLOR).unwrap(),
            jitter_intensity: 0.5,
            falloff: Vec3::new(1.0, 1.0, 0.05),
            ..default()
        },
    ));

    // bottom left galaxy
    cmd.spawn((
        BackgroundLightScene,
        SpatialBundle {
            transform: Transform::from_translation(vec3(140.0, -45.0, 0.0)),
            ..default()
        },
        OmniLightSource2D {
            intensity: GALAXY_LIGHT_INTENSITY,
            color: Color::hex(STAR_LIGHT_COLOR).unwrap(),
            falloff: Vec3::new(35.0, 35.0, 0.05),
            ..default()
        },
    ));

    // bottom right galaxy
    cmd.spawn((
        BackgroundLightScene,
        SpatialBundle {
            transform: Transform::from_translation(vec3(-280.0, -55.0, 0.0)),
            ..default()
        },
        OmniLightSource2D {
            intensity: GALAXY_LIGHT_INTENSITY,
            color: Color::hex(STAR_LIGHT_COLOR).unwrap(),
            falloff: Vec3::new(35.0, 35.0, 0.05),
            ..default()
        },
    ));

    // top center galaxy
    cmd.spawn((
        BackgroundLightScene,
        SpatialBundle {
            transform: Transform::from_translation(vec3(-20.0, 150.0, 0.0)),
            ..default()
        },
        OmniLightSource2D {
            intensity: GALAXY_LIGHT_INTENSITY,
            color: Color::hex(STAR_LIGHT_COLOR).unwrap(),
            falloff: Vec3::new(50.0, 50.0, 0.05),
            ..default()
        },
    ));
}
