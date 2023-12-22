use bevy::time::Stopwatch;
// use bevy_magic_light_2d::gi::types::LightOccluder2D;
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};
use rand::thread_rng;

use crate::{
    climate::Climate,
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
    weather::Weather,
};

use crate::path::LevelPath;

const BLACKHOLE_FLICKER_CHANCE_PER_SECOND: f32 = 0.5;
const BLACKHOLE_FLICKER_DURATION: Duration = Duration::from_millis(100);
// const LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL: f32 = 35.0;

// const DISTRACTION_SPRITE_SIZE: f32 = 100.0;
/// There's some empty space around the sprite.
const DISTRACTION_PERCEIVED_SIZE: f32 = 50.0;
// /// The default angular velocity kick applied to distractions when the
// weather /// interacts with them.
// /// Scaled by the action factor (some actions increase it.)
// const DEFAULT_ANGVEL_KICK: f32 = PI;
// /// If the dot product of the velocity of the distraction and the velocity of
// /// the weather is less than this threshold, then the distraction is affected
// /// by the weather.
// /// The rotation of the distraction is increased by the action factor.
// const PERPENDICULARITY_THRESHOLD: f32 = PI / 8.0;
// /// Weather must be within this distance of the distraction to affect it.
// const HITBOX_RADIUS: f32 = DISTRACTION_PERCEIVED_SIZE * 1.25;
// /// Every second, the angular velocity of distractions is reduced.
// /// How many seconds does it take to stop rotating?
// const ROTATION_STOP_IN_SECS: f32 = 15.0;

#[derive(Component)]
pub(crate) struct Distraction {
    current_path_since: Stopwatch,
    path: LevelPath,
    transition_into: Option<LevelPath>,
}

#[derive(Component)]
pub(crate) struct DistractionOccluder;

impl Distraction {
    pub(crate) fn new() -> Self {
        Self {
            path: LevelPath::random_intro(),
            current_path_since: Stopwatch::new(),
            transition_into: None,
        }
    }

    pub(crate) fn pause(&mut self) {
        self.current_path_since.pause();
    }

    pub(crate) fn resume(&mut self) {
        self.current_path_since.unpause();
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
    pub(crate) at_translation: Vec2,
}

pub(crate) fn spawn(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((
            Distraction::new(),
            AngularVelocity::default(),
            SpatialBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                DistractionOccluder,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(
                            DISTRACTION_PERCEIVED_SIZE,
                            DISTRACTION_PERCEIVED_SIZE,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        0., 0., 100.,
                    )),
                    ..default()
                },
                // SpatialBundle {
                //     transform: Transform::from_translation(Vec3::new(
                //         0.0, 0.0, 100.0, // TODO
                //     )),
                //     ..default()
                // },
                // LightOccluder2D {
                //     h_size: Vec2::new(
                //         DISTRACTION_PERCEIVED_SIZE,
                //         DISTRACTION_PERCEIVED_SIZE,
                //     ),
                // },
            ));

            parent.spawn(SpriteBundle {
                texture: asset_server.load("textures/distractions/frame.png"),
                // z is higher than the the video
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

/// Climate has something similar, but without the level up logic.
pub(crate) fn follow_curve(
    game: Query<&Game, Without<Paused>>,
    mut distraction: Query<(&mut Distraction, &mut Transform)>,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

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
            // let should_level_up = rand::random::<f32>() < 0.8; // TODO
            let should_level_up = true;
            distraction.transition_into =
                Some(distraction.path.transition_into(should_level_up));
        }

        let seg = &distraction.path.segments()[seg_index];

        transform.translation = seg.position(seg_t).extend(z);
    }
}

// /// If weather is very close and does special, the distraction is destroyed.
// ///
// /// Otherwise, if weather is still close and moves in line with the
// distraction /// (velocities are aligned or ~ opposite), then the distraction
// angular /// velocity is increased.
// ///
// /// TODO: this needs calibration
// /// TODO; delete rotation, merge with the destroy system
// pub(crate) fn react_to_weather(
//     game: Query<&Game, Without<Paused>>,
//     mut score: EventWriter<DistractionDestroyedEvent>,
//     mut weather_actions: EventReader<weather::ActionEvent>,
//     weather: Query<
//         (&Velocity, &Transform),
//         (With<Weather>, Without<Distraction>),
//     >,
//     mut distraction: Query<
//         (Entity, &Distraction, &Transform, &mut AngularVelocity),
//         Without<Weather>,
//     >,
//     mut commands: Commands,
// ) {
//     if game.is_empty() {
//         return;
//     }

//     let Some(action) = weather_actions.read().last() else {
//         return;
//     };

//     let Ok((weather_vel, weather_transform)) = weather.get_single() else {
//         return;
//     };
//     let weather_translation = weather_transform.translation.truncate();

//     for (entity, distraction, transform, mut angvel) in
// distraction.iter_mut() {         let translation =
// transform.translation.truncate();         let distance_to_weather =
// translation.distance(weather_translation);

//         // in case we in future change the consts
//         debug_assert!(
//             HITBOX_RADIUS > LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL
//         );
//         if distance_to_weather > HITBOX_RADIUS {
//             continue;
//         }

