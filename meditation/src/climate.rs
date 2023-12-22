//! TODO: some nice animation possibly synced with music that restores jumps and
//! specials

use std::f32::consts::PI;

use bevy::time::Stopwatch;
use bevy_magic_light_2d::gi::types::{LightOccluder2D, OmniLightSource2D};

use crate::{path::LevelPath, prelude::*};

const OCCLUDER_COUNT: usize = 6;
const OCCLUDER_SIZE: f32 = 12.5;
const OCCLUDER_DISTANCE: f32 = 45.0;
/// evenly spaced holes in circle around climate
const OCCLUDER_SPACING: f32 = 2.0;
pub(crate) const INITIAL_OCCLUDER_ROTATION: f32 =
    2.0 * PI * OCCLUDER_SPACING / (OCCLUDER_COUNT as f32 * OCCLUDER_SPACING);

#[derive(Component)]
pub(crate) struct Climate {
    path: LevelPath,
    current_path_since: Stopwatch,
}

#[derive(Component)]
struct ClimateLight;

#[derive(Component)]
struct ClimateOccluder {
    initial_rotation: f32,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .register_type::<LightOccluder2D>()
            .add_systems(Update, (follow_curve, move_occluders));
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
            intensity: 1.5,                    // TODO
            color: Color::rgb_u8(137, 79, 24), // TODO
            // TODO: jitter the more distractions there are
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
}

/// Distractions have something similar, but with some extra logic to change
/// path.
pub(crate) fn follow_curve(
    game: Query<&Game, Without<Paused>>,
    mut climate: Query<(&mut Climate, &mut Transform)>,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

    let (mut climate, mut transform) = climate.single_mut();

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
    climate: Query<&Transform, (With<Climate>, Without<ClimateOccluder>)>,
    mut occluders: Query<(&ClimateOccluder, &mut Transform), Without<Climate>>,
    time: Res<Time>,
) {
    if game.is_empty() {
        return;
    }

    let climate = climate.single();

    for (ClimateOccluder { initial_rotation }, mut transform) in
        occluders.iter_mut()
    {
        let position =
            initial_rotation + Climate::change_in_ray_angle_over_time(&time);

        transform.translation = climate.translation
            + (vec2(position.sin(), position.cos()).extend(0.0)
                * OCCLUDER_DISTANCE);
        transform.rotation = Quat::from_rotation_z(-position);
    }
}

impl Climate {
    pub(crate) fn pause(&mut self) -> &mut Self {
        self.current_path_since.pause();
        self
    }

    pub(crate) fn resume(&mut self) -> &mut Self {
        self.current_path_since.unpause();
        self
    }

    /// Calculates the angle between the closest ray and the given position.
    /// Rays are casted from the climate.
    /// We can think of them as straight lines forming a star of sorts with
    /// the lines meeting at `own_pos`.
    /// The lines are evenly distributed around the circle (2PI)
    /// starting at half the [`INITIAL_OCCLUDER_ROTATION`].
    #[inline]
    pub(crate) fn angle_between_closest_ray_and_other(
        own_pos: Vec2,
        other: Vec2,
        time: &Time,
    ) -> f32 {
        // half way between the first two occluders
        let first_ray_rotation_at_t_0 = INITIAL_OCCLUDER_ROTATION / 2.0;

        let first_ray_rotation_now = first_ray_rotation_at_t_0
            + Self::change_in_ray_angle_over_time(time);
        angle_between_closest_ray(first_ray_rotation_now, own_pos, other)
    }

    #[inline]
    fn change_in_ray_angle_over_time(time: &Time) -> f32 {
        time.elapsed_seconds() / 2.0
    }

    fn new() -> Self {
        Self {
            path: LevelPath::InfinitySign,
            current_path_since: Stopwatch::default(),
        }
    }

    fn path_segment(&self) -> (usize, f32) {
        self.path.segment(&self.current_path_since.elapsed())
    }
}

