use std::f32::consts::PI;

use bevy::{render::view::RenderLayers, utils::Instant};
use common_visuals::camera::render_layer;
use itertools::Itertools;
use main_game_lib::common_ext::ColorExt;

use crate::{
    cameras::BackgroundLightScene, hoshi, path::LevelPath, prelude::*,
};

/// Climate casts light rays.
/// We achieve those light rays by orbiting occluders around the climate.
/// How strong are they?
const LIGHT_INTENSITY: f32 = 3.0;
/// How far do the rays reach?
const FALLOFF_LIGHT_SIZE: f32 = 400.0;
/// Determines how many rays are casted.
const OCCLUDER_COUNT: usize = 4;
/// Inversely proportional to the ray size.
const OCCLUDER_SIZE: f32 = 18.0;
/// Determines the ray slope.
const OCCLUDER_DISTANCE: f32 = 40.0;
/// Occluders are evenly distributed around the climate.
/// We calculate the distribution around for the `occluder[1]` (0th starts at
/// 0).
const INITIAL_ROTATION: f32 = 2.0 * PI / OCCLUDER_COUNT as f32;
const INITIAL_HALF_ROTATION: f32 = INITIAL_ROTATION / 2.0;
/// When the mode is [`ClimateLightMode::Hot`], we deduct this much from the
/// score.
const HOT_DEDUCTION: usize = 80;
/// How often do we deduct from the score when the mode is
/// [`ClimateLightMode::Hot`].
const HOT_DEDUCTION_INTERVAL: Duration = from_millis(5_000);
/// Burns Polpos, so should reflect that.
const LIGHT_COLOR_HOT: Color = Color::rgb(0.6, 0.3, 0.1);
/// Purply cold color.
const LIGHT_COLOR_COLD: Color = crate::background::COLOR;
/// When the mode is [`ClimateLightMode::Cold`], we deduct this much from the
/// score.
const COLD_DEDUCTION: usize = 100;
/// How often do we deduct from the score when the mode is
/// [`ClimateLightMode::Cold`].
const COLD_DEDUCTION_INTERVAL: Duration = from_millis(10_000);
/// How long does it take for the light to change color when changing mode.
const LIGHT_COLOR_TRANSITION: Duration = from_millis(2500);
/// The pushback force scales from 0 to this value based on the angle between
/// the nearest climate ray and the Polpo.
pub(crate) const HOT_PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS: f32 = 45.0;
/// The pushback force is different for hot and cold.
pub(crate) const COLD_PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS: f32 = 35.0;

#[derive(Component)]
pub(crate) struct Climate {
    path: LevelPath,
    current_path_since: Stopwatch,
    /// Timer for the rays of light.
    /// Allows us to pause the ray animation when the game is paused.
    rays_animation: Stopwatch,
    /// When was the mode changed and the mode itself.
    mode: (Instant, ClimateLightMode),
}
/// Evenly distributed around the climate, they shape the light into rays.
#[derive(Component)]
struct ClimateOccluder {
    /// Each occluder is characterized by its initial rotation.
    initial_rotation: f32,
}
#[derive(Default, Clone, Copy)]
pub(crate) enum ClimateLightMode {
    #[default]
    Hot,
    Cold,
}

/// Debug tool.
/// Point which is shown when being lit by the climate.
#[cfg(feature = "devtools")]
#[derive(Component)]
struct RayPoint;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalGameState::LoadingMeditation), spawn)
            .add_systems(
                Update,
                (
                    toggle_mode
                        .run_if(on_event::<hoshi::ActionEvent>())
                        .after(hoshi::loading_special),
                    smoothly_transition_light_color,
                    move_occluders,
                )
                    .run_if(in_state(GlobalGameState::InGameMeditation)),
            )
            .add_systems(
                FixedUpdate,
                follow_curve
                    .run_if(in_state(GlobalGameState::InGameMeditation)),
            )
            .add_systems(
                Update,
                // don't stop the color transition in menu, no need as it does
                // not affect the gameplay but complicates things as it uses
                // instant to time stuff
                smoothly_transition_light_color
                    .run_if(in_state(GlobalGameState::MeditationInMenu)),
            );

        #[cfg(feature = "devtools")]
        app.add_systems(
            Update,
            visualize_raypoints
                .run_if(in_state(GlobalGameState::InGameMeditation)),
        )
        .add_systems(
            OnExit(GlobalGameState::QuittingMeditation),
            despawn_raypoints,
        );
    }
}

fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let climate = Climate::new();
    let climate_translation = {
        let (seg_index, seg_t) = climate.path_segment();
        let seg = &climate.path.segments()[seg_index];

        seg.position(seg_t).extend(zindex::CLIMATE)
    };

    cmd.spawn((
        climate,
        BackgroundLightScene,
        AngularVelocity::default(),
        SpatialBundle {
            transform: Transform::from_translation(climate_translation),
            ..default()
        },
        OmniLightSource2D {
            intensity: LIGHT_INTENSITY,
            color: ClimateLightMode::default().color(),
            falloff: Vec3::new(FALLOFF_LIGHT_SIZE, FALLOFF_LIGHT_SIZE, 0.05),
            ..default()
        },
    ))
    .with_children(|commands| {
        commands.spawn((
            RenderLayers::layer(render_layer::OBJ),
            SpriteBundle {
                texture: asset_server.load(assets::CLIMATE_DEFAULT),
                ..default()
            },
        ));
    });

    for i in 0..OCCLUDER_COUNT {
        let initial_rotation = INITIAL_ROTATION * i as f32;

        cmd.spawn((
            ClimateOccluder { initial_rotation },
            BackgroundLightScene,
            SpatialBundle {
                transform: {
                    let mut t = Transform::from_translation(
                        climate_translation
                            + (vec2(
                                initial_rotation.sin(),
                                initial_rotation.cos(),
                            )
                            .extend(0.0)
                                * OCCLUDER_DISTANCE),
                    );
                    t.rotate_z(-initial_rotation);
                    t
                },
                ..default()
            },
            LightOccluder2D {
                h_size: Vec2::new(OCCLUDER_SIZE, OCCLUDER_SIZE),
            },
        ));
    }

    #[cfg(feature = "devtools")]
    for _ in 0..10000 {
        use rand::{thread_rng, Rng};

        // spawns random points across the screen that will be lit by the rays

        let x = thread_rng().gen_range(-320.0..320.0);
        let y = thread_rng().gen_range(-180.0..180.0);

        cmd.spawn(RayPoint).insert(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.0, 1.0, 0.0, 1.0),
                custom_size: Some(vec2(1.0, 1.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(x, y, 100.)),
            visibility: Visibility::Hidden,
            ..default()
        });
    }
}

