use bevy::time::Stopwatch;
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};

use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
    weather::{self, Weather},
};

const BLACKHOLE_FLICKER_CHANCE_PER_SECOND: f32 = 0.5;
const BLACKHOLE_FLICKER_DURATION: Duration = Duration::from_millis(100);
const LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL: f32 = 35.0;
const DISTRACTION_HEIGHT: f32 = 50.0;
const DISTRACTION_WIDTH: f32 = DISTRACTION_HEIGHT;
const HITBOX_WIDTH: f32 = 50.0;
const HITBOX_HEIGHT: f32 = HITBOX_WIDTH;
const HITBOX_SIZE: Vec2 = Vec2::new(HITBOX_WIDTH, HITBOX_HEIGHT);
const HITBOX_DISTANCE_TO_DISTRACTION: f32 = 0.0;
const DEFAULT_KICK: f32 = 5.0;

#[derive(Component)]
pub(crate) struct Distraction {
    level: Level,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

#[derive(Event)]
pub(crate) struct DistractionDestroyedEvent {
    pub(crate) level: Level,
    pub(crate) at_translation: Vec2,
}

pub(crate) fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    for i in 0..3 {
        let parent = commands
            .spawn((
                Distraction { level: Level::One },
                Velocity::new(Vec2::new(0.0, 0.0)),
                SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(
                        TextureAtlas::from_grid(
                            asset_server
                                .load("textures/distractions/frame.png"),
                            Vec2::new(50.0, 50.0),
                            5,
                            1,
                            None,
                            None,
                        ),
                    ),
                    sprite: TextureAtlasSprite {
                        index: 1,
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        100.0 - i as f32 * 100.0,
                        0.0,
                        0.0,
                    )),
                    ..default()
                },
            ))
            .id();

        let child = commands
            .spawn((bevy_webp_anim::WebpBundle {
                animation: asset_server
                    .load("textures/distractions/videos/1.webp"),
                frame_rate: bevy_webp_anim::FrameRate::new(2),
                sprite: Sprite { ..default() },
                ..default()
            },))
            .id();

        commands.entity(parent).add_child(child);
    }
}

