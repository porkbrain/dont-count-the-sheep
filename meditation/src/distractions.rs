use std::f32::consts::PI;

use bevy::time::Stopwatch;
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};

use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
    weather::{self, Weather},
};

use crate::path::LevelPath;

const BLACKHOLE_FLICKER_CHANCE_PER_SECOND: f32 = 0.5;
const BLACKHOLE_FLICKER_DURATION: Duration = Duration::from_millis(100);
const LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL: f32 = 35.0;

const DISTRACTION_SIZE: f32 = 50.0;
/// The default angular velocity kick applied to distractions when the weather
/// interacts with them.
/// Scaled by the action factor (some actions increase it.)
const DEFAULT_ANGVEL_KICK: f32 = PI;
/// If the dot product of the velocity of the distraction and the velocity of
/// the weather is less than this threshold, then the distraction is affected
/// by the weather.
/// The rotation of the distraction is increased by the action factor.
const PERPENDICULARITY_THRESHOLD: f32 = PI / 8.0;
/// Weather must be within this distance of the distraction to affect it.
const HITBOX_RADIUS: f32 = DISTRACTION_SIZE * 1.25;
/// Every second, the angular velocity of distractions is reduced.
/// How many seconds does it take to stop rotating?
const ROTATION_STOP_IN_SECS: f32 = 15.0;

#[derive(Component)]
pub(crate) struct Distraction {
    level: Level,
    current_path_since: Stopwatch,
    path: LevelPath,
    transition_into: Option<LevelPath>,
}

impl Distraction {
    pub(crate) fn new() -> Self {
        Self {
            level: Level::One,
            path: LevelPath::random_intro(),
            current_path_since: Stopwatch::new(),
            transition_into: None,
        }
    }
}

// TODO: score perhaps a distance to the center of the screen
#[allow(dead_code)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    #[default]
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
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
) {
    commands
        .spawn((
            Distraction::new(),
            AngularVelocity::default(),
            SpatialBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn(SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("textures/distractions/frame.png"),
                    vec2(DISTRACTION_SIZE, DISTRACTION_SIZE),
                    5,
                    1,
                    None,
                    None,
                )),
                sprite: TextureAtlasSprite {
                    index: 1, // TODO: random
                    ..default()
                },
                // above the video
                transform: Transform::from_translation(Vec3::new(
                    0.0, 0.0, 1.0,
                )),
                ..default()
            });

            // TODO: vary videos
            parent.spawn(bevy_webp_anim::WebpBundle {
                animation: asset_server
                    .load("textures/distractions/videos/1.webp"),
                frame_rate: bevy_webp_anim::FrameRate::new(2),
                sprite: Sprite { ..default() },
                ..default()
            });

            // TODO: sound
        });
}

/// TODO: dedup with climate
pub(crate) fn follow_curve(
    mut distraction: Query<(&mut Distraction, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut distraction, mut transform) in distraction.iter_mut() {
        distraction.current_path_since.tick(time.delta());

        let z = transform.translation.z;
        let (seg_index, seg_t) = distraction.path_segment();

        let at_least_one_lap = distraction.laps() > 0;
        let at_lap_beginning = seg_index == 0 && seg_t < 2. / 60.;
        let ready_to_transition = distraction.transition_into.is_some();

        if at_lap_beginning && at_least_one_lap && ready_to_transition {
            distraction.current_path_since.reset();
            distraction.path = distraction.transition_into.take().unwrap();
        } else if !ready_to_transition {
            // roll a dice to see if distraction levels up
            let should_level_up = rand::random::<f32>() < 0.8; // TODO
            distraction.transition_into =
                Some(distraction.path.transition_into(should_level_up));
        }

        let seg = &distraction.path.segments()[seg_index];

        transform.translation = seg.position(seg_t).extend(z);
    }
}

