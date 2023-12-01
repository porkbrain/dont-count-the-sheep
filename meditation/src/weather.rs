use crate::prelude::*;
use bevy::{sprite::MaterialMesh2dBundle, time::Stopwatch};
use mode::Mode;

mod consts {
    use std::time::Duration;

    /// How many pixels per second pulls weather down.
    pub(crate) const GRAVITY: f32 = 512.0;
    /// Pressing up does nothing if the last jump was less than this
    pub(crate) const MIN_JUMP_DELAY: Duration = Duration::from_millis(200);
    /// Pressing left/right does nothing if the last dash was less than this
    pub(crate) const MIN_DASH_DELAY: Duration = Duration::from_millis(500);
    /// Pressing down does nothing if the last dip was less than this
    pub(crate) const MIN_DIP_DELAY: Duration = Duration::from_millis(200);
    /// Maximum amount of time weather can be selecting the angle of its special
    /// before it fires.
    pub(crate) const SPECIAL_LOADING_TIME: Duration =
        Duration::from_millis(1500);
    /// Cannot jump more times in a row than this before resetting.
    pub(crate) const MAX_JUMPS: u8 = 6;
    /// When left/right is pressed while up/down then weather gets an extra kick
    pub(crate) const HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP: f32 = 176.0;
    /// When down is pressed, weather's vertical velocity is set to this value
    pub(crate) const VERTICAL_VELOCITY_ON_DIP: f32 = -600.0;
    /// Caps gravity effect and if weather is falling faster than this, it
    /// starts to slow down.
    pub(crate) const TERMINAL_VELOCITY: f32 = -300.0;
    /// When left/right is pressed, weather gets an extra kick
    pub(crate) const DASH_VELOCITY_BOOST: f32 = 216.0;
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
        // there's a minimum delay between dashes
        pub(crate) last_dash: Stopwatch,
        // there's a minimum delay between dips
        pub(crate) last_dip: Stopwatch,
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
            self.last_dash.tick(time.delta());
            self.last_dip.tick(time.delta());
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
                last_dash: Stopwatch::default(),
                last_jump: Stopwatch::default(),
                last_dip: Stopwatch::default(),
                can_use_special: true,
            }
        }
    }
}

pub(crate) fn control_loading_special(
    mut weather: Query<(Entity, &mut mode::LoadingSpecial, &mut Velocity)>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, mut mode, mut vel)) = weather.get_single_mut() else {
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
        //  perhaps if timed right it's better?
        commands.entity(entity).remove::<mode::LoadingSpecial>();
        commands.entity(entity).insert(mode::Normal {
            jumps: mode.jumps,
            last_jump: Stopwatch::default(),
            last_dash: Stopwatch::default(),
            last_dip: {
                let mut t = Stopwatch::default();
                t.tick(consts::MIN_DIP_DELAY * 2);
                t
            },
            can_use_special: false,
        });

        // TODO: fire!!!

        return;
    }

    // set velocity and acceleration to 0 each frame
    // this means that the weather will slowly move down due to gravity
    *vel = Default::default();

    if pressed_left {
        mode.angle = mode.angle - 0.1; // TODO
    }

    if pressed_right {
        mode.angle = mode.angle + 0.1; // TODO
    }
}

pub(crate) fn control_normal(
    mut weather: Query<(Entity, &mut mode::Normal, &mut Velocity)>,
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((entity, mut mode, mut vel)) = weather.get_single_mut() else {
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

    let pressed_left =
        keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
    let pressed_right =
        keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);
    let just_pressed_down = keyboard.just_pressed(KeyCode::Down)
        || keyboard.just_pressed(KeyCode::S);
    let just_pressed_up =
        keyboard.just_pressed(KeyCode::Up) || keyboard.just_pressed(KeyCode::W);

    enum Direction {
        Left,
        Right,
    }
    let mut update_horizontal = |dir: Direction, pressed: bool| {
        use Direction::*;
        if pressed && mode.last_dash.elapsed() > consts::MIN_DASH_DELAY {
            mode.last_dash = Stopwatch::new();

            let discrim = match dir {
                Left => -1.0,
                Right => 1.0,
            };

            vel.x = match dir {
                Left => vel.x.min(0.),
                Right => vel.x.max(0.),
            } + discrim * consts::DASH_VELOCITY_BOOST;
        }
    };

    update_horizontal(Direction::Left, pressed_left);
    update_horizontal(Direction::Right, pressed_right);

    if just_pressed_down && mode.last_dip.elapsed() > consts::MIN_DIP_DELAY {
        mode.last_dip = Stopwatch::new();

        // the downward movement is stabilized
        vel.y = consts::VERTICAL_VELOCITY_ON_DIP;

        if pressed_left {
            vel.x -= consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
        if pressed_right {
            vel.x += consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
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
        mode.jumps = mode.jumps + 1;
        mode.last_jump = Stopwatch::new();

        // each jump is less and less strong until reset
        let jump_boost = (consts::MAX_JUMPS + 1 - mode.jumps) as f32
            / consts::MAX_JUMPS as f32;

        // TODO: sucks
        vel.y = 300.0 + 300.0 * jump_boost;

        if pressed_left {
            vel.x -= consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
        if pressed_right {
            vel.x += consts::HORIZONTAL_VELOCITY_BOOST_WHEN_JUMP_OR_DIP;
        }
    }

    // apply friction to the horizontal movement
    vel.x -= vel.x * d;
}
