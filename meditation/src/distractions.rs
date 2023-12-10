use bevy::{
    math::cubic_splines::CubicCurve, sprite::MaterialMesh2dBundle,
    time::Stopwatch,
};
use common_physics::{GridCoords, PoissonsEquationUpdateEvent};
use rand::{thread_rng, Rng};

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
const HITBOX_SIZE: Vec2 = vec2(HITBOX_WIDTH, HITBOX_HEIGHT);
const HITBOX_DISTANCE_TO_DISTRACTION: f32 = 0.0;
const DEFAULT_KICK: f32 = 5.0;

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
            + vec2(
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
                + vec2(
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
            + vec2(
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
                + vec2(
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
    let f = 5.0;
    [1.0 * f, 1.4 * f, 1.7 * f, 2.0 * f, 3.0 * f, 3.5 * f]
};
const TOTAL_PATH_TIME: f32 = SEGMENT_TIMING[SEGMENT_TIMING.len() - 1];

/// Distractions follow an infinite loop curve in shape of the infinity symbol.
pub(crate) fn follow_curve(
    mut distraction: Query<(&mut Distraction, &mut Transform)>,
    time: Res<Time>,
) {
    // total path time, as path repeats once all segments have been traversed
    let total_t = time.elapsed_seconds() % TOTAL_PATH_TIME;

    // now calculate how much of the current segment has been traversed by
    // 1. finding the current segment
    // 2. finding finding how much is left
    // 3. finding the length of the current segment
    // 4. dividing 2. by 3. to get the percentage of the segment that has been
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

    for (mut distraction, mut transform) in distraction.iter_mut() {
        let z = transform.translation.z;

        transform.translation = distraction.curve.segments()[seg_index]
            .position(seg_t)
            .extend(z);
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
