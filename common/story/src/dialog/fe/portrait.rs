//! When a dialog is spawned, it's already loaded as it should look and does not
//! require any additional actions.
//!
//! You first want to obtain the dialog be [`Dialog`] and then spawn the
//! dialog UI with [`Dialog::spawn_with_portrait_ui`].

use std::{collections::BTreeMap, time::Duration};

use bevy::{
    math::vec2, prelude::*, render::view::RenderLayers, text::TextLayoutInfo,
    utils::Instant,
};
use common_action::{ActionState, GlobalAction};
use common_store::GlobalStore;
use common_visuals::camera::render_layer;
use itertools::Itertools;

use crate::{
    dialog::{
        AdvanceOutcome, BranchStatus, Branching, Dialog, NodeKind, NodeName,
    },
    Character,
};

const DIALOG_LEFT: Val = Val::Vw(10.0);
const FONT_SIZE: f32 = 21.0;
const CHOICE_FONT_SIZE: f32 = 17.0;
const FONT: &str = common_assets::fonts::PENCIL1;
const TEXT_BOUNDS: Vec2 = vec2(250.0, 120.0);
const CHOICE_TEXT_BOUNDS: Vec2 = vec2(250.0, 50.0);
const MIN_TEXT_FRAME_TIME: Duration = Duration::from_millis(200);

/// Will be true if in a dialog that takes away player control.
pub fn in_portrait_dialog() -> impl FnMut(Option<Res<PortraitDialog>>) -> bool {
    move |dialog| dialog.is_some()
}

/// If inserted, then the game is in the dialog UI.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct PortraitDialog {
    /// We force a small delay between frames to prevent the player from
    /// skipping through the dialog way too fast.
    /// The tiny delay lets the brain to at least get the gist of what's
    /// being said.
    last_frame_shown_at: Instant,
    last_rendered_node: Option<NodeName>,
}

impl Dialog {
    /// Spawns the dialog UI and inserts all necessary resources.
    pub fn spawn_with_portrait_ui(
        self,
        cmd: &mut Commands,
        asset_server: &AssetServer,
    ) {
        let speaker = self.current_node_info().who;
        PortraitDialog::default().spawn(cmd, asset_server, speaker);

        self.spawn(cmd);
    }
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
enum PortraitDialogState {
    /// This is the initial state.
    /// When in this state, no systems run.
    #[default]
    NotInDialog,
    /// When in this state, we run system [`await_portrait_async_ops`] every
    /// tick.
    /// When the dialog is ready, it will yield control to player by
    /// transitioning to [`PortraitDialogState::PlayerControl`].
    WaitingForAsync,
    /// When in this state, we run system [`player_wishes_to_continue`] when
    /// the player presses the interact key.
    PlayerControl,
}

/// Marks the dialog camera.
#[derive(Component)]
struct DialogCamera;

/// The root entity of the dialog UI.
#[derive(Component)]
struct DialogUiRoot;

/// A child of the root entity that contains the text.
#[derive(Component)]
struct DialogText;

/// A child of the root entity that contains the portrait image.
#[derive(Component)]
struct DialogPortrait;

/// Entities that render choices in dialogs.
/// When advancing the dialog, the selected choice will be used to determine
/// the next sequence.
#[derive(Component, Clone, Debug, Reflect)]
struct DialogChoice {
    of: NodeName,
    /// Starts at 0.
    order: usize,
    /// Is selected either if it's the first choice or if the player changed
    /// selection to this.
    is_selected: bool,
}

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PortraitDialogState>();

        app.add_systems(
            First,
            await_portrait_async_ops
                .run_if(in_state(PortraitDialogState::WaitingForAsync)),
        )
        .add_systems(
            Last,
            player_wishes_to_continue
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(common_action::interaction_just_pressed()),
        )
        .add_systems(
            Update,
            change_selection
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(common_action::move_action_just_pressed()),
        )
        .add_systems(
            Update,
            cancel
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(common_action::cancel_just_pressed()),
        );

        #[cfg(feature = "devtools")]
        {
            app.register_type::<PortraitDialogState>()
                .register_type::<PortraitDialog>()
                .register_type::<DialogChoice>();

            use bevy_inspector_egui::quick::StateInspectorPlugin;
            app.add_plugins(
                StateInspectorPlugin::<PortraitDialogState>::default(),
            );
        }
    }
}