#[inline]
fn angle_between_closest_ray(
    rotation: f32,
    own_pos: Vec2,
    target: Vec2,
) -> f32 {
    // we don't care about full rotations around the circle
    let rotation = rotation % (PI * 2.0);
    let half_rotation = rotation / 2.0;
    let first_ray = vec2(rotation.cos(), rotation.sin());

    // vector from (0, 0) in the direction of climate to target
    let diff = target - own_pos;

    // the very first ray might not be the closest one
    let angle_between_first_ray = diff.angle_between(first_ray);

    // next ray might not be the closest one, look behind!
    let angle_between_next_ray = angle_between_first_ray.abs() % rotation;

    // finds the closest ray by looking behind
    let angle_between_closest_ray =
        half_rotation - (half_rotation - angle_between_next_ray).abs();

    angle_between_closest_ray
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::*;

    #[test]
    fn it_smoothly_finds_angle() {
        const FPS: f32 = 1000.0;
        const PLAY_FOR: Duration = Duration::from_secs(10);

        let mut time = Time::default();
        let own_pos = vec2(0.0, 0.0);
        let other = vec2(1.0, 1.0);

        let first_angle =
            Climate::angle_between_closest_ray_and_other(own_pos, other, &time);
        approximately_eq(PI / 3.0 - PI / 4.0, first_angle).unwrap();

        time.advance_by(Duration::from_secs_f32(1.0 / FPS));

        // shouldn't be a big change
        let second_angle =
            Climate::angle_between_closest_ray_and_other(own_pos, other, &time);
        approximately_eq(PI / 3.0 - PI / 4.0, second_angle).unwrap();

        let expected_step = (first_angle.abs() - second_angle.abs()).abs();
        approximately_eq(0.0005, expected_step).unwrap();

        let mut prev_angle = first_angle;
        let mut max: f32 = 0.0;
        let mut min = f32::MAX;
        while time.elapsed() <= PLAY_FOR {
            time.advance_by(Duration::from_secs_f32(1.0 / FPS));

            let new_angle = Climate::angle_between_closest_ray_and_other(
                own_pos, other, &time,
            );

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
        const FPS: f32 = 1000.0;
        const PLAY_FOR: Duration = Duration::from_secs(300);

        let mut time = Time::default();
        let mut own_pos = vec2(0.0, 0.0);
        let other = vec2(1.0, 1.0);

        // this is tested above, here we are focused on moving the climate
        let first_angle =
            Climate::angle_between_closest_ray_and_other(own_pos, other, &time);
        time.advance_by(Duration::from_secs_f32(1.0 / FPS));
        let second_angle =
            Climate::angle_between_closest_ray_and_other(own_pos, other, &time);
        let expected_step = (first_angle.abs() - second_angle.abs()).abs();

        let mut prev_angle = first_angle;

        while time.elapsed() <= PLAY_FOR {
            time.advance_by(Duration::from_secs_f32(1.0 / FPS));
            own_pos += vec2(
                thread_rng().gen_range(-1.0..1.0),
                thread_rng().gen_range(-1.0..1.0),
            );

            let new_angle = Climate::angle_between_closest_ray_and_other(
                own_pos, other, &time,
            );

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
        let own_pos = vec2(0.0, 0.0);

        for k in 0..10 {
            println!("Test with k = {k}\n");
            test_with_k_rotations_around_circle(own_pos, k);
        }
    }

    fn test_with_k_rotations_around_circle(own_pos: Vec2, k: usize) {
        let offset = 2.0 * PI * k as f32;

        let other = vec2(1.0, 1.0); // 45°
        let rotation = PI / 2.0 + offset; // 90°
        approximately_eq(
            PI / 4.0, // 45° diff
            angle_between_closest_ray(rotation, own_pos, other),
        )
        .unwrap();
        // test scale free
        approximately_eq(
            PI / 4.0,
            angle_between_closest_ray(rotation, own_pos, other * 2.0),
        )
        .unwrap();

        let other = vec2(1.0, 1.0); // 45°
        let rotation = PI / 4.0 + offset;
        approximately_eq(
            0.0,
            angle_between_closest_ray(rotation, own_pos, other),
        )
        .unwrap();

        let other = vec2(-1.0, 1.0); // 135°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            PI / 4.0,
            angle_between_closest_ray(rotation, own_pos, other),
        )
        .unwrap();

        let other = vec2(-1.0, -1.0); // 225°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            PI / 4.0,
            angle_between_closest_ray(rotation, own_pos, other),
        )
        .unwrap();

        let other = vec2(0.0, -1.0); // 270°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            0.0,
            angle_between_closest_ray(rotation, own_pos, other),
        )
        .unwrap();

        let other = vec2(-0.25, 1.0); // something slightly more than 90°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            0.24497,
            angle_between_closest_ray(rotation, own_pos, other),
        )
        .unwrap();
        let other = vec2(0.25, 1.0); // something slightly less than 90°
        let rotation = PI / 2.0 + offset;
        approximately_eq(
            0.24497,
            angle_between_closest_ray(rotation, own_pos, other),
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
