//! TODO: some nice animation possibly synced with music that restores jumps and
//! specials

use std::f32::consts::PI;

use bevy::time::Stopwatch;
use bevy_magic_light_2d::gi::types::{LightOccluder2D, OmniLightSource2D};
use itertools::Itertools;

use crate::{path::LevelPath, prelude::*};

/// Climate casts light rays.
/// We achieve those light rays by orbiting occluders around the climate.
/// How strong are they?
const LIGHT_INTENSITY: f32 = 3.0;
/// TODO: Something warm but spacy?
const LIGHT_COLOR: Color = Color::rgb(0.6, 0.3, 0.1);
/// Determines how many rays are casted.
const OCCLUDER_COUNT: usize = 5;
/// Determines the ray size.
const OCCLUDER_SIZE: f32 = 14.5;
/// Determines the ray slope.
const OCCLUDER_DISTANCE: f32 = 45.0;
/// Occluders are evenly distributed around the climate.
/// We calculate the distribution around for the occluder[1] (0th starts at 0).
const INITIAL_OCCLUDER_ROTATION: f32 = 2.0 * PI / OCCLUDER_COUNT as f32;
/// How far can the ray reach?
///
/// Some good values based on occulder count:
/// * 6: PI / 12.0
/// * 5: PI / 8.0
const MAX_RAY_ANGULAR_DISTANCE: f32 = PI / 8.0;

#[derive(Component)]
pub(crate) struct Climate {
    path: LevelPath,
    current_path_since: Stopwatch,
    /// Timer for the rays of light.
    /// Allows us to pause the ray animation when the game is paused.
    rays_timer: Stopwatch,
}
/// Source of light at the center of the climate.
#[derive(Component)]
struct ClimateLight;
/// Evenly distributed around the climate, they shape the light into rays.
#[derive(Component)]
struct ClimateOccluder {
    initial_rotation: f32,
}

/// Point which is shown when being lit by the climate.
#[cfg(feature = "dev")]
#[derive(Component)]
struct RayPoint;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .register_type::<LightOccluder2D>()
            .add_systems(Update, (follow_curve, move_occluders));

        #[cfg(feature = "dev")]
        app.add_systems(Update, visualize_raypoints);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Climate::new(),
            AngularVelocity::default(),
            SpatialBundle {
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    zindex::CLIMATE,
                )),
                ..default()
            },
        ))
        .insert(OmniLightSource2D {
            intensity: LIGHT_INTENSITY,
            color: LIGHT_COLOR,
            falloff: Vec3::new(50.0, 50.0, 0.05),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn(SpriteBundle {
                texture: asset_server.load("textures/climate/default.png"),
                ..default()
            });
        });

    for i in 0..OCCLUDER_COUNT {
        let initial_rotation = INITIAL_OCCLUDER_ROTATION * i as f32;

        commands.spawn((
            ClimateOccluder { initial_rotation },
            SpatialBundle {
                transform: {
                    let mut t = Transform::default();
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

    #[cfg(feature = "dev")]
    for _ in 0..10000 {
        // spawns random points across the screen that will be lit by the rays

        use rand::{thread_rng, Rng};

        let x = thread_rng().gen_range(-320.0..320.0);
        let y = thread_rng().gen_range(-180.0..180.0);

        commands.spawn(RayPoint).insert(SpriteBundle {
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

/// Distractions have something similar, but with some extra logic to change
/// path.
fn follow_curve(
    game: Query<&Game, Without<Paused>>,
    mut climate: Query<(&mut Climate, &mut Transform)>,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

    let (mut climate, mut transform) = climate.single_mut();

    climate.rays_timer.tick(time.delta());
    climate.current_path_since.tick(time.delta());

    let z = transform.translation.z;
    let (seg_index, seg_t) = climate.path_segment();

    let seg = &climate.path.segments()[seg_index];

    transform.translation = seg.position(seg_t).extend(z);
}

/// Animates the occluders around the climate.
/// This results in rays of light being casted from the climate.
fn move_occluders(
    game: Query<&Game, Without<Paused>>,
    climate: Query<(&Climate, &Transform), Without<ClimateOccluder>>,
    mut occluders: Query<(&ClimateOccluder, &mut Transform), Without<Climate>>,
) {
    if game.is_empty() {
        return;
    }

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
    pub(crate) fn pause(&mut self) -> &mut Self {
        self.current_path_since.pause();
        self.rays_timer.pause();
        self
    }

    pub(crate) fn resume(&mut self) -> &mut Self {
        self.current_path_since.unpause();
        self.rays_timer.unpause();
        self
    }

    /// In interval [0, 1], how strongly is target lit by the climate?
    /// This ignores any possible obstacle in the way and just computes angle
    /// between the closest ray and the target.
    pub(crate) fn ray_bath(&self, climate_pos: Vec2, target: Vec2) -> f32 {
        let angle_to_ray =
            self.angle_between_closest_ray_and_point(climate_pos, target);

        // a smooth monotonically decreasing function (-x^2)
        -MAX_RAY_ANGULAR_DISTANCE.powi(-2)
            * angle_to_ray.clamp(0.0, MAX_RAY_ANGULAR_DISTANCE).powi(2)
            + 1.0
    }

    fn angle_between_closest_ray_and_point(
        &self,
        climate_pos: Vec2,
        target: Vec2,
    ) -> f32 {
        angle_between_closest_ray_and_point(
            INITIAL_OCCLUDER_ROTATION,
            self.change_in_ray_angle_over_time(),
            climate_pos,
            target,
        )
    }

    #[inline]
    fn change_in_ray_angle_over_time(&self) -> f32 {
        (self.rays_timer.elapsed_secs() * 0.25) % INITIAL_OCCLUDER_ROTATION
    }

    fn new() -> Self {
        Self {
            path: LevelPath::InfinitySign,
            current_path_since: Stopwatch::default(),
            rays_timer: Stopwatch::default(),
        }
    }

    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
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

#[cfg(feature = "dev")]
fn visualize_raypoints(
    game: Query<&Game, Without<Paused>>,
    climate: Query<(&Climate, &Transform)>,
    mut raypoints: Query<
        (&Transform, &mut Visibility, &mut Sprite),
        With<RayPoint>,
    >,
) {
    if game.is_empty() {
        return;
    }

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

        climate.rays_timer.tick(Duration::from_secs_f32(1.0 / FPS));

        // shouldn't be a big change
        let second_angle =
            climate.angle_between_closest_ray_and_point(climate_pos, target);
        approximately_eq(PI / 3.0 - PI / 4.0, second_angle).unwrap();

        let expected_step = (first_angle.abs() - second_angle.abs()).abs();
        approximately_eq(0.0005, expected_step).unwrap();

        let mut max: f32 = 0.0;
        let mut min = f32::MAX;
        let mut prev_angle = second_angle;
        while climate.rays_timer.elapsed() <= PLAY_FOR {
            climate.rays_timer.tick(Duration::from_secs_f32(1.0 / FPS));

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
        climate.rays_timer.tick(Duration::from_secs_f32(1.0 / FPS));
        let second_angle =
            climate.angle_between_closest_ray_and_point(climate_pos, target);
        let expected_step = (first_angle.abs() - second_angle.abs()).abs();

        let mut prev_angle = second_angle;
        while climate.rays_timer.elapsed() <= PLAY_FOR {
            climate.rays_timer.tick(Duration::from_secs_f32(1.0 / FPS));

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