#[cfg(feature = "devtools")]
fn despawn_raypoints(
    mut cmd: Commands,
    raypoints: Query<Entity, With<RayPoint>>,
) {
    for entity in raypoints.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

/// Changes the mode of the climate on hoshi's special.
/// See readme for the game to understand what this means.
/// In short: we change light color, how score is deducted and how strong is
/// the ray on the Polpos.
fn toggle_mode(
    mut action: EventReader<hoshi::ActionEvent>,

    mut climate: Query<&mut Climate>,
    mut score: Query<&mut crate::ui::Score>,
) {
    let just_started_loading = action
        .read()
        .any(|e| matches!(e, hoshi::ActionEvent::FiredSpecial));

    if !just_started_loading {
        return;
    }

    let mut climate = climate.single_mut();
    let mut score = score.single_mut();

    let new_mode = !climate.mode.1;
    climate.mode = (Instant::now(), new_mode);
    score.set_deduction(new_mode.deduction());
    score.set_deduction_interval(new_mode.deduction_interval());
}

fn smoothly_transition_light_color(
    mut climate: Query<(&Climate, &mut OmniLightSource2D)>,
) {
    let (climate, mut light) = climate.single_mut();

    // change color of the light based on climate.mode smoothly
    let (changed_at, mode) = climate.mode;
    let elapsed = changed_at.elapsed();

    if elapsed > LIGHT_COLOR_TRANSITION {
        light.color = mode.color();
        return;
    }

    let t =
        (elapsed.as_secs_f32() / LIGHT_COLOR_TRANSITION.as_secs_f32()).min(1.0);
    light.color = light.color.lerp(mode.color(), t);
}

/// Polpos have something similar, but with some extra logic to change their
/// path.
fn follow_curve(
    mut climate: Query<(&mut Climate, &mut Transform)>,
    time: Res<Time>,
) {
    let (mut climate, mut transform) = climate.single_mut();

    climate.rays_animation.tick(time.delta());
    climate.current_path_since.tick(time.delta());

    let z = transform.translation.z;
    let (seg_index, seg_t) = climate.path_segment();

    let seg = &climate.path.segments()[seg_index];

    transform.translation = seg.position(seg_t).extend(z);
}

/// Animates the occluders around the climate.
/// This results in rays of light being casted from the climate.
fn move_occluders(
    climate: Query<(&Climate, &Transform), Without<ClimateOccluder>>,
    mut occluders: Query<(&ClimateOccluder, &mut Transform), Without<Climate>>,
) {
    let (climate, climate_transform) = climate.single();

    for (ClimateOccluder { initial_rotation }, mut transform) in
        occluders.iter_mut()
    {
        let rotation_now =
            initial_rotation + climate.change_in_ray_angle_over_time();

        transform.translation = climate_transform.translation
            + (vec2(rotation_now.sin(), rotation_now.cos()).extend(0.0)
                * OCCLUDER_DISTANCE);
        transform.rotation = Quat::from_rotation_z(-rotation_now);
    }
}

impl Climate {
    pub(crate) fn mode(&self) -> ClimateLightMode {
        self.mode.1
    }

    /// In interval [0, 1], how strongly is target lit by the climate?
    /// This ignores any possible obstacle in the way and just computes angle
    /// between the closest ray and the target.
    pub(crate) fn ray_bath(&self, self_pos: Vec2, target_pos: Vec2) -> f32 {
        let angle_to_ray =
            self.angle_between_closest_ray_and_point(self_pos, target_pos);

        // a smooth monotonically decreasing function that starts at (0, 1)
        // and ends at (INITIAL_HALF_ROTATION, 0)
        //
        // (x - h)^i / h^i {x in [0, h]}

        let h = INITIAL_HALF_ROTATION;
        let to_the_power_of = 6;

        (angle_to_ray - h).powi(to_the_power_of)
        //--------------------------------------
        /         h.powi(to_the_power_of)
    }

    fn new() -> Self {
        Self {
            path: LevelPath::InfinitySign,
            current_path_since: Stopwatch::default(),
            rays_animation: Stopwatch::default(),
            mode: (Instant::now(), default()),
        }
    }

    fn angle_between_closest_ray_and_point(
        &self,
        climate_pos: Vec2,
        target: Vec2,
    ) -> f32 {
        angle_between_closest_ray_and_point(
            INITIAL_ROTATION,
            self.change_in_ray_angle_over_time(),
            climate_pos,
            target,
        )
    }

    #[inline]
    fn change_in_ray_angle_over_time(&self) -> f32 {
        (self.rays_animation.elapsed_secs() * 0.25) % INITIAL_ROTATION
    }

    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }
}

impl ClimateLightMode {
    pub(crate) fn push_back_force_fully_casted_in_climate_rays(self) -> f32 {
        match self {
            ClimateLightMode::Hot => {
                HOT_PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS
            }
            ClimateLightMode::Cold => {
                COLD_PUSH_BACK_FORCE_FULLY_CASTED_IN_CLIMATE_RAYS
            }
        }
    }

