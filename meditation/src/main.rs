//! Player controls weather sprite.
//!
//! The controls are WASD (or arrow keys) to move and space to activate special.
//! The sprite should feel floaty as if you were playing Puff in Smashbros.

#![allow(clippy::assertions_on_constants)]

mod background;
mod prelude;
mod weather;

use bevy_pixel_camera::{PixelCameraPlugin, PixelViewport, PixelZoom};
use prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::WARN,
                    filter: "meditation=trace".to_string(),
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PixelCameraPlugin)
        .insert_resource(ClearColor(Color::hex("#0d0e1f").unwrap()))
        .add_event::<weather::ActionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                weather::controls::normal,
                weather::controls::loading_special,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                apply_velocity,
                advance_animation,
                weather::anim::rotate,
                background::twinkle,
                background::shooting_star,
                weather::anim::apply_bloom,
                weather::anim::sprite,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn((
        Camera2dBundle::default(),
        weather::anim::CameraState::default(),
        PixelZoom::Fixed(3),
        PixelViewport,
    ));

    background::spawn(&mut commands, &asset_server, &mut texture_atlases);
    weather::spawn(&mut commands, &asset_server, &mut texture_atlases);

    // TODO: for some reason the last sprite to be spawned defies the rules
    // and is put under everything else instead of on top.
    commands.spawn((SpriteBundle {
        texture: asset_server.load("textures/bg/default.png"),
        ..Default::default()
    },));
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let d = time.delta_seconds();

    for (mut transform, vel) in &mut query {
        // TODO
        transform.translation.x += vel.x * d / 2.0;
        transform.translation.y += vel.y * d / 2.0;
    }
}

fn advance_animation(
    mut query: Query<(
        Entity,
        &Animation,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut Visibility,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, indices, mut timer, mut sprite, mut visibility) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                if !indices.should_repeat_when_played {
                    commands.entity(entity).remove::<AnimationTimer>();
                    *visibility = Visibility::Hidden;
                }

                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}
