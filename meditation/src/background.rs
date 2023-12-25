//! The background of the game comprises a starry sky and a shooting star.
//!
//! TODO: lower light intensity of stars and galaxies as more distractions come
//! in, and increase hellish red light from the edges or edge distractions.

use bevy::math::vec3;
use bevy_magic_light_2d::gi::types::OmniLightSource2D;

use crate::prelude::*;

pub(crate) const COLOR: &str = "#0d0e1f";
const STAR_LIGHT_COLOR: &str = "#dbcbff";

pub(crate) const TWINKLE_DURATION: Duration = from_millis(250);
pub(crate) const TWINKLE_CHANCE_PER_SECOND: f32 = 1.0 / 8.0;
pub(crate) const TWINKLE_COUNT: usize = 4;

pub(crate) const SHOOTING_STAR_CHANCE_PER_SECOND: f32 = 1.0 / 10.0;
pub(crate) const SHOOTING_STAR_FRAMES: usize = 4;
pub(crate) const SHOOTING_STAR_FRAME_TIME: Duration = from_millis(50);
pub(crate) const SHOOTING_STAR_WIDTH: f32 = 35.0;
pub(crate) const SHOOTING_STAR_HEIGHT: f32 = 35.0;
pub(crate) const SHOOTING_STAR_POSITION: Vec2 = vec2(-180.0, 50.0);

pub(crate) fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn((SpriteBundle {
        texture: asset_server.load("textures/bg/default.png"),
        transform: Transform::from_translation(Vec3::new(
            0.0,
            0.0,
            zindex::MAIN_BACKGROUND,
        )),
        ..default()
    },));

    spawn_twinkles(&mut commands, &asset_server);
    spawn_light_sources(&mut commands);
    spawn_shooting_star(&mut commands, &asset_server, &mut texture_atlases);
}

fn spawn_twinkles(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    for i in 1..=TWINKLE_COUNT {
        commands.spawn((
            Flicker::new(TWINKLE_CHANCE_PER_SECOND, TWINKLE_DURATION),
            SpriteBundle {
                texture: asset_server
                    .load(format!("textures/bg/twinkle{i}.png")),
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
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    let animation = Animation {
        // we schedule it at random
        on_last_frame: AnimationEnd::RemoveTimer,
        first: 0,
        last: SHOOTING_STAR_FRAMES - 1,
    };
    commands.spawn((
        BeginAnimationAtRandom {
            chance_per_second: SHOOTING_STAR_CHANCE_PER_SECOND,
            frame_time: SHOOTING_STAR_FRAME_TIME,
        },
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/bg/shootingstar_atlas.png"),
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
fn spawn_light_sources(commands: &mut Commands) {
    // top right star
    commands.spawn((
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
    commands.spawn((
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

    const GALAXY_LIGHT_INTENSITY: f32 = 0.25;

    // bottom left galaxy
    commands.spawn((
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
    commands.spawn((
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
    commands.spawn((
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