    pub(crate) fn deduction(self) -> usize {
        match self {
            ClimateLightMode::Hot => HOT_DEDUCTION,
            ClimateLightMode::Cold => COLD_DEDUCTION,
        }
    }

    pub(crate) fn deduction_interval(self) -> Duration {
        match self {
            ClimateLightMode::Hot => HOT_DEDUCTION_INTERVAL,
            ClimateLightMode::Cold => COLD_DEDUCTION_INTERVAL,
        }
    }

    fn color(self) -> Color {
        match self {
            ClimateLightMode::Hot => LIGHT_COLOR_HOT,
            ClimateLightMode::Cold => LIGHT_COLOR_COLD,
        }
    }
}

impl std::ops::Not for ClimateLightMode {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ClimateLightMode::Hot => ClimateLightMode::Cold,
            ClimateLightMode::Cold => ClimateLightMode::Hot,
        }
    }
}

/// Find the angle between the closest ray and the given point.
/// Rays are uniformly distributed around the circle with center at `climate`.
///
/// The `dt` tells us how much are the rotating rays
/// currently rotated.
#[inline]
fn angle_between_closest_ray_and_point(
    rotation: f32,
    dt: f32,
    climate: Vec2,
    target: Vec2,
) -> f32 {
    // origin at climate
    let normalized_target = target - climate;

    let (angle_to_ray, _) = (0..OCCLUDER_COUNT)
        .map(|i| {
            let initial_rotation = rotation * i as f32;
            let rotation_now = initial_rotation + dt;

            // occluder with in basis with origin at climate
            let Vec2 { x, y } = vec2(rotation_now.sin(), rotation_now.cos())
                * OCCLUDER_DISTANCE;

            let half_angle = rotation / 2.0;
            let half_rotated_occluder = vec2(
                x * half_angle.cos() - y * half_angle.sin(),
                x * half_angle.sin() + y * half_angle.cos(),
            );

            half_rotated_occluder.angle_between(normalized_target).abs()
        })
        .minmax()
        .into_option()
        .expect("at least one occluder");

    angle_to_ray
}

