use crate::prelude::*;
use bevy::utils::Instant;

mod consts {
    use std::time::Duration;

    pub(crate) const TWINKLE_DURATION: Duration = Duration::from_millis(250);
    pub(crate) const TWINKLE_CHANCE_PER_SECOND: f32 = 1.0 / 8.0;
    pub(crate) const TWINKLE_COUNT: usize = 4;

    pub(crate) const SHOOTING_STAR_CHANCE_PER_SECOND: f32 = 1.0 / 10.0;
    pub(crate) const SHOOTING_STAR_FRAMES: usize = 4;
    pub(crate) const SHOOTING_STAR_FRAME_TIME: Duration =
        Duration::from_millis(50);
    pub(crate) const SHOOTING_START_WIDTH: f32 = 35.0;
    pub(crate) const SHOOTING_START_HEIGHT: f32 = 35.0;
}

pub(crate) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    for i in 1..=consts::TWINKLE_COUNT {
        commands.spawn((
            Twinkle(Instant::now()),
            SpriteBundle {
                texture: asset_server
                    .load(format!("textures/bg/twinkle{i}.png")),
                ..Default::default()
            },
        ));
    }

    let animation = Animation {
        should_repeat_when_played: false, // we schedule it at random
        first: 0,
        last: consts::SHOOTING_STAR_FRAMES - 1,
    };
    commands.spawn((
        ShootingStar,
        SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/bg/shootingstar_atlas.png"),
                Vec2::new(
                    consts::SHOOTING_START_WIDTH,
                    consts::SHOOTING_START_HEIGHT,
                ),
                consts::SHOOTING_STAR_FRAMES,
                1,
                None,
                None,
            )),
            sprite: TextureAtlasSprite::new(animation.first),
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(Vec3::new(
                -180.0, 50.0, 0.0,
            )),
            ..default()
        },
        animation,
    ));
}

/// When did the twinkle start?
#[derive(Component, Deref)]
pub(crate) struct Twinkle(Instant);

pub(crate) fn twinkle(
    mut query: Query<(&mut Twinkle, &mut Visibility)>,
    time: Res<Time>,
) {
    for (mut twinkle, mut visibility) in &mut query {
        if matches!(*visibility, Visibility::Hidden) {
            if twinkle.elapsed() > consts::TWINKLE_DURATION {
                *visibility = Visibility::Visible;
            }
        } else if rand::random::<f32>()
            < consts::TWINKLE_CHANCE_PER_SECOND * time.delta_seconds()
        {
            twinkle.0 = Instant::now();
            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Component)]
pub(crate) struct ShootingStar;

pub(crate) fn shooting_star(
    mut query: Query<
        (Entity, &mut Visibility),
        (With<ShootingStar>, Without<AnimationTimer>),
    >,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut visibility) in &mut query {
        if rand::random::<f32>()
            < consts::SHOOTING_STAR_CHANCE_PER_SECOND * time.delta_seconds()
        {
            trace!("Watch out for the shooting star");
            *visibility = Visibility::Visible;
            commands.entity(entity).insert(AnimationTimer(Timer::new(
                consts::SHOOTING_STAR_FRAME_TIME,
                TimerMode::Repeating,
            )));
        }
    }
}