/// Call this to load the next step in the dialog.
/// A step could be some text, or a player choice, etc.
///
/// This should run only in state [`PortraitDialogState::PlayerControl`] and
/// if the player hit the interact button.
#[allow(clippy::too_many_arguments)]
fn player_wishes_to_continue(
    mut cmd: Commands,
    mut next_dialog_state: ResMut<NextState<PortraitDialogState>>,
    mut dialog_fe: ResMut<PortraitDialog>,
    mut dialog_be: ResMut<Dialog>,
    asset_server: Res<AssetServer>,
    store: Res<GlobalStore>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    camera: Query<Entity, With<DialogCamera>>,
    root: Query<Entity, With<DialogUiRoot>>,
    mut text: Query<(&mut Text, &TextLayoutInfo), With<DialogText>>,
    mut portrait: Query<&mut UiImage, With<DialogPortrait>>,
    choices: Query<(Entity, &DialogChoice)>,
) {
    if dialog_fe.last_frame_shown_at.elapsed() < MIN_TEXT_FRAME_TIME {
        return;
    }

    let (mut text, layout) = text.single_mut();

    // If we rendered some glyphs, we need to check whether we rendered all
    // of the text.
    // Empty rendered glyphs means there was no text to render.
    let rendered_glyphs_count = layout.glyphs.len();
    if rendered_glyphs_count > 0 {
        // Since white spaces are not rendered by instead used to calculate the
        // positions of the other glyphs, we need to skip those when calculating
        // what is the portion of the text that has NOT been rendered yet.
        let next_char_info = text.sections[0]
            .value
            .chars()
            .enumerate()
            .filter(|(_, c)| !c.is_whitespace())
            .nth(rendered_glyphs_count); // the next char won't be a white space

        if let Some((next_char_index, next_char)) = next_char_info {
            if let Some(remaining_text) =
                text.sections[0].value.get(next_char_index..)
            {
                debug_assert_eq!(
                    remaining_text.chars().next(),
                    Some(next_char)
                );

                // if there's more text to render, set the remaining text to
                // the text component value and wait for the player to continue
                text.sections[0].value = remaining_text.to_string();
                dialog_fe.last_frame_shown_at = Instant::now();

                trace!("Rendering remaining text");
                return;
            }
        }
    }

    let next_state = advance_dialog(
        &mut cmd,
        &store,
        &mut dialog_be,
        &mut dialog_fe,
        &asset_server,
        &mut controls,
        camera.single(),
        root.single(),
        &mut text,
        &mut portrait.single_mut(),
        choices,
    );
    match next_state {
        PortraitDialogState::PlayerControl => {}
        next_state => {
            trace!("Player advancing dialog to {next_state:?}");
            next_dialog_state.set(next_state);
        }
    };
}

/// Sometimes dialog nodes require some async operations to be completed before
/// the dialog can continue.
/// These are e.g. animations etc.
///
/// This system will wait until the async operations are done and then continue.
///
/// Run this only if in state [`PortraitDialogState::WaitingForAsync`].
#[allow(clippy::too_many_arguments)]
fn await_portrait_async_ops(
    mut cmd: Commands,
    mut next_dialog_state: ResMut<NextState<PortraitDialogState>>,
    mut dialog_fe: ResMut<PortraitDialog>,
    mut dialog_be: ResMut<Dialog>,
    asset_server: Res<AssetServer>,
    store: Res<GlobalStore>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    camera: Query<Entity, With<DialogCamera>>,
    root: Query<Entity, With<DialogUiRoot>>,
    mut text: Query<&mut Text, With<DialogText>>,
    mut portrait: Query<&mut UiImage, With<DialogPortrait>>,
    choices: Query<(Entity, &DialogChoice)>,
) {
    let next_state = advance_dialog(
        &mut cmd,
        &store,
        &mut dialog_be,
        &mut dialog_fe,
        &asset_server,
        &mut controls,
        camera.single(),
        root.single(),
        &mut text.single_mut(),
        &mut portrait.single_mut(),
        choices,
    );
    match next_state {
        PortraitDialogState::WaitingForAsync => {}
        next_state => {
            trace!("Await advancing dialog to {next_state:?}");
            next_dialog_state.set(next_state);
        }
    };
}

