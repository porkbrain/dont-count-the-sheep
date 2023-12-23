mod consts;
mod spawner;
mod videos;

use bevy::time::Stopwatch;
use bevy_magic_light_2d::gi::types::OmniLightSource2D;
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};

use crate::path::LevelPath;
use crate::{
    climate::Climate,
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
    weather::{self, Weather},
};
use consts::*;
use videos::Video;

#[derive(Component)]
pub(crate) struct Distraction {
    video: Video,
    current_path_since: Stopwatch,
    path: LevelPath,
    transition_into: Option<LevelPath>,
}

#[derive(Component)]
struct DistractionOccluder;

#[derive(Component)]
struct BlackHole(GridCoords, Stopwatch);

#[derive(Event)]
struct DistractionDestroyedEvent {
    /// Which video was playing on the distraction.
    video: Video,
    /// Where the distraction was when it was destroyed.
    at_translation: Vec2,
    /// Whether the distraction was destroyed by the weather special or by
    /// just accumulating cracks.
    by_special: bool,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DistractionDestroyedEvent>()
            .insert_resource(spawner::Spawner::new())
            .add_systems(
                Update,
                (
                    spawner::try_spawn_next,
                    // `after` so that the distraction does not for 1 frame
                    // appear in the middle
                    follow_curve.after(spawner::try_spawn_next),
                    react_to_environment,
                    react_to_weather_special
                        .after(weather::loading_special_system),
                    destroyed
                        .after(react_to_weather_special)
                        .after(react_to_environment),
                ),
            );
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

/// Climate has something similar, but without the level up logic.
fn follow_curve(
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

/// If weather is very close and does special, the distraction is destroyed.
fn react_to_weather_special(
    game: Query<&Game, Without<Paused>>,
    mut score: EventWriter<DistractionDestroyedEvent>,
    mut weather_actions: EventReader<weather::ActionEvent>,
    weather: Query<&Transform, (With<Weather>, Without<Distraction>)>,
    distractions: Query<(Entity, &Distraction, &Transform), Without<Weather>>,
    mut commands: Commands,
) {
    if game.is_empty() {
        return;
    }

    // it's possible that the game is paused the same frame as the event being
    // emitted, but that's so unlikely that we don't care
    let fired_special = weather_actions
        .read()
        .any(|a| matches!(a, weather::ActionEvent::FiredSpecial));
    if !fired_special {
        return;
    }

    let Ok(weather_transform) = weather.get_single() else {
        return;
    };

    let weather_translation = weather_transform.translation.truncate();

    for (entity, distraction, transform) in distractions.iter() {
        let translation = transform.translation.truncate();
        let distance_to_weather = translation.distance(weather_translation);

        if distance_to_weather <= WEATHER_SPECIAL_HITBOX_RADIUS {
            debug!("Distraction destroy by special event sent");
            score.send(DistractionDestroyedEvent {
                video: distraction.video,
                by_special: true,
                at_translation: translation,
            });
            commands.entity(entity).despawn_recursive();

            // ... go to next, can destroy multiple distractions per special
        }
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
fn react_to_environment(
    game: Query<&Game, Without<Paused>>,
    mut climate: Query<
        (&Climate, &Transform, &mut OmniLightSource2D),
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
    mut distractions: Query<
        (Entity, &Distraction, &Transform, &mut TextureAtlasSprite),
        (
            Without<Climate>,
            Without<Weather>,
            Without<DistractionOccluder>,
        ),
    >,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut score: EventWriter<DistractionDestroyedEvent>,
    mut commands: Commands,
) {
    if game.is_empty() {
        return;
    }

    let Ok(weather) = weather.get_single() else {
        return;
    };
    let Ok((climate, climate_transform, mut climate_light)) =
        climate.get_single_mut()
    else {
        return;
    };

    for (distraction_id, mut occluder_pos) in distraction_occluders.iter_mut() {
        let (distraction_entity, distraction, distraction_pos, mut sprite) =
            distractions
                .get_mut(distraction_id.get())
                .expect("Each occluder should have a distraction parent");

        //
        // 1.
        //

        // between [0; 1], increases as weather gets closer to distraction
        let weather_ray_bath = {
            let d = weather.translation.distance(distraction_pos.translation);

            1.0 / (d / HALF_OF_WEATHER_PUSH_BACK_FORCE_AT_DISTANCE + 1.0)
        };

        // between [0; 1], how much is the distraction being lit by the climate
        let climate_ray_bath = climate.ray_bath(
            climate_transform.translation.truncate(),
            distraction_pos.translation.truncate(),
        );

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

        let crack_chance = CRACK_CHANCE_PER_SECOND * time.delta_seconds();

        // TODO: balance
        let should_crack =
            push_back_force > 35.0 && rand::random::<f32>() < crack_chance;

        let is_on_last_crack = sprite.index == MAX_CRACKS - 1;
        if should_crack && !is_on_last_crack {
            sprite.index += 1;

            let is_on_second_to_last_crack = sprite.index == MAX_CRACKS - 2;

            if is_on_second_to_last_crack {
                let first_frame = 0;

                let static_entity = commands
                    .spawn((
                        Animation {
                            on_last_frame: AnimationEnd::Loop,
                            first: first_frame,
                            last: STATIC_ATLAS_FRAMES - 1,
                        },
                        AnimationTimer::new(
                            STATIC_ATLAS_FRAME_TIME,
                            TimerMode::Repeating,
                        ),
                    ))
                    .insert(SpriteSheetBundle {
                        texture_atlas: texture_atlases.add(
                            TextureAtlas::from_grid(
                                asset_server.load(
                                    "textures/distractions/static_atlas.png",
                                ),
                                vec2(
                                    DISTRACTION_SPRITE_SIZE,
                                    DISTRACTION_SPRITE_SIZE,
                                ),
                                STATIC_ATLAS_FRAMES,
                                1,
                                None,
                                None,
                            ),
                        ),
                        sprite: TextureAtlasSprite::new(first_frame),
                        transform: Transform::from_translation(
                            vec2(0.0, 0.0).extend(zindex::DISTRACTION_STATIC),
                        ),
                        ..default()
                    })
                    .id();

                commands.entity(distraction_entity).add_child(static_entity);
            }
        } else if should_crack && is_on_last_crack {
            //
            // 4.
            //

            debug!("Distraction destroy event sent");
            score.send(DistractionDestroyedEvent {
                video: distraction.video,
                by_special: false,
                at_translation: distraction_pos.translation.truncate(),
            });

            commands.entity(distraction_entity).despawn_recursive();
        }

        //
        // 5.
        //

        // On a line between climate and distraction, pushed back behind the
        // distraction by push_back_force.
        //
        // Unfortunately, the lighting engine does not use global transform to
        // calculate positions, so we need to add the distraction's translation
        occluder_pos.translation = distraction_pos.translation
            + (distraction_pos.translation - climate_transform.translation)
                .normalize()
                * push_back_force;
    }

    // increase jitter intensity as more distractions are spawned
    climate_light.jitter_intensity =
        (distractions.iter().len() as f32 / 5.0).min(1.0);
}

/// Either distraction is destroyed by the weather special or by accumulating
/// cracks.
///
/// TODO: bug sometimes black hole not removed
fn destroyed(
    game: Query<&Game>,
    mut score: Query<&mut crate::ui::Score>,
    mut spawner: ResMut<spawner::Spawner>,
    mut events: EventReader<DistractionDestroyedEvent>,
    mut gravity: EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if game.is_empty() {
        return;
    }

    if events.is_empty() {
        return;
    }

    let mut score = score.single_mut();

    for DistractionDestroyedEvent {
        video,
        at_translation,
        by_special,
    } in events.read()
    {
        debug!("Received distraction destroyed event (special: {by_special})");

        // the further away the distraction is, the more points it's worth
        *score += at_translation.length() as usize;
        // notify the spawner that the distraction is gone
        spawner.despawn(*video);

        if !by_special {
            // TODO: some animation of the distraction falling apart

            continue;
        }

        trace!("Spawning black hole");
        spawn_black_hole(
            &mut commands,
            &asset_server,
            &mut texture_atlases,
            &mut gravity,
            *at_translation,
        );
    }
}

/// Includes effects of gravity on the poissons equation.
fn spawn_black_hole(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    gravity: &mut EventWriter<PoissonsEquationUpdateEvent<Gravity>>,
    at_translation: Vec2,
) {
    let gravity_grid_coords = PoissonsEquationUpdateEvent::send(
        gravity,
        BLACK_HOLE_GRAVITY,
        ChangeOfBasis::new(at_translation),
    );

    let on_last_frame = AnimationEnd::Custom(Box::new(
        move |entity,
              _animation,
              _timer,
              _atlas,
              _visibility,
              commands,
              _time| {
            // delete the black hole
            commands.entity(entity).despawn_recursive();

            // remove gravity influence
            commands.add(move |world: &mut World| {
                world.send_event(PoissonsEquationUpdateEvent::<Gravity>::new(
                    -BLACK_HOLE_GRAVITY,
                    ChangeOfBasis::new(at_translation),
                ))
            });
        },
    ));

    commands
        .spawn(BlackHole(gravity_grid_coords, Stopwatch::new()))
        .insert((
            Animation {
                first: 0,
                last: BLACK_HOLE_ATLAS_FRAMES - 1,
                on_last_frame,
            },
            BeginAnimationAtRandom {
                chance_per_second: BLACK_HOLE_DESPAWN_CHANCE_PER_SECOND,
                frame_time: BLACK_HOLE_FRAME_TIME,
            },
        ))
        .insert(SpriteSheetBundle {
            texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load("textures/distractions/blackhole_atlas.png"),
                vec2(BLACK_HOLE_SPRITE_SIZE, BLACK_HOLE_SPRITE_SIZE),
                BLACK_HOLE_ATLAS_FRAMES,
                1,
                None,
                None,
            )),
            transform: Transform::from_translation(
                at_translation.extend(zindex::BLACK_HOLE),
            ),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Flicker::new(
                    BLACK_HOLE_FLICKER_CHANCE_PER_SECOND,
                    BLACK_HOLE_FLICKER_DURATION,
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

impl Distraction {
    pub(crate) fn pause(&mut self) {
        self.current_path_since.pause();
    }

    pub(crate) fn resume(&mut self) {
        self.current_path_since.unpause();
    }

    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }

    fn laps(&self) -> usize {
        (self.current_path_since.elapsed_secs() / self.path.total_path_time())
            as usize
    }

    fn new(video: Video) -> Self {
        Self {
            video,
            path: LevelPath::random_intro(),
            current_path_since: Stopwatch::new(),
            transition_into: None,
        }
    }
}
