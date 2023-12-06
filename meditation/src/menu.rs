use bevy::app::AppExit;

use crate::{control_mode, prelude::*};

mod consts {
    use bevy::math::Vec2;

    pub(crate) const FIRST_SELECTION_FACE_OFFSET: Vec2 = Vec2::new(-80.0, 50.0);
    pub(crate) const SELECTIONS_SPACING: f32 =
        crate::weather::consts::FACE_RENDERED_SIZE + 4.0;
}

pub(crate) fn spawn(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let bounding_box = commands
        .spawn((
            Menu,
            SpriteBundle {
                texture: asset_server.load("textures/menu/box.png"),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    zindex::MENU,
                )),
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .id();

    let options = commands
        .spawn((SpriteBundle {
            texture: asset_server.load("textures/menu/options.png"),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                0.0,
                zindex::MENU,
            )),
            ..default()
        },))
        .id();

    commands.entity(bounding_box).add_child(options);
}

#[derive(Component)]
pub(crate) struct Menu;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) enum Selection {
    #[default]
    Resume = 0,
    Restart = 1,
    GodMode = 2,
    Quit = 3,
}

pub(crate) fn open(
    mut weather: Query<
        (
            Entity,
            &mut control_mode::Normal,
            &mut Transform,
            &mut Visibility,
            &Velocity,
        ),
        Without<Menu>, // to make bevy be sure there won't be conflicts
    >,
    mut menu: Query<&mut Visibility, With<Menu>>,
    mut commands: Commands,
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    if keyboard.pressed(KeyCode::Escape) {
        // trace!("Pressed esc: {}", keyboard.just_pressed(KeyCode::Escape));
    }

    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    let Ok((entity, mut mode, mut transform, mut visibility, vel)) =
        weather.get_single_mut()
    else {
        return;
    };

    debug!("Pausing to open menu");
    keyboard.clear(); // prevent accidental immediate unpausing

    *visibility = Visibility::Hidden;
    commands.entity(entity).remove::<control_mode::Normal>();
    commands.entity(entity).remove::<Velocity>();
    commands.entity(entity).insert(control_mode::InMenu {
        selection: Selection::default(),
        from_mode: { mode.pause().clone() },
        from_velocity: *vel,
        from_transform: *transform,
    });

    let mut menu_visibility = menu.single_mut();
    *menu_visibility = Visibility::Visible;

    *transform = Transform::from_translation(Vec3::new(
        consts::FIRST_SELECTION_FACE_OFFSET.x,
        consts::FIRST_SELECTION_FACE_OFFSET.y,
        zindex::WEATHER_IN_MENU,
    ));
}

pub(crate) fn close(
    mut weather: Query<
        (
            Entity,
            &control_mode::InMenu,
            &mut Transform,
            &mut Visibility,
        ),
        Without<Menu>, // to make bevy be sure there won't be conflicts
    >,
    mut menu: Query<&mut Visibility, With<Menu>>,
    mut commands: Commands,
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    let Ok((entity, mode, mut transform, mut visibility)) =
        weather.get_single_mut()
    else {
        return;
    };

    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    debug!("Closing menu and unpausing");
    // prevent accidental immediate unpausing
    keyboard.clear();
    // we simulate press to close the menu, so we need to simulate release
    keyboard.release(KeyCode::Escape);

    commands.entity(entity).remove::<control_mode::InMenu>();
    commands.entity(entity).insert({
        let mut mode = mode.from_mode.clone();
        mode.unpause();
        mode
    });
    commands.entity(entity).insert(mode.from_velocity);
    *transform = mode.from_transform;
    *visibility = Visibility::Visible;

    let mut menu_visibility = menu.single_mut();
    *menu_visibility = Visibility::Hidden;
}

/// The order of the systems is important.
/// We simulate ESC to close the menu.
/// So we need to select before we close.
pub(crate) fn select(
    mut menu: Query<(&mut control_mode::InMenu, &mut Transform)>,
    mut keyboard: ResMut<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    let Ok((mut mode, mut transform)) = menu.get_single_mut() else {
        return;
    };

    let curr_selection = mode.selection;

    if keyboard.just_pressed(KeyCode::Return) {
        debug!("Going with {curr_selection:?}");

        match curr_selection {
            Selection::Resume => keyboard.press(KeyCode::Escape),
            Selection::Restart => {
                mode.from_mode = default();
                mode.from_transform = crate::weather::consts::DEFAULT_TRANSFORM;
                mode.from_velocity = default();
                // TODO: weather angular velocity

                keyboard.press(KeyCode::Escape);
            }
            Selection::GodMode => {
                mode.from_mode.god_mode = !mode.from_mode.god_mode;
                // TODO: make it visually clear whether god mode is on or off
            }
            Selection::Quit => exit.send(AppExit),
        }

        return;
    }

    let pressed_up =
        keyboard.just_pressed(KeyCode::Up) || keyboard.just_pressed(KeyCode::W);
    let pressed_down = keyboard.just_pressed(KeyCode::Down)
        || keyboard.just_pressed(KeyCode::S);

    let new_selection = if pressed_up {
        Some(curr_selection.prev())
    } else if pressed_down {
        Some(curr_selection.next())
    } else {
        None
    };

    if let Some(new_selection) = new_selection {
        debug!("Selected {curr_selection:?}");
        mode.selection = new_selection;
        transform.translation.y = consts::FIRST_SELECTION_FACE_OFFSET.y
            - consts::SELECTIONS_SPACING * new_selection as u8 as f32;
    }
}

impl Selection {
    fn next(&self) -> Self {
        match self {
            Self::Resume => Self::Restart,
            Self::Restart => Self::GodMode,
            Self::GodMode => Self::Quit,
            Self::Quit => Self::Resume,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Resume => Self::Quit,
            Self::Restart => Self::Resume,
            Self::GodMode => Self::Restart,
            Self::Quit => Self::GodMode,
        }
    }
}