#[allow(clippy::too_many_arguments)]
fn advance_dialog(
    cmd: &mut Commands,
    store: &GlobalStore,
    dialog_be: &mut Dialog,
    dialog_fe: &mut PortraitDialog,
    asset_server: &AssetServer,
    controls: &mut ActionState<GlobalAction>,
    camera: Entity,
    root: Entity,
    text: &mut Text,
    portrait: &mut UiImage,
    choices: Query<(Entity, &DialogChoice)>,
) -> PortraitDialogState {
    loop {
        let last_matches_be_current = dialog_fe
            .last_rendered_node
            .as_ref()
            .is_some_and(|n| n == dialog_be.current_node());

        if !last_matches_be_current {
            dialog_fe.last_rendered_node =
                Some(dialog_be.current_node().clone());

            let node = dialog_be.current_node_info();
            if let NodeKind::Vocative { line } = &node.kind {
                trace!("Rendering vocative {:?}: '{line:?}'", node.who);

                text.sections[0].value = line.clone();
                portrait.texture =
                    asset_server.load(node.who.portrait_asset_path());

                // let the player read the text
                break PortraitDialogState::PlayerControl;
            }
        }

        match dialog_be.advance(cmd, store) {
            AdvanceOutcome::Transition => {
                // run `advance` again
            }
            AdvanceOutcome::ScheduledDespawn => {
                trace!("Despawning portrait dialog FE");

                dialog_fe.despawn(cmd, controls, camera, root);

                // the dialog is over
                break PortraitDialogState::NotInDialog;
            }
            AdvanceOutcome::AwaitingPlayerChoice => {
                if choices.is_empty() {
                    trace!("Displaying player choices");

                    let Branching::Choice(branches) = &dialog_be.branching
                    else {
                        panic!(
                            "AwaitingPlayerChoice implies Branching::Choice"
                        );
                    };

                    show_player_choices(
                        cmd,
                        asset_server,
                        dialog_be,
                        root,
                        branches,
                    );

                    // let the player make a choice
                    break PortraitDialogState::PlayerControl;
                } else {
                    // choices were already displayed so this time the player
                    // confirmed their choice

                    let chosen_node_name = choices
                        .iter()
                        .find(|(_, choice)| choice.is_selected)
                        .map(|(_, choice)| choice.of.clone())
                        .expect("There should be a selected choice");

                    choices.iter().for_each(|(entity, _)| {
                        cmd.entity(entity).despawn_recursive()
                    });

                    trace!("Player chose {chosen_node_name:?}");
                    dialog_be.transition_to(cmd, store, chosen_node_name);

                    // run `advance` again
                }
            }
            AdvanceOutcome::WaitUntilNextTick => {
                // guards are still doing their thing, try again later
                break PortraitDialogState::WaitingForAsync;
            }
        }
    }
}

/// Cancel the dialog.
/// For example, if the player presses the cancel key.
fn cancel(
    mut cmd: Commands,
    mut next_dialog_state: ResMut<NextState<PortraitDialogState>>,
    mut dialog_be: ResMut<Dialog>,
    mut dialog_fe: ResMut<PortraitDialog>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    root: Query<Entity, With<DialogUiRoot>>,
    camera: Query<Entity, With<DialogCamera>>,
) {
    trace!("Cancelling dialog");
    dialog_be.despawn(&mut cmd);
    dialog_fe.despawn(&mut cmd, &mut controls, camera.single(), root.single());

    next_dialog_state.set(PortraitDialogState::NotInDialog);
}

