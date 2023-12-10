use std::f32::consts::PI;

use bevy::{
    math::cubic_splines::CubicCurve, sprite::MaterialMesh2dBundle,
    time::Stopwatch,
};
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};

use crate::{
    gravity::{ChangeOfBasis, Gravity},
    prelude::*,
    weather::{self, Weather},
};

const BLACKHOLE_FLICKER_CHANCE_PER_SECOND: f32 = 0.5;
const BLACKHOLE_FLICKER_DURATION: Duration = Duration::from_millis(100);
const LVL1_MAX_DIST_FOR_INSTA_DESTRUCT_ON_SPECIAL: f32 = 35.0;
const DISTRACTION_SIZE: f32 = 50.0;
const DEFAULT_ANGVEL_KICK: f32 = 5.0;
/// If the dot product of the velocity of the distraction and the velocity of
/// the weather is less than this threshold, then the distraction is affected
/// by the weather.
/// The rotation of the distraction is increased by the action factor.
const PERPENDICULARITY_THRESHOLD: f32 = PI / 8.0;
/// Weather must be within this distance of the distraction to affect it.
const HITBOX_RADIUS: f32 = DISTRACTION_SIZE * 1.25;
/// Every second, the angular velocity of distractions is reduced by this
/// percent amount.
const ROTATION_SLOWDOWN_PERCENT_PER_SECOND: f32 = 0.98;

#[derive(Component)]
pub(crate) struct Distraction {
    level: Level,
    curve: CubicCurve<Vec2>,
}

impl Distraction {
    pub(crate) fn new(curve: CubicCurve<Vec2>) -> Self {
        Self {
            level: Level::One,
            curve,
        }
    }
}

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    let factor = 4.0;

    // t = top
    // b = bottom
    // r = right
    // l = ?

    let l_control2 = vec2(-55.0, 23.0) * factor;
    let t_l = vec2(-64.0, 36.0) * factor;
    let t_control1 = vec2(-24.0, 25.0) * factor;

    let t_control2 = vec2(24.0, 42.0) * factor;
    let t_r = vec2(64.0, 36.0) * factor;
    let r_spiral_in_control1 = vec2(70.0, 16.0) * factor;

    let r_spiral_in_control2 = vec2(54.0, 6.0) * factor;
    let r_spiral_in = vec2(45.0, 12.0) * factor;
    let r_spiral_out_control1 = vec2(56.0, 24.0) * factor;

    let r_spiral_out_control2 = vec2(68.0, 0.0) * factor;
    let r_spiral_out = vec2(64.0, -12.0) * factor;
    let r_control1 = vec2(54.0, -20.0) * factor;

    let r_control2 = vec2(72.0, -28.0) * factor;
    let b_r = vec2(64.0, -36.0) * factor;
    let b_control1 = vec2(24.0, -40.0) * factor;

    let b_control2 = vec2(-36.0, -40.0) * factor;
    let b_l = vec2(-64.0, -36.0) * factor;
    let l_control1 = vec2(-68.0, -20.0) * factor;

    #[cfg(feature = "dev")]
    {
        // draw visualization for the curve

        for (pos, color) in [
            (l_control2, Color::SILVER),
            (t_l, Color::BLACK),
            (t_control1, Color::GREEN),
            (t_control2, Color::BLUE),
            (t_r, Color::BLACK),
            (r_spiral_in_control1, Color::AQUAMARINE),
            (r_spiral_in_control2, Color::GREEN),
            (r_spiral_in, Color::DARK_GREEN),
            (r_spiral_out_control1, Color::YELLOW),
            (r_spiral_out_control2, Color::ORANGE),
            (r_spiral_out, Color::RED),
            (r_control1, Color::SILVER),
            (r_control2, Color::GOLD),
            (b_r, Color::BLACK),
            (b_control1, Color::GREEN),
            (b_control2, Color::BLUE),
            (b_l, Color::BLACK),
            (l_control1, Color::GOLD),
        ] {
            // draw a small circle

            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(2.).into()).into(),
                material: materials.add(ColorMaterial::from(color)),
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            });
        }
    }

    let parent = commands
        .spawn((
            Distraction::new(
                CubicBezier::new(vec![
                    [t_l, t_control1, t_control2, t_r],
                    [
                        t_r,
                        r_spiral_in_control1,
                        r_spiral_in_control2,
                        r_spiral_in,
                    ],
                    [
                        r_spiral_in,
                        r_spiral_out_control1,
                        r_spiral_out_control2,
                        r_spiral_out,
                    ],
                    [r_spiral_out, r_control1, r_control2, b_r],
                    [b_r, b_control1, b_control2, b_l],
                    [b_l, l_control1, l_control2, t_l],
                ])
                .to_curve(),
            ),
            AngularVelocity::default(),
            SpriteSheetBundle {
                texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                    asset_server.load("textures/distractions/frame.png"),
                    vec2(50.0, 50.0),
                    5,
                    1,
                    None,
                    None,
                )),
                sprite: TextureAtlasSprite {
                    index: 1,
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    0.0, 0.0, 0.0,
                )),
                ..default()
            },
        ))
        .id();

    let child = commands
        .spawn((bevy_webp_anim::WebpBundle {
            animation: asset_server.load("textures/distractions/videos/1.webp"),
            frame_rate: bevy_webp_anim::FrameRate::new(2),
            sprite: Sprite { ..default() },
            ..default()
        },))
        .id();

    commands.entity(parent).add_child(child);
}

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
    time: Res<Time>,
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

        let (seg_index, seg_t) = distraction.segment(&time);
        let seg = &distraction.curve.segments()[seg_index];
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