/// The angular velocity of distractions is reduced every second.
///
/// TODO: if rotating fast exit the stage
pub(crate) fn rotate(
    mut distractions: Query<
        (&mut AngularVelocity, &mut Transform),
        With<Distraction>,
    >,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut angular_vel, mut transform) in distractions.iter_mut() {
        let z = transform.translation.z;
        let angvel = **angular_vel;

        transform.rotate_z(-angvel * dt);

        *angular_vel = AngularVelocity::new(
            angvel - angvel * (dt / ROTATION_STOP_IN_SECS),
        );

        transform.translation.z = z;
    }
}

/// If weather is very close and does special, the distraction is destroyed.
///
/// Otherwise, if weather is still close and moves in line with the distraction
/// (velocities are aligned or ~ opposite), then the distraction angular
/// velocity is increased.
///
/// TODO: this needs calibration
pub(crate) fn react_to_weather(
    mut score: EventWriter<DistractionDestroyedEvent>,
    mut weather_actions: EventReader<weather::ActionEvent>,
    weather: Query<
        (&Velocity, &Transform),
        (With<Weather>, Without<Distraction>),
    >,
    mut distraction: Query<
        (Entity, &Distraction, &Transform, &mut AngularVelocity),
        Without<Weather>,
    >,
    mut commands: Commands,
) {
    let Some(action) = weather_actions.read().last() else {
        return;
    };

    let Ok((weather_vel, weather_transform)) = weather.get_single() else {
        return;
    };
    let weather_translation = weather_transform.translation.truncate();
    let weather_vel = **weather_vel;

    for (entity, distraction, transform, mut angvel) in distraction.iter_mut() {
        let translation = transform.translation.truncate();
        let distance_to_weather = translation.distance(weather_translation);

        // in case we in future change the consts
        debug_assert!(
            HITBOX_RADIUS > LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL
        );
        if distance_to_weather > HITBOX_RADIUS {
            continue;
        }

        match (action, distraction.level) {
            (weather::ActionEvent::FiredSpecial, Level::One)
                if distance_to_weather
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
            // this action doesn't affect distractions
            (weather::ActionEvent::StartLoadingSpecial { .. }, _) => continue,
            _ => {}
        };

        let (seg_index, seg_t) = distraction.path_segment();
        let seg = &distraction.path.segments()[seg_index];
        let vel = seg.velocity(seg_t);

        // if the two vectors are closely aligned, the we affect the distraction
        let a = vel.angle_between(weather_vel);
        if a.abs() > PERPENDICULARITY_THRESHOLD {
            continue;
        }

        // Different action will have differently proportional effect on the
        // angular velocity.
        let action_factor = match action {
            // we early returned above
            weather::ActionEvent::StartLoadingSpecial { .. } => unreachable!(),
            weather::ActionEvent::DashedAgainstVelocity { .. } => 1.0,
            weather::ActionEvent::Dipped => 2.0,
            // higher if bigger jumps
            weather::ActionEvent::Jumped { jumps_left } => {
                2.0 * (*jumps_left as f32 / weather::consts::MAX_JUMPS as f32)
            }
            weather::ActionEvent::FiredSpecial => 5.0,
        };

        // The kick to angvel is inversely proportional to distance between
        // weather and distraction.
        // This encourages the player to move the weather around the distraction
        // fast back and forth to increase the angular velocity.
        let distance_penalty = 1.0 - (distance_to_weather / HITBOX_RADIUS);

        let kick = DEFAULT_ANGVEL_KICK * action_factor * distance_penalty;
        trace!("Kick to angvel: {kick}");

        *angvel += AngularVelocity::new(kick);
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
                            vec2(100.0, 100.0), // TODO
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
                        texture: asset_server.load(
                            "textures/distractions/blackhole_flicker.png"
                                .to_string(),
                        ),
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

impl Distraction {
    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }

    fn laps(&self) -> usize {
        (self.current_path_since.elapsed_secs() / self.path.total_path_time())
            as usize
    }
}