/// Run if pressed some movement key.
/// If there are choices, then the selection will be changed if the movement
/// was either up or down.
fn change_selection(
    controls: Res<ActionState<GlobalAction>>,
    asset_server: Res<AssetServer>,

    mut choices: Query<(&Children, &mut DialogChoice, &mut UiImage)>,
    mut texts: Query<&mut Text>,
) {
    if choices.is_empty() {
        return;
    }

    let up = controls.pressed(&GlobalAction::MoveUp)
        || controls.pressed(&GlobalAction::MoveUpLeft)
        || controls.pressed(&GlobalAction::MoveUpRight);

    let down = controls.pressed(&GlobalAction::MoveDown)
        || controls.pressed(&GlobalAction::MoveDownLeft)
        || controls.pressed(&GlobalAction::MoveDownRight);

    if !up && !down {
        return;
    }

    let (active_order, mut choice_map) = {
        let mut active = 0;
        let choice_map: BTreeMap<_, _> = choices
            .iter_mut()
            .map(|(children, choice, image)| {
                if choice.is_selected {
                    active = choice.order;
                }
                (choice.order, (children, choice, image))
            })
            .collect();

        (active, choice_map)
    };

    let new_active_order = if up {
        // previous
        if active_order == 0 {
            choice_map.len() - 1
        } else {
            active_order - 1
        }
    } else {
        // next
        if active_order == choice_map.len() - 1 {
            0
        } else {
            active_order + 1
        }
    };

    // set the active order's font to WHITE and the image to highlighted option
    if let Some((children, new_choice, image)) =
        choice_map.get_mut(&new_active_order)
    {
        new_choice.is_selected = true;

        image.texture =
            asset_server.load(common_assets::dialog::DIALOG_CHOICE_HIGHLIGHTED);

        debug_assert_eq!(1, children.len());
        if let Ok(mut text) = texts.get_mut(children[0]) {
            text.sections[0].style.color = Color::WHITE;
        } else {
            error!("Cannot find text for choice with order {new_active_order}");
        }
    } else {
        error!("Cannot find choice with order {new_active_order}");
    }

    // now set the old active order's font to BLACK and the image to normal
    // option
    if let Some((children, old_choice, image)) =
        choice_map.get_mut(&active_order)
    {
        old_choice.is_selected = false;

        image.texture = asset_server.load(common_assets::dialog::DIALOG_CHOICE);

        debug_assert_eq!(1, children.len());
        if let Ok(mut text) = texts.get_mut(children[0]) {
            text.sections[0].style.color = Color::BLACK;
        } else {
            error!("Cannot find text for choice with order {active_order}");
        }
    } else {
        error!("Cannot find choice with order {active_order}");
    }
}

impl PortraitDialog {
    fn despawn(
        &mut self,
        cmd: &mut Commands,
        controls: &mut ActionState<GlobalAction>,
        camera: Entity,
        root: Entity,
    ) {
        cmd.entity(camera).despawn();
        cmd.entity(root).despawn_recursive();
        cmd.remove_resource::<PortraitDialog>();

        controls.consume_all();
    }

