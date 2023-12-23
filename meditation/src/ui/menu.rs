use bevy::app::AppExit;

use crate::{
    climate::Climate, control_mode, distractions::Distraction, prelude::*,
};

use super::consts::*;

#[derive(Component)]
pub(super) struct Menu;

#[derive(Component)]
pub(super) struct GodModeToggle;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) enum Selection {
    #[default]
    Resume = 0,
    Restart = 1,
    GodMode = 2,
    Quit = 3,
}

pub(super) fn open(
    game: Query<Entity, With<Game>>,
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
    mut distractions: Query<&mut Distraction>,
    mut climate: Query<&mut Climate>,
    mut menu: Query<&mut Visibility, With<Menu>>,
    mut god_mode: Query<&mut Text, With<GodModeToggle>>,
    mut commands: Commands,
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    let Ok(game) = game.get_single() else {
        return;
    };

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

    commands.entity(game).insert(Paused);

    *visibility = Visibility::Hidden;
    commands.entity(entity).remove::<control_mode::Normal>();
    commands.entity(entity).remove::<Velocity>();
    commands.entity(entity).insert(control_mode::InMenu {
        selection: Selection::default(),
        from_mode: { mode.pause().clone() },
        from_velocity: *vel,
        from_transform: *transform,
    });

    {
        let mut menu_visibility = menu.single_mut();
        *menu_visibility = Visibility::Visible;

        *transform = Transform::from_translation(Vec3::new(
            FIRST_SELECTION_FACE_OFFSET.x,
            FIRST_SELECTION_FACE_OFFSET.y,
            zindex::WEATHER_IN_MENU,
        ));
    }

    // updates the text of the second section, ie. after "GOD MODE:"
    god_mode.single_mut().sections[1].value = if mode.god_mode {
        "ON".to_string()
    } else {
        "OFF".to_string()
    };

    for mut distraction in distractions.iter_mut() {
        distraction.pause();
    }

    climate.single_mut().pause();
}

pub(super) fn close(
    game: Query<Entity, With<Game>>,
    mut weather: Query<
        (
            Entity,
            &control_mode::InMenu,
            &mut Transform,
            &mut Visibility,
        ),
        Without<Menu>, // to make bevy be sure there won't be conflicts
    >,
    mut distractions: Query<&mut Distraction>,
    mut climate: Query<&mut Climate>,
    mut menu: Query<&mut Visibility, With<Menu>>,
    mut commands: Commands,
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    let Ok(game) = game.get_single() else {
        return;
    };

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

    commands.entity(game).remove::<Paused>();

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

    for mut distraction in distractions.iter_mut() {
        distraction.resume();
    }

    climate.single_mut().resume();
}

/// The order of the systems is important.
/// We simulate ESC to close the menu.
/// So we need to select before we close.
pub(super) fn select(
    mut menu: Query<(&mut control_mode::InMenu, &mut Transform)>,
    mut god_mode: Query<&mut Text, With<GodModeToggle>>,
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
            // TODO: proper reset of the whole game
            Selection::Restart => {
                // preserve god mode, but reset the rest
                let god_mode = mode.from_mode.god_mode;
                mode.from_mode = default();
                mode.from_mode.god_mode = god_mode;

                mode.from_transform = crate::weather::consts::DEFAULT_TRANSFORM;
                mode.from_velocity = default();

                keyboard.press(KeyCode::Escape);
            }
            Selection::GodMode => {
                mode.from_mode.god_mode = !mode.from_mode.god_mode;

                // updates the text of the second section, ie. after "GOD MODE:"
                god_mode.single_mut().sections[1].value =
                    if mode.from_mode.god_mode {
                        "ON".to_string()
                    } else {
                        "OFF".to_string()
                    };
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
        transform.translation.y = FIRST_SELECTION_FACE_OFFSET.y
            - SELECTIONS_SPACING * new_selection as u8 as f32;
    }
}

pub(super) fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Menu,
            NodeBundle {
                style: Style {
                    // the node bundle units don't honor pixel camera 3x scale
                    width: Val::Px(MENU_BOX_WIDTH),
                    height: Val::Px(MENU_BOX_HEIGHT),
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|parent| spawn_ui(parent, &asset_server));
}

fn spawn_ui(ui_root: &mut ChildBuilder, asset_server: &Res<AssetServer>) {
    // displays see through box around the menu options
    ui_root.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            // a `NodeBundle` is transparent by default, so
            // to see the image we have to its color to
            // `WHITE`
            background_color: Color::WHITE.into(),
            ..default()
        },
        UiImage::new(asset_server.load("ui/menu_box.png")),
    ));

    // positions the menu options
    ui_root
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                left: SELECTIONS_LEFT_OFFSET,
                top: SELECTIONS_TOP_OFFSET,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "CONTINUE",
                TextStyle {
                    font: asset_server.load(FONT),
                    font_size: BIG_FONT_SIZE,
                    ..default()
                },
            ));
            parent.spawn(
                TextBundle::from_section(
                    "RESTART",
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: BIG_FONT_SIZE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(SELECTIONS_PADDING_TOP),
                    ..default()
                }),
            );
            parent.spawn((
                GodModeToggle,
                TextBundle::from_sections([
                    TextSection {
                        value: "GOD MODE: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load(FONT),
                            font_size: BIG_FONT_SIZE,
                            ..default()
                        },
                    },
                    TextSection::from_style(TextStyle {
                        font: asset_server.load(FONT),
                        font_size: SMALL_FONT_SIZE,
                        color: Color::hex(HIGHLIGHT_COLOR).unwrap(),
                    }),
                ])
                .with_style(Style {
                    margin: UiRect::top(SELECTIONS_PADDING_TOP),
                    ..default()
                }),
            ));
            parent.spawn(
                TextBundle::from_section(
                    "EXIT",
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: BIG_FONT_SIZE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(SELECTIONS_PADDING_TOP),
                    ..default()
                }),
            );
        });
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
