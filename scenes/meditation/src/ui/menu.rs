use std::time::Instant;

use common_loading_screen::{LoadingScreenSettings, LoadingScreenState};
use common_visuals::camera::PIXEL_ZOOM;
use main_game_lib::common_ext::QueryExt;

use super::consts::*;
use crate::{cameras::MeditationCamera, consts::*, prelude::*};

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

/// We render a small image and move it based on which selection is currently
/// active to give the player some visual feedback.
#[derive(Component)]
pub(super) struct SelectionMarker;

pub(super) fn spawn(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,

    camera: Query<Entity, With<MeditationCamera>>,
) {
    cmd.spawn(Menu {
        selection: Selection::Resume,
    })
    .insert(Name::new("Menu"))
    .insert(TargetCamera(camera.single()))
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

pub(super) fn despawn(mut cmd: Commands, menu: Query<Entity, With<Menu>>) {
    if let Some(entity) = menu.get_single_or_none() {
        cmd.entity(entity).despawn_recursive();
    }
}

pub(super) fn open(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    debug!("Pausing to open menu");
    // prevent accidental immediate unpausing
    controls.consume(&GlobalAction::Cancel);

    next_state.set(GlobalGameState::MeditationInMenu);
}

pub(super) fn close(
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,
) {
    debug!("Closing menu and unpausing");
    // prevent accidental immediate pausing
    controls.consume(&GlobalAction::Cancel);

    next_state.set(GlobalGameState::InGameMeditation);
}

pub(super) fn change_selection(
    controls: Res<ActionState<GlobalAction>>,
    asset_server: Res<AssetServer>,

    mut menu: Query<&mut Menu>,
    mut selection_marker: Query<
        (&mut Style, &mut UiImage),
        With<SelectionMarker>,
    >,

    mut last_changed: Local<Option<Instant>>,
) {
    let Some(action) = controls.movement_action() else {
        return;
    };

    // The player does not have to release the button to keep changing the
    // selection, it's enough to hold it as well and we smoothly change the
    // selection.
    // Very useful for control sticks.
    let elapsed_since_changed =
        last_changed.get_or_insert_with(Instant::now).elapsed();
    let should_trigger = controls.just_pressed(&GlobalAction::Move)
        || elapsed_since_changed > Duration::from_millis(250);
    if !should_trigger {
        return;
    }
    *last_changed = Some(Instant::now());

    let mut menu = menu.single_mut();
    let curr_selection = menu.selection;

    let new_selection = if action.is_in_up_direction() {
        Some(curr_selection.prev())
    } else if action.is_in_down_direction() {
        Some(curr_selection.next())
    } else {
        None
    };

    if let Some(new_selection) = new_selection {
        menu.selection = new_selection;

        let (mut style, mut image) = selection_marker.single_mut();

        style.top = Val::Px(
            SELECTION_MARKER_TOP_OFFSET_PX
                + new_selection as u8 as f32
                    * SELECTION_MARKER_TOP_PADDING_PX_PER_SELECTION,
        );

        // this is ugly, promise you won't tell xx
        *image = UiImage::new(asset_server.load(match new_selection {
            Selection::Resume => assets::FACE_ON_CONTINUE,
            Selection::Restart => assets::FACE_ON_RESTART,
            Selection::Quit => assets::FACE_ON_EXIT,
        }));
    }
}

/// The order of the systems is important.
/// We simulate ESC to close the menu.
/// So we need to select before we close.
pub(super) fn select(
    mut cmd: Commands,
    mut transition: ResMut<GlobalGameStateTransition>,
    mut next_state: ResMut<NextState<GlobalGameState>>,
    mut next_loading_state: ResMut<NextState<LoadingScreenState>>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    menu: Query<&Menu>,
) {
    let curr_selection = menu.single().selection;

    debug!("Going with {curr_selection:?}");

    match curr_selection {
        Selection::Resume => controls.press(&GlobalAction::Cancel),
        Selection::Restart => {
            // just a quick loading screen, no bg
            cmd.insert_resource(LoadingScreenSettings {
                atlas: None,
                fade_loading_screen_in:
                    ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN,
                fade_loading_screen_out: ON_RESTART_FADE_LOADING_SCREEN_OUT,
                ..default()
            });
            next_loading_state.set(common_loading_screen::start_state());

            *transition = GlobalGameStateTransition::RestartMeditation;
            next_state.set(GlobalGameState::QuittingMeditation);
        }
        Selection::Quit => {
            // just a quick loading screen, no bg
            cmd.insert_resource(LoadingScreenSettings {
                atlas: None,
                fade_loading_screen_in:
                    ON_RESTART_OR_EXIT_FADE_LOADING_SCREEN_IN,
                fade_loading_screen_out: ON_EXIT_FADE_LOADING_SCREEN_OUT,
                ..default()
            });
            next_loading_state.set(common_loading_screen::start_state());

            *transition =
                GlobalGameStateTransition::MeditationToBuilding1PlayerFloor;
            next_state.set(GlobalGameState::QuittingMeditation);
        }
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
            ..default()
        },
        UiImage::new(asset_server.load(assets::MENU_BOX)),
    ));

    ui_root.spawn((
        SelectionMarker,
        NodeBundle {
            style: Style {
                width: Val::Px(36.0 * PIXEL_ZOOM as f32),
                height: Val::Px(36.0 * PIXEL_ZOOM as f32),
                top: Val::Px(SELECTION_MARKER_TOP_OFFSET_PX),
                left: SELECTION_MARKER_LEFT_OFFSET,
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        UiImage::new(asset_server.load(assets::FACE_ON_CONTINUE)),
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
