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
            (
                control_normal,
                control_loading_special,
                apply_acceleration,
                apply_velocity,
            )
                .chain(),
        )
        .run();
}

#[derive(Component, Default, Deref, DerefMut)]
struct Acceleration(Vec2);
#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

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
        mode::Normal::default(),
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

pub(crate) mod mode {
    use std::time::Duration;

    use bevy::{ecs::component::Component, time::Stopwatch};

    pub(crate) trait Mode {
        fn tick(&mut self, elapsed: Duration);
    }

    #[derive(Component, Default)]
    pub(crate) struct Normal {
        // weather has a limited number of jumps before it must reset
        // via the [`Climate`]
        pub(crate) jumps: u8,
        // there's a minimum delay between jumps
        pub(crate) last_jump: Stopwatch,
        // weather can only use its special ability once per reset
        pub(crate) has_used_special: bool,
    }

    #[derive(Component, Default)]
    pub(crate) struct LoadingSpecial {
        // while special is loading, the player can control an angle in which
        // it fires
        pub(crate) angle: f32,
        // special mode has a set duration after which it fires
        pub(crate) activated: Stopwatch,
        // once special is fired, weather can only do the same amount of jumps
        // as it had before
        pub(crate) jumps: u8,
    }

    impl Mode for Normal {
        fn tick(&mut self, elapsed: Duration) {
            self.last_jump.tick(elapsed);
        }
    }

    impl Mode for LoadingSpecial {
        fn tick(&mut self, elapsed: Duration) {
            self.activated.tick(elapsed);
        }
    }
}

use mode::Mode;

fn control_loading_special(
    mut weather: Query<(
        Entity,
        &mut mode::LoadingSpecial,
        &mut Velocity,
        &mut Acceleration,
    )>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, mut mode, mut vel, mut acc)) = weather.get_single_mut() else {
        return;
    };
    mode.tick(time.delta());

    let pressed_space = keyboard.pressed(KeyCode::Space);
    let pressed_left = keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
    let pressed_right = keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);

    if !pressed_space || mode.activated.elapsed() > consts::weather::SPECIAL_LOADING_TIME {
        commands.entity(entity).insert(mode::Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            has_used_special: true,
        });
        // TODO
        commands.entity(entity).remove::<mode::LoadingSpecial>();
    } else {
        // set velocity and acceleration to 0 each frame
        // this means that the weather will slowly move down due to gravity
        vel.0 = Vec2::ZERO;
        acc.0 = Vec2::ZERO;

        if pressed_left {
            mode.angle = mode.angle - 0.1; // TODO
        }

        if pressed_right {
            mode.angle = mode.angle + 0.1; // TODO
        }
    }
}

fn control_normal(
    mut weather: Query<(Entity, &mut mode::Normal, &mut Velocity, &mut Acceleration)>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, mut mode, mut vel, mut acc)) = weather.get_single_mut() else {
        return;
    };
    mode.tick(time.delta());

    let pressed_space = keyboard.pressed(KeyCode::Space);

    if !mode.has_used_special && pressed_space {
        commands.entity(entity).insert(mode::LoadingSpecial {
            angle: 0.0, // TODO
            activated: Stopwatch::default(),
            jumps: mode.jumps,
        });
        // TODO
        commands.entity(entity).remove::<mode::Normal>();
    } else {
        let pressed_left = keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
        let pressed_right = keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);
        let pressed_down = keyboard.pressed(KeyCode::Down) || keyboard.pressed(KeyCode::S);
        let just_pressed_up =
            keyboard.just_pressed(KeyCode::Up) || keyboard.just_pressed(KeyCode::W);

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
            && mode.jumps < consts::weather::MAX_JUMPS
            && mode.last_jump.elapsed() > consts::weather::MIN_JUMP_DELAY
        {
            let jump_boost = (consts::weather::MAX_JUMPS + 1 - mode.jumps) as f32;

            mode.last_jump = Stopwatch::new();
            mode.jumps = mode.jumps + 1;

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