    /// Spawns [`PortraitDialog`] resource and all the necessary UI components.
    fn spawn(
        self,
        cmd: &mut Commands,
        asset_server: &AssetServer,
        speaker: Character,
    ) {
        cmd.insert_resource(self);
        // this transitions into the first dialog node
        cmd.add(|w: &mut World| {
            w.get_resource_mut::<NextState<PortraitDialogState>>()
                .unwrap()
                .set(PortraitDialogState::WaitingForAsync)
        });

        // Spawns the dialog camera which has a high order and only renders the
        // dialog entities.
        let camera = cmd
            .spawn((
                Name::from("Portrait dialog camera"),
                DialogCamera,
                RenderLayers::layer(render_layer::DIALOG),
                Camera2dBundle {
                    camera: Camera {
                        hdr: true,
                        order: common_visuals::camera::order::DIALOG,
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        let text = Text::from_section(
            "",
            TextStyle {
                font: asset_server.load(FONT),
                font_size: FONT_SIZE,
                color: Color::BLACK,
            },
        );

        let root = cmd
            .spawn((
                Name::new("Portrait dialog root"),
                DialogUiRoot,
                TargetCamera(camera),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        left: DIALOG_LEFT,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        cmd.entity(root).with_children(|parent| {
            parent
                .spawn((
                    Name::new("Bubble"),
                    RenderLayers::layer(render_layer::DIALOG),
                    UiImage::new(
                        asset_server.load(common_assets::dialog::DIALOG_BUBBLE),
                    ),
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            width: Val::Px(400.0),
                            height: Val::Px(414.0),
                            bottom: Val::Px(290.0),
                            justify_content: JustifyContent::Center,
                            justify_items: JustifyItems::Center,
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        // a `NodeBundle` is transparent by default, so to see
                        // the image we have to its
                        // color to `WHITE`
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        DialogText,
                        Name::new("Dialog text"),
                        RenderLayers::layer(render_layer::DIALOG),
                        TextBundle {
                            text,
                            style: Style {
                                width: Val::Px(TEXT_BOUNDS.x),
                                height: Val::Px(TEXT_BOUNDS.y),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                });

            parent.spawn((
                DialogPortrait,
                Name::new("Portrait"),
                RenderLayers::layer(render_layer::DIALOG),
                NodeBundle {
                    style: Style {
                        width: Val::Px(common_assets::portraits::SIZE_PX.x),
                        height: Val::Px(common_assets::portraits::SIZE_PX.y),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                    // a `NodeBundle` is transparent by default, so to see the
                    // image we have to its color to `WHITE`
                    background_color: Color::WHITE.into(),
                    ..default()
                },
                UiImage::new(asset_server.load(speaker.portrait_asset_path())),
            ));
        });
    }
}

/// Renders UI for player choices.
fn show_player_choices(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    dialog_be: &Dialog,
    root: Entity,
    branches: &[BranchStatus],
) {
    let node = dialog_be.current_node_info();
    let between = branches
        .iter()
        .enumerate()
        .filter_map(|(branch_index, status)| match status {
            BranchStatus::OfferAsChoice(text) => {
                let node_name = node.next[branch_index].clone();

                Some((node_name, text))
            }
            BranchStatus::Stop => None,
            BranchStatus::Pending => unreachable!(),
        })
        .collect_vec();

    let total = between.len();
    debug_assert_ne!(0, total);

    let transform_manager = node.who.choice_transform_manager(total);

    for (order, (node_name, choice_text)) in between.iter().enumerate() {
        let choice = spawn_choice(
            cmd,
            asset_server,
            &transform_manager,
            order,
            node_name,
            choice_text,
        );
        cmd.entity(root).add_child(choice);
    }
}

fn spawn_choice(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    transform_manager: &ChoicePositionManager,
    order: usize,
    node_name: &NodeName,
    choice_text: &str,
) -> Entity {
    let (asset, color) = if order == 0 {
        (
            common_assets::dialog::DIALOG_CHOICE_HIGHLIGHTED,
            Color::WHITE,
        )
    } else {
        (common_assets::dialog::DIALOG_CHOICE, Color::BLACK)
    };

    let choice = DialogChoice {
        of: node_name.clone(),
        order,
        is_selected: order == 0,
    };

    let Vec2 { x: left, y: bottom } = transform_manager.get(order);

    cmd.spawn((
        Name::new(format!("Choice {order}: {node_name:?}")),
        RenderLayers::layer(render_layer::DIALOG),
        choice,
        UiImage::new(asset_server.load(asset)),
        NodeBundle {
            z_index: ZIndex::Local(1 + order as i32),
            style: Style {
                width: Val::Px(350.0),
                height: Val::Px(92.0),
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                bottom: Val::Px(bottom),
                justify_content: JustifyContent::Center,
                ..default()
            },
            // a `NodeBundle` is transparent by default, so to see the
            // image we have to its color to `WHITE`
            background_color: Color::WHITE.into(),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("Dialog choice text"),
            RenderLayers::layer(render_layer::DIALOG),
            TextBundle {
                text: Text::from_section(
                    choice_text,
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: CHOICE_FONT_SIZE,
                        color,
                    },
                ),
                style: Style {
                    max_width: Val::Px(CHOICE_TEXT_BOUNDS.x),
                    max_height: Val::Px(CHOICE_TEXT_BOUNDS.y),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                ..default()
            },
        ));
    })
    .id()
}

pub(super) struct ChoicePositionManager {
    positions: Vec<Vec2>,
}

impl ChoicePositionManager {
    fn get(&self, index: usize) -> Vec2 {
        debug_assert!(
            index < self.positions.len(),
            "Cannot get position index for index {index}"
        );
        self.positions[index]
    }
}

impl Character {
    fn choice_transform_manager(
        self,
        total_choices: usize,
    ) -> ChoicePositionManager {
        #[allow(clippy::match_single_binding)]
        let positions = match total_choices {
            1 => match self {
                _ => vec![vec2(240.0, 75.0)],
            },
            2 => match self {
                _ => vec![vec2(240.0, 140.0), vec2(260.0, 75.0)],
            },
            3 => match self {
                _ => vec![
                    vec2(227.0, 145.0),
                    vec2(240.0, 75.0),
                    vec2(260.0, 5.0),
                ],
            },
            total => todo!("Cannot handle {total} choices"),
        };
        debug_assert_eq!(total_choices, positions.len());

        ChoicePositionManager { positions }
    }
}

impl Default for PortraitDialog {
    fn default() -> Self {
        Self {
            last_frame_shown_at: Instant::now(),
            last_rendered_node: default(),
        }
    }
}
