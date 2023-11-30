//! Illustrates bloom post-processing in 2d.

mod consts;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::Stopwatch};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            level: bevy::log::Level::WARN,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (control_weather, apply_acceleration, apply_velocity).chain(),
        )
        .run();
}

#[derive(Component, Default, Deref, DerefMut)]
struct Acceleration(Vec2);
#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);
#[derive(Component)]
struct Weather;

#[derive(Bundle, Default)]
struct BodyBundle {
    mesh: MaterialMesh2dBundle<ColorMaterial>,
    acceleration: Acceleration,
    velocity: Velocity,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle { ..default() });

    commands.spawn((
        Weather,
        State::default(),
        BodyBundle {
            mesh: MaterialMesh2dBundle {
                mesh: meshes.add(shape::RegularPolygon::new(16., 6).into()).into(),
                // 4. Put something bright in a dark environment to see the effect
                material: materials.add(ColorMaterial::from(Color::rgb(7.5, 0.0, 7.5))),
                transform: Transform::from_translation(Vec3::new(-200., 0., 0.)),
                ..default()
            },
            acceleration: Acceleration(Vec2::new(0., consts::GRAVITY_PER_SECOND)),
            ..Default::default()
        },
    ));
}

#[derive(Component)]
enum State {
    Normal {
        // weather has a limited number of jumps before it must reset
        // via the [`Climate`]
        jumps: u8,
        // there's a minimum delay between jumps
        last_jump: Stopwatch,
        // weather can only use its special ability once per reset
        has_used_special: bool,
    },
    LoadingSpecial {
        // while special is loading, the player can control an angle in which
        // it fires
        angle: f32,
        // special mode has a set duration after which it fires
        activated: Stopwatch,
        // once special is fired, weather can only do the same amount of jumps
        // as it had before
        jumps: u8,
    },
}

fn control_weather(
    mut query: Query<(&mut State, &mut Velocity, &mut Acceleration), With<Weather>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let pressed_left = keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A);
    let pressed_right =
        keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D);
    let pressed_space = keyboard_input.pressed(KeyCode::Space);

    let (mut state, mut vel, mut acc) = query.single_mut();

    let new_state = match &mut *state {
        State::Normal {
            jumps,
            has_used_special,
            ..
        } if !*has_used_special && pressed_space => Some(State::LoadingSpecial {
            angle: 0.0, // TODO
            activated: Stopwatch::default(),
            jumps: *jumps,
        }),
        &mut State::Normal {
            ref mut jumps,
            ref mut last_jump,
            ..
        } => {
            last_jump.tick(time.elapsed());

            let pressed_down =
                keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S);
            let just_pressed_up =
                keyboard_input.just_pressed(KeyCode::Up) || keyboard_input.just_pressed(KeyCode::W);

            if pressed_left {
                acc.0.x = -8.0;
                vel.0.x = vel.0.x.min(0.) - 25.0;
            }

            if pressed_right {
                acc.0.x = 8.0;
                vel.0.x = vel.0.x.max(0.) + 25.0;
            }

            // when down is pressed, the weather should fall faster
            if pressed_down {
                acc.0.y -= 2.0;
                vel.0.y = vel.0.y.min(0.) - 50.0;
            }

            if just_pressed_up
                && *jumps < consts::weather::MAX_JUMPS
                && last_jump.elapsed() > consts::weather::MIN_JUMP_DELAY
            {
                let jump_boost = (consts::weather::MAX_JUMPS + 1 - *jumps) as f32;

                *last_jump = Stopwatch::new();
                *jumps += 1;

                acc.0.y = consts::weather::JUMP_ACCELERATION;
                vel.0.y = (vel.0.y.max(0.) + consts::weather::JUMP_ACCELERATION * jump_boost)
                    .min(consts::GRAVITY_PER_SECOND * jump_boost);

                if pressed_left {
                    vel.0.x -= 15.0;
                }
                if pressed_right {
                    vel.0.x += 15.0;
                }
            }

            None
        }
        State::LoadingSpecial {
            activated, jumps, ..
        } if !pressed_space || activated.elapsed() > consts::weather::SPECIAL_LOADING_TIME => {
            Some(State::Normal {
                jumps: *jumps,
                last_jump: Stopwatch::default(),
                has_used_special: true,
            })
        }
        &mut State::LoadingSpecial {
            ref mut angle,
            ref mut activated,
            ..
        } => {
            activated.tick(time.elapsed());

            // set velocity and acceleration to 0 each frame
            // this means that the weather will slowly move down due to gravity
            vel.0 = Vec2::ZERO;
            acc.0 = Vec2::ZERO;

            if pressed_left {
                *angle -= 0.1;
            }

            if pressed_right {
                *angle += 0.1;
            }

            None
        }
    };

    if let Some(new_state) = new_state {
        *state = new_state;
    }
}

fn apply_acceleration(mut query: Query<(&mut Velocity, &mut Acceleration)>, time: Res<Time>) {
    let d = time.delta_seconds();

    for (mut vel, mut acc) in &mut query {
        // apply gravity
        acc.0.y -= consts::GRAVITY_PER_SECOND * d;

        // TODO
        vel.0.x -= vel.0.x * 0.75 * d;
        // acc.x *= 0.8;

        // TODO
        // clamp acceleration
        // acc.0.x = acc.0.x.clamp(-1000.0, 1000.0);
        acc.0.y = acc.0.y.clamp(-1000.0, f32::MAX);
        // // clamp velocity
        // vel.0.x = vel.0.x.clamp(-4000.0, 4000.0);
        vel.0.y = vel.0.y.clamp(-600.0, f32::MAX);

        vel.x += acc.x * d;
        vel.y += acc.y * d;
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * time.delta_seconds();
        transform.translation.y += vel.y * time.delta_seconds();
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Normal {
            jumps: 0,
            has_used_special: false,
            last_jump: Stopwatch::default(),
        }
    }
}
