use crate::prelude::*;
use bevy::{sprite::MaterialMesh2dBundle, time::Stopwatch};
use mode::Mode;

mod consts {
    use std::time::Duration;
    /// How many pixels per second pulls weather down.
    pub(crate) const GRAVITY: f32 = 256.0;

    /// After jumping, weather gets accelerated by this much up.
    /// Existing acceleration is overwritten.
    pub(crate) const JUMP_ACCELERATION: f32 = GRAVITY / 4.0;
    /// Pressing jump won't do anything if the last jump was less than this
    pub(crate) const MIN_JUMP_DELAY: Duration = Duration::from_millis(150);
    /// Maximum amount of time weather can be selecting the angle of its special
    /// before it fires.
    pub(crate) const SPECIAL_LOADING_TIME: Duration =
        Duration::from_millis(1500);
    /// Cannot jump more times in a row than this before resetting.
    pub(crate) const MAX_JUMPS: u8 = 4;
    /// When left/right is pressed while jumping weather gets an extra kick
    pub(crate) const HORIZONTAL_VELOCITY_BOOST_WHEN_JUMPING: f32 = 400.0;
    /// When down is pressed, weather's vertical velocity is set to this value
    pub(crate) const VERTICAL_VELOCITY_WHEN_PRESSED_DOWN: f32 = -600.0;
    /// Caps gravity effect and if weather is falling faster than this, it
    /// starts to slow down.
    pub(crate) const TERMINAL_VELOCITY: f32 = -250.0;
}

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
            acceleration: Acceleration::new(Vec2::new(0., consts::GRAVITY)),
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

    #[derive(Component)]
    pub(crate) struct Normal {
        // weather has a limited number of jumps before it must reset
        // via the [`Climate`]
        pub(crate) jumps: u8,
        // there's a minimum delay between jumps
        pub(crate) last_jump: Stopwatch,
        // weather can only use its special ability once per reset
        pub(crate) can_use_special: bool,
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

    impl Default for Normal {
        fn default() -> Self {
            Self {
                jumps: 0,
                last_jump: Stopwatch::default(),
                can_use_special: true,
            }
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

    if !pressed_space || mode.activated.elapsed() > consts::SPECIAL_LOADING_TIME
    {
        commands.entity(entity).remove::<mode::LoadingSpecial>();
        commands.entity(entity).insert(mode::Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            can_use_special: false,
        });

        // TODO: fire!!!

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

    if mode.can_use_special && pressed_space {
        commands.entity(entity).remove::<mode::Normal>();
        commands.entity(entity).insert(mode::LoadingSpecial {
            angle: 0.0, // TODO
            activated: Stopwatch::default(),
            jumps: mode.jumps,
        });
        return;
    }

    let d = time.delta_seconds();
    // apply friction
    vel.x -= vel.x * d;

    let pressed_left =
        keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
    let pressed_right =
        keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);
    let pressed_down =
        keyboard.pressed(KeyCode::Down) || keyboard.pressed(KeyCode::S);
    let just_pressed_up =
        keyboard.just_pressed(KeyCode::Up) || keyboard.just_pressed(KeyCode::W);
    let just_pressed_left = keyboard.just_pressed(KeyCode::Left)
        || keyboard.just_pressed(KeyCode::A);
    let just_pressed_right = keyboard.just_pressed(KeyCode::Right)
        || keyboard.just_pressed(KeyCode::D);

    enum Direction {
        Left,
        Right,
    }
    // TODO: sucks
    let mut update_horizontal =
        |dir: Direction, just_pressed: bool, pressed: bool| {
            use Direction::*;
            let discrim = match dir {
                Left => -1.0,
                Right => 1.0,
            };
            if just_pressed {
                acc.x = 0.0;
                vel.x = match dir {
                    Left => vel.x.min(0.),
                    Right => vel.x.max(0.),
                } + discrim * 150.0;
            } else if pressed {
                acc.x = discrim * 200.0;
            }
        };

    update_horizontal(Direction::Left, just_pressed_left, pressed_left);
    update_horizontal(Direction::Right, just_pressed_right, pressed_right);

    if pressed_down {
        // the downward movement is stabilized
        vel.y = consts::VERTICAL_VELOCITY_WHEN_PRESSED_DOWN;
    } else {
        if vel.y < consts::TERMINAL_VELOCITY {
            // evenly slow down to terminal velocity
            vel.y += {
                debug_assert!(consts::TERMINAL_VELOCITY < 0.0);
                let diff = -vel.y + consts::TERMINAL_VELOCITY;
                // always slow down at least 1 pixel per second to avoid
                // infinite approach
                (diff * d).max(1.0)
            };
        } else {
            // apply gravity
            vel.y =
                (vel.y - consts::GRAVITY * d).max(consts::TERMINAL_VELOCITY);
        }
    }

    if just_pressed_up
        && mode.jumps < consts::MAX_JUMPS
        && mode.last_jump.elapsed() > consts::MIN_JUMP_DELAY
    {
        // each jump is less and less strong until reset
        let jump_boost = (consts::MAX_JUMPS + 1 - mode.jumps) as f32;

        mode.last_jump = Stopwatch::new();
        // TODO: only in god mode
        // mode.jumps = mode.jumps + 1;

        // TODO: sucks
        vel.y = (vel.y.max(0.) + consts::JUMP_ACCELERATION * jump_boost)
            .min(consts::GRAVITY * jump_boost);

        if pressed_left {
            acc.x += 400.0;
            vel.x -= consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMPING;
        }
        if pressed_right {
            acc.x -= 400.0;
            vel.x += consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMPING;
        }
    }

    // apply acceleration
    vel.x += acc.x * d;
}
