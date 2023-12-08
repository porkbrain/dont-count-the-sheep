use crate::prelude::*;

pub(crate) const TWINKLE_DURATION: Duration = from_millis(250);
pub(crate) const TWINKLE_CHANCE_PER_SECOND: f32 = 1.0 / 8.0;
pub(crate) const TWINKLE_COUNT: usize = 4;

pub(crate) const SHOOTING_STAR_CHANCE_PER_SECOND: f32 = 1.0 / 10.0;
pub(crate) const SHOOTING_STAR_FRAMES: usize = 4;
pub(crate) const SHOOTING_STAR_FRAME_TIME: Duration = from_millis(50);
pub(crate) const SHOOTING_STAR_WIDTH: f32 = 35.0;
pub(crate) const SHOOTING_STAR_HEIGHT: f32 = 35.0;

pub(crate) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
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

    spawn_shooting_star(commands, asset_server, texture_atlases);
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
                Vec2::new(SHOOTING_STAR_WIDTH, SHOOTING_STAR_HEIGHT),
                SHOOTING_STAR_FRAMES,
                1,
                None,
                None,
            )),
            sprite: TextureAtlasSprite::new(animation.first),
            visibility: Visibility::Hidden,
            transform: Transform::from_translation(Vec3::new(
                -180.0,
                50.0,
                zindex::SHOOTING_STARS,
            )),
            ..default()
        },
        animation,
    ));
}