//TODO: assert len
const SEGMENT_TIMING: [f32; 6] = {
    let f = 7.0;
    [1.0 * f, 1.4 * f, 1.7 * f, 2.0 * f, 3.0 * f, 3.5 * f]
};
const TOTAL_PATH_TIME: f32 = SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

/// Distractions follow an infinite loop curve in shape of the infinity symbol.
pub(crate) fn follow_curve(
    mut distraction: Query<(&Distraction, &mut Transform)>,
    time: Res<Time>,
) {
    for (distraction, mut transform) in distraction.iter_mut() {
        let z = transform.translation.z;

        let (seg_index, seg_t) = distraction.segment(&time);
        let seg = &distraction.curve.segments()[seg_index];

        transform.translation = seg.position(seg_t).extend(z);
    }
}

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
            angvel - angvel * ROTATION_SLOWDOWN_PERCENT_PER_SECOND * dt,
        );

        transform.translation.z = z;
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
    /// Returns the current segment and how much of it has been traversed.
    fn segment(&self, time: &Time) -> (usize, f32) {
        // total path time, as path repeats once all segments have been
        // traversed
        let total_t = time.elapsed_seconds() % TOTAL_PATH_TIME;

        // now calculate how much of the current segment has been traversed by
        // 1. finding the current segment
        // 2. finding finding how much is left
        // 3. finding the length of the current segment
        // 4. dividing 2. by 3. to get the percentage of the segment that has
        //    been
        // traversed

        // 1.
        let (seg_index, seg_ends_at) = SEGMENT_TIMING
            .iter()
            .enumerate()
            .find(|(_, seg_t)| total_t < **seg_t)
            .map(|(i, seg_t)| (i, *seg_t))
            .unwrap_or((
                SEGMENT_TIMING.len() - 1,
                SEGMENT_TIMING[SEGMENT_TIMING.len() - 1],
            ));
        // 2.
        let seg_remaining = seg_ends_at - total_t;
        // 3.
        let seg_length = if seg_index == 0 {
            SEGMENT_TIMING[0]
        } else {
            SEGMENT_TIMING[seg_index] - SEGMENT_TIMING[seg_index - 1]
        };
        // 4.
        let seg_t = 1.0 - (seg_remaining / seg_length);

        (seg_index, seg_t)
    }
}