#[cfg(feature = "devtools")]
fn visualize_raypoints(
    climate: Query<(&Climate, &Transform)>,
    mut raypoints: Query<
        (&Transform, &mut Visibility, &mut Sprite),
        With<RayPoint>,
    >,
) {
    let (climate, climate_transform) = climate.single();

    for (transform, mut visibility, mut sprite) in raypoints.iter_mut() {
        let ray_bath = climate.ray_bath(
            climate_transform.translation.truncate(),
            transform.translation.truncate(),
        );

        *visibility = if ray_bath > f32::EPSILON {
            sprite.color.set_a(ray_bath);

            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::*;

    const FPS: f32 = 1000.0;
    const PLAY_FOR: Duration = Duration::from_secs(300);

    #[test]
    fn it_smoothly_finds_angle() {
        let climate_pos = vec2(0.0, 0.0);
        let target = vec2(1.0, 1.0);

        let mut climate = Climate::new();

        let first_angle =
            climate.angle_between_closest_ray_and_point(climate_pos, target);
        approximately_eq(PI / 3.0 - PI / 4.0, first_angle).unwrap();

        climate
            .rays_animation
            .tick(Duration::from_secs_f32(1.0 / FPS));

        // shouldn't be a big change
        let second_angle =
            climate.angle_between_closest_ray_and_point(climate_pos, target);
        approximately_eq(PI / 3.0 - PI / 4.0, second_angle).unwrap();

        let expected_step = (first_angle.abs() - second_angle.abs()).abs();
        approximately_eq(0.0005, expected_step).unwrap();

        let mut max: f32 = 0.0;
        let mut min = f32::MAX;
        let mut prev_angle = second_angle;
        while climate.rays_animation.elapsed() <= PLAY_FOR {
            climate
                .rays_animation
                .tick(Duration::from_secs_f32(1.0 / FPS));

            let new_angle = climate
                .angle_between_closest_ray_and_point(climate_pos, target);

            // diff should be small - ie. it should be a smooth transition
            approximately_eq(
                (prev_angle.abs() - new_angle.abs()).abs(),
                expected_step,
            )
            .unwrap();

            max = max.max(new_angle);
            min = min.min(new_angle);

            prev_angle = new_angle;
        }

        approximately_eq(0.0, min).unwrap();
        approximately_eq(PI / 3.0 / 2.0, max).unwrap();
    }

    #[test]
    fn it_smoothly_finds_while_moving_climate() {
        let mut climate_pos = vec2(0.0, 0.0);
        let target = vec2(1.0, 1.0);

        let mut climate = Climate::new();

        // this is tested above, here we are focused on moving the climate
        let first_angle =
            climate.angle_between_closest_ray_and_point(climate_pos, target);
        climate
            .rays_animation
            .tick(Duration::from_secs_f32(1.0 / FPS));
        let second_angle =
            climate.angle_between_closest_ray_and_point(climate_pos, target);
        let expected_step = (first_angle.abs() - second_angle.abs()).abs();

        let mut prev_angle = second_angle;
        while climate.rays_animation.elapsed() <= PLAY_FOR {
            climate
                .rays_animation
                .tick(Duration::from_secs_f32(1.0 / FPS));

            climate_pos += vec2(
                thread_rng().gen_range(-1.0..1.0),
                thread_rng().gen_range(-1.0..1.0),
            );

            let new_angle = climate
                .angle_between_closest_ray_and_point(climate_pos, target);

            // diff should be small - ie. it should be a smooth transition
            approximately_eq(
                (prev_angle.abs() - new_angle.abs()).abs(),
                expected_step,
            )
            .unwrap();

            prev_angle = new_angle;
        }
    }

    #[test]
    fn it_finds_angle_at_t_zero_and_climate_at_origin() {
        let climate_pos = vec2(0.0, 0.0);

        for k in 0..10 {
            println!("Test with k = {k}\n");
            test_with_k_rotations_around_circle(climate_pos, k);
        }
    }

    fn test_with_k_rotations_around_circle(climate_pos: Vec2, k: usize) {
        let offset = 2.0 * PI * k as f32;

        let target = vec2(1.0, 1.0); // 45°
        let rotation = PI / 2.0 + offset; // 90°
        approximately_eq(
            PI / 4.0, // 45° diff
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();
        // test scale free
        approximately_eq(
            PI / 4.0,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target * 2.0,
            ),
        )
        .unwrap();

        let target = vec2(1.0, 1.0); // 45°
        let rotation = PI / 4.0 + offset;
        approximately_eq(
            0.0,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();

        let target = vec2(-1.0, 1.0); // 135°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            PI / 4.0,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();

        let target = vec2(-1.0, -1.0); // 225°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            PI / 4.0,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();

        let target = vec2(0.0, -1.0); // 270°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            0.0,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();

        let target = vec2(-0.25, 1.0); // something slightly more than 90°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            0.24497,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();
        let target = vec2(0.25, 1.0); // something slightly less than 90°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            0.24497,
            angle_between_closest_ray_and_point(
                rotation,
                0.0,
                climate_pos,
                target,
            ),
        )
        .unwrap();
    }

    fn approximately_eq(expected: f32, got: f32) -> Result<(), String> {
        let tolerance = 0.01; // 1%
        let max_value = expected.abs().max(got.abs());
        let error = (expected - expected).abs();

        if error <= max_value * tolerance {
            Ok(())
        } else {
            Err(format!("expected: {expected}, got: {got}",))
        }
    }
}
