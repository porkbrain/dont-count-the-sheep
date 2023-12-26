use main_game_lib::{
    GlobalGameStateTransition, GlobalGameStateTransitionStack,
};

use crate::{climate::Climate, distractions::Distraction, prelude::*};

use super::consts::*;

#[derive(Component)]
pub(super) struct Menu {
    selection: Selection,
}

#[derive(Default, Debug, Clone, Copy)]
enum Selection {
    #[default]
    Resume = 0,
    Restart = 1,
    Quit = 2,
}

pub(super) fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Menu {
            selection: Selection::Resume,
        })
        .insert(NodeBundle {
            style: Style {
                // the node bundle units don't honor pixel camera 3x scale
                width: Val::Px(MENU_BOX_WIDTH),
                height: Val::Px(MENU_BOX_HEIGHT),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| spawn_ui(parent, &asset_server));
}

pub(super) fn despawn(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    if let Ok(entity) = menu.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}

pub(super) fn open(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut distractions: Query<&mut Distraction>, // TODO: move
    mut climate: Query<&mut Climate>,          // TODO: move
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    debug!("Pausing to open menu");
    keyboard.clear(); // prevent accidental immediate unpausing

    next_state.set(GlobalGameState::MeditationInMenu);

    // TODO: pause weather

    for mut distraction in distractions.iter_mut() {
        distraction.pause();
    }

    climate.single_mut().pause();
}

pub(super) fn close(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut distractions: Query<&mut Distraction>, // TODO: move
    mut climate: Query<&mut Climate>,          // TODO: move
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    debug!("Closing menu and unpausing");

    // prevent accidental immediate unpausing
    keyboard.clear();
    // we simulate press to close the menu, so we need to simulate release
    keyboard.release(KeyCode::Escape);

    next_state.set(GlobalGameState::MeditationInGame);

    for mut distraction in distractions.iter_mut() {
        distraction.resume();
    }

    climate.single_mut().resume();
}

/// The order of the systems is important.
/// We simulate ESC to close the menu.
/// So we need to select before we close.
///
/// TODO: transition into a quitting state
pub(super) fn select(
    mut stack: ResMut<GlobalGameStateTransitionStack>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut menu: Query<&mut Menu>,
    mut keyboard: ResMut<Input<KeyCode>>,
) {
    let Ok(mut menu) = menu.get_single_mut() else {
        return;
    };

    let curr_selection = menu.selection;

    if keyboard.just_pressed(KeyCode::Return) {
        debug!("Going with {curr_selection:?}");

        match curr_selection {
            Selection::Resume => keyboard.press(KeyCode::Escape),
            Selection::Restart => {
                stack.push(GlobalGameStateTransition::MeditationQuittingToMeditationLoading);
                next_state.set(GlobalGameState::MeditationQuitting);
            }
            Selection::Quit => {
                stack.push(GlobalGameStateTransition::MeditationQuittingToExit);
                next_state.set(GlobalGameState::MeditationQuitting);
            }
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
        menu.selection = new_selection;
    }
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
        UiImage::new(asset_server.load(assets::MENU_BOX)),
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
            Self::Restart => Self::Quit,
            Self::Quit => Self::Resume,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Resume => Self::Quit,
            Self::Restart => Self::Resume,
            Self::Quit => Self::Restart,
        }
    }
}
