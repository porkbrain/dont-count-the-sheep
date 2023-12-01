use crate::prelude::*;
use bevy::utils::Instant;
use bevy_pixel_camera::{PixelViewport, PixelZoom};

mod consts {
    use std::time::Duration;

    pub(crate) const TWINKLE_DURATION: Duration = Duration::from_millis(250);
    pub(crate) const TWINKLE_CHANCE_PER_SECOND: f32 = 1.0 / 8.0;
    pub(crate) const TWINKLE_COUNT: usize = 4;
}

pub(crate) fn spawn_main(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    commands.spawn((MainCamera, State::default(), MainCamera::bundle()));
    commands.spawn(SpriteBundle {
        texture: asset_server.load("textures/bg/default.png"),
        ..Default::default()
    });

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
}

#[derive(Component)]
pub(crate) struct MainCamera;

/// When did the twinkle start?
#[derive(Component, Deref)]
pub(crate) struct Twinkle(Instant);

#[derive(Component, Default, Clone, Copy)]
pub(crate) enum State {
    #[default]
    Normal,
    BloomGoingDown {
        until: Instant,
    },
    BloomGoingUp,
}

impl MainCamera {
    fn bundle() -> impl Bundle {
        (
            Camera2dBundle::default(),
            PixelZoom::Fixed(3),
            PixelViewport,
        )
    }
}

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