//         match (action, distraction.level) {
//             (weather::ActionEvent::FiredSpecial, Level::One)
//                 if distance_to_weather
//                     < LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL =>
//             {
//                 debug!("Distraction destroy event sent");
//                 score.send(DistractionDestroyedEvent {
//                     level: Level::One,
//                     at_translation: translation,
//                 });
//                 commands.entity(entity).despawn_recursive();

//                 continue;
//             }
//             // this action doesn't affect distractions
//             (weather::ActionEvent::StartLoadingSpecial { .. }, _) =>
// continue,             _ => {}
//         };
//     }
// }

#[derive(Component)]
pub(crate) struct BlackHole(GridCoords, Stopwatch);

pub(crate) fn destroyed(
    game: Query<&Game>,
    mut events: EventReader<DistractionDestroyedEvent>,
    mut score: Query<&mut crate::ui::Score>,
    mut gravity: EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if game.is_empty() {
        return;
    }

    let mut score = score.single_mut();

    for DistractionDestroyedEvent { at_translation } in events.read() {
        *score += 1;

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

/// For each distraction:
/// 1. Check whether light is being cast on it (depends on rays from climate and
///    weather proximity)
/// 2. If it is, increase push back on distraction's occluder otherwise decrease
///    it
/// 3. If light is being cast on the distraction by climate, roll a dice to
///    crack it which adds a crack sprite to the distraction
/// 4. Remember number of cracks and if more than limit, destroy the distraction
/// 5. Find the line between climate and the distraction and place the occluder
///    on that line. Distance from center being the distraction's distance plus
///    the push back.
pub(crate) fn react_to_environment(
    game: Query<&Game, Without<Paused>>,
    climate: Query<
        (&Climate, &Transform),
        (
            Without<Weather>,
            Without<Distraction>,
            Without<DistractionOccluder>,
        ),
    >,
    weather: Query<
        &Transform,
        (
            Without<Climate>,
            With<Weather>,
            Without<Distraction>,
            Without<DistractionOccluder>,
        ),
    >,
    mut distraction_occluders: Query<
        (&Parent, &mut Transform),
        (
            Without<Climate>,
            Without<Weather>,
            Without<Distraction>,
            With<DistractionOccluder>,
        ),
    >,
    distractions: Query<
        &Transform,
        (
            Without<Climate>,
            Without<Weather>,
            With<Distraction>,
            Without<DistractionOccluder>,
        ),
    >,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

    let Ok(weather) = weather.get_single() else {
        return;
    };
    let Ok((climate, climate_transform)) = climate.get_single() else {
        return;
    };

    for (distraction_id, mut occluder_pos) in distraction_occluders.iter_mut() {
        let distraction = distractions
            .get(distraction_id.get())
            .expect("Each occluder should have a distraction parent");

        //
        // 1.
        //

        /// By default, occluder is pushed towards the climate.
        const PUSH_BACK_FORCE_AT_REST: f32 = -25.0;
        const PUSH_BACK_FORCE_WEATHER_DISTANCE: f32 = 50.0;
        const PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS: f32 = 50.0;
        /// At this distance, the occulder is pushed back by half of
        /// [`PUSH_BACK_FORCE_WEATHER_DISTANCE`].
        const HALF_OF_WEATHER_PUSH_BACK_FORCE_AT_DISTANCE: f32 = 100.0;

        // between [0; 1], increases as weather gets closer to distraction
        let weather_ray_bath = {
            let d = weather.translation.distance(distraction.translation);

            1.0 / (d / HALF_OF_WEATHER_PUSH_BACK_FORCE_AT_DISTANCE + 1.0)
        };

        // between [0; 1], how much is the distraction being lit by the climate
        let angle_to_ray = climate.angle_between_closest_ray_and_point(
            climate_transform.translation.truncate(),
            distraction.translation.truncate(),
        );
        // let climate_ray_bath = 1.0 - (PI / 12.0 - angle_to_ray.min(PI /
        // 12.0));
        println!("angle_to_ray {angle_to_ray}");
        println!("climate {}", climate_transform.translation.truncate());
        println!("distraction {}", distraction.translation.truncate());
        let climate_ray_bath = 1.0 - angle_to_ray.clamp(0.0, 1.0);
        println!("climate_ray_bath {climate_ray_bath}\n");

        //
        // 2.
        //

        // positive if pushed away from climate
        let push_back_force = PUSH_BACK_FORCE_AT_REST
            + weather_ray_bath * PUSH_BACK_FORCE_WEATHER_DISTANCE
            + climate_ray_bath * PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS;

        //
        // 3.
        //

        // TODO

        //
        // 4.
        //

        // TODO

        //
        // 5.
        //

        // On a line between climate and distraction, pushed back behind the
        // distraction by push_back_force.
        //
        // Unfortunately, the lighting engine does not use global transform to
        // calculate positions, so we need to add the distraction's translation
        occluder_pos.translation = /*distraction.translation
            +*/ (distraction.translation - climate_transform.translation).normalize()
                * push_back_force;
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