pub(crate) fn react_to_weather(
    mut score: EventWriter<DistractionDestroyedEvent>,
    mut weather_actions: EventReader<weather::ActionEvent>,
    weather: Query<&Transform, (With<Weather>, Without<Distraction>)>,
    mut distraction: Query<
        (Entity, &Distraction, &Transform, &mut Velocity),
        Without<Weather>,
    >,
    mut commands: Commands,
) {
    let Some(action) = weather_actions.read().last() else {
        return;
    };

    let Ok(weather_transform) = weather.get_single() else {
        return;
    };
    let weather_translation = weather_transform.translation.truncate();

    for (entity, distraction, transform, mut vel) in distraction.iter_mut() {
        let translation = transform.translation.truncate();

        match (action, distraction.level) {
            (weather::ActionEvent::FiredSpecial, Level::One)
                if translation.distance(weather_translation)
                    < LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL =>
            {
                debug!("Distraction destroy event sent");
                score.send(DistractionDestroyedEvent {
                    level: Level::One,
                    at_translation: translation,
                });
                commands.entity(entity).despawn_recursive();

                continue;
            }
            _ => {}
        };

        let vel_factor = match action {
            weather::ActionEvent::StartLoadingSpecial { .. } => 0.0,
            weather::ActionEvent::DashedAgainstVelocity { .. } => 1.0,
            weather::ActionEvent::Dipped => 2.0,
            weather::ActionEvent::FiredSpecial => 5.0,
            // higher if bigger jumps
            weather::ActionEvent::Jumped { jumps_left } => {
                3.0 * (*jumps_left as f32 / weather::consts::MAX_JUMPS as f32)
            }
        };

        // There are 4 boxes around the distraction:
        // 1. above, 2. below, 3. left and 4. right.
        // if weather is in any of them, then distraction gets velocity in
        // opposite direction

        //
        // 1.
        //
        let box_above_center = translation
            + Vec2::new(
                0.0,
                DISTRACTION_HEIGHT / 2.0 + HITBOX_DISTANCE_TO_DISTRACTION,
            );
        let box_above = Rect::from_center_size(box_above_center, HITBOX_SIZE);
        if box_above.contains(weather_translation) {
            trace!("ABOVE {vel_factor:.2}");
            vel.y = -DEFAULT_KICK * vel_factor;
        } else {
            //
            // 2.
            //
            let box_below_center = translation
                + Vec2::new(
                    0.0,
                    -DISTRACTION_HEIGHT / 2.0 - HITBOX_DISTANCE_TO_DISTRACTION,
                );
            let box_below =
                Rect::from_center_size(box_below_center, HITBOX_SIZE);

            if box_below.contains(weather_translation) {
                trace!("below {vel_factor:.2}");
                vel.y = DEFAULT_KICK * vel_factor;
            }
        }
        //
        // 3.
        //
        let box_left_center = translation
            + Vec2::new(
                -DISTRACTION_WIDTH / 2.0 - HITBOX_DISTANCE_TO_DISTRACTION,
                0.0,
            );
        let box_left = Rect::from_center_size(box_left_center, HITBOX_SIZE);
        if box_left.contains(weather_translation) {
            trace!("LEft {vel_factor:.2}");
            vel.x = DEFAULT_KICK * vel_factor;
        } else {
            //
            // 4.
            //
            let box_right_center = translation
                + Vec2::new(
                    DISTRACTION_WIDTH / 2.0 + HITBOX_DISTANCE_TO_DISTRACTION,
                    0.0,
                );
            let box_right =
                Rect::from_center_size(box_right_center, HITBOX_SIZE);

            if box_right.contains(weather_translation) {
                trace!("rigHT {vel_factor:.2}");
                vel.x = -DEFAULT_KICK * vel_factor;
            }
        }
    }
}

#[derive(Component)]
pub(crate) struct BlackHole(GridCoords, Stopwatch);

pub(crate) fn destroyed(
    mut events: EventReader<DistractionDestroyedEvent>,
    mut score: Query<&mut crate::ui::Score>,
    mut gravity: EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut score = score.single_mut();

    for DistractionDestroyedEvent {
        level,
        at_translation,
    } in events.read()
    {
        *score += level.score();

        // TODO: animate out

        let gravity_grid_coords = PoissonsEquationUpdateEvent::send(
            &mut gravity,
            1.5, // TODO
            ChangeOfBasis::new(*at_translation),
        );

        trace!("Spawning black hole");
        commands
            .spawn((
                BlackHole(gravity_grid_coords, Stopwatch::new()),
                SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(
                        TextureAtlas::from_grid(
                            asset_server
                                .load("textures/distractions/blackhole.png"),
                            Vec2::new(100.0, 100.0), // TODO
                            5,
                            1,
                            None,
                            None,
                        ),
                    ),
                    sprite: TextureAtlasSprite {
                        index: 4,
                        ..default()
                    },
                    transform: Transform::from_translation(
                        at_translation.extend(zindex::BLACK_HOLE),
                    ),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Flicker::new(
                        BLACKHOLE_FLICKER_CHANCE_PER_SECOND,
                        BLACKHOLE_FLICKER_DURATION,
                    ),
                    SpriteBundle {
                        texture: asset_server.load(format!(
                            "textures/distractions/blackhole_flicker.png"
                        )),
                        transform: Transform::from_translation(Vec3::new(
                            0.0,
                            0.0,
                            zindex::BLACK_HOLE_TWINKLE,
                        )),
                        ..default()
                    },
                ));
            });
    }
}

impl Level {
    fn score(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 4,
            Self::Three => 32, // optimal level
            Self::Four => 32,  // stronger but no bonus
            Self::Five => 1,   // sucks to be you
        }
    }
}
