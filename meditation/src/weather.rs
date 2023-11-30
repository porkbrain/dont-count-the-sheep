use crate::prelude::*;
use bevy::{sprite::MaterialMesh2dBundle, time::Stopwatch};
use mode::Mode;

pub(crate) fn spawn(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        mode::Normal::default(),
        BodyBundle {
            // TODO: use a sprite
            mesh: MaterialMesh2dBundle {
                mesh: meshes
                    .add(shape::RegularPolygon::new(16., 6).into())
                    .into(),
                material: materials
                    .add(ColorMaterial::from(Color::rgb(7.5, 0.0, 7.5))),
                transform: Transform::from_translation(Vec3::new(
                    -200., 0., 0.,
                )),
                ..default()
            },
            acceleration: Acceleration::new(Vec2::new(
                0.,
                consts::GRAVITY_PER_SECOND,
            )),
            ..Default::default()
        },
    ));
}

pub(crate) mod mode {
    use bevy::{
        ecs::component::Component,
        time::{Stopwatch, Time},
    };

    pub(crate) trait Mode {
        fn tick(&mut self, time: &Time);
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
        fn tick(&mut self, time: &Time) {
            self.last_jump.tick(time.delta());
        }
    }

    impl Mode for LoadingSpecial {
        fn tick(&mut self, time: &Time) {
            self.activated.tick(time.delta());
        }
    }
}

pub(crate) fn control_loading_special(
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
    let Ok((entity, mut mode, mut vel, mut acc)) = weather.get_single_mut()
    else {
        return;
    };
    mode.tick(&time);

    let pressed_space = keyboard.pressed(KeyCode::Space);
    let pressed_left =
        keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
    let pressed_right =
        keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);

    if !pressed_space
        || mode.activated.elapsed() > consts::weather::SPECIAL_LOADING_TIME
    {
        commands.entity(entity).remove::<mode::LoadingSpecial>();
        commands.entity(entity).insert(mode::Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            has_used_special: true,
        });

        return;
    }

    // set velocity and acceleration to 0 each frame
    // this means that the weather will slowly move down due to gravity
    *vel = Default::default();
    *acc = Default::default();

    if pressed_left {
        mode.angle = mode.angle - 0.1; // TODO
    }

    if pressed_right {
        mode.angle = mode.angle + 0.1; // TODO
    }
}

pub(crate) fn control_normal(
    mut weather: Query<(
        Entity,
        &mut mode::Normal,
        &mut Velocity,
        &mut Acceleration,
    )>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, mut mode, mut vel, mut acc)) = weather.get_single_mut()
    else {
        return;
    };
    mode.tick(&time);

    let pressed_space = keyboard.pressed(KeyCode::Space);

    if !mode.has_used_special && pressed_space {
        commands.entity(entity).remove::<mode::Normal>();
        commands.entity(entity).insert(mode::LoadingSpecial {
            angle: 0.0, // TODO
            activated: Stopwatch::default(),
            jumps: mode.jumps,
        });
        return;
    }

    let d = time.delta_seconds();
    // apply gravity
    acc.y -= consts::GRAVITY_PER_SECOND * d;
    // slow down horizontal movement over time
    vel.x -= vel.x * d;

    let pressed_left =
        keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
    let pressed_right =
        keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);
    let pressed_down =
        keyboard.pressed(KeyCode::Down) || keyboard.pressed(KeyCode::S);
    let just_pressed_up =
        keyboard.just_pressed(KeyCode::Up) || keyboard.just_pressed(KeyCode::W);

    if pressed_left {
        acc.x = -8.0;
        vel.x = vel.x.min(0.) - 25.0;
    }

    if pressed_right {
        acc.x = 8.0;
        vel.x = vel.x.max(0.) + 25.0;
    }

    // when down is pressed, the weather should fall faster
    if pressed_down {
        acc.y -= 2.0;
        vel.y = vel.y.min(0.) - 50.0;
    }

    if just_pressed_up
        && mode.jumps < consts::weather::MAX_JUMPS
        && mode.last_jump.elapsed() > consts::weather::MIN_JUMP_DELAY
    {
        // each jump is less and less strong until reset
        let jump_boost = (consts::weather::MAX_JUMPS + 1 - mode.jumps) as f32;

        mode.last_jump = Stopwatch::new();
        mode.jumps = mode.jumps + 1;

        acc.y = consts::weather::JUMP_ACCELERATION;
        vel.y = (vel.y.max(0.)
            + consts::weather::JUMP_ACCELERATION * jump_boost)
            .min(consts::GRAVITY_PER_SECOND * jump_boost);

        if pressed_left {
            vel.x -= consts::weather::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMPING;
        }
        if pressed_right {
            vel.x += consts::weather::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMPING;
        }
    }

    // TODO
    // clamp acceleration
    // acc.x = acc.x.clamp(-1000.0, 1000.0);
    acc.y = acc.y.clamp(-1000.0, f32::MAX);
    // // clamp velocity
    // vel.x = vel.x.clamp(-4000.0, 4000.0);
    vel.y = vel.y.clamp(-600.0, f32::MAX);
}
