//! When a dialog is spawned, it's already loaded as it should look and does not
//! require any additional actions.
//!
//! You first want to obtain the dialog BE [`Dialog`] and then spawn the
//! dialog FE with [`StartDialogWhenLoaded::portrait`].

use std::{collections::BTreeMap, time::Duration};

use bevy::{
    prelude::*, render::view::RenderLayers, text::TextLayoutInfo,
    utils::Instant,
};
use common_action::{ActionState, GlobalAction};
use common_assets::ui::DIALOG_BOX;
use common_store::GlobalStore;
use common_visuals::camera::{render_layer, PIXEL_ZOOM};

use super::DialogFrontend;
use crate::{
    dialog::{
        AdvanceOutcome, Branching, Dialog, NodeKind, NodeName,
        StartDialogWhenLoaded,
    },
    Character,
};

const FONT_SIZE: f32 = 21.0;
const CHOICE_FONT_SIZE: f32 = 17.0;
const FONT: &str = common_assets::fonts::PENCIL1;
/// Dark orange
const CHOICE_HIGHLIGHT_COLOR: Color = Color::rgb(0.789, 0.455, 0.007);
const MIN_TEXT_FRAME_TIME: Duration = Duration::from_millis(200);

/// Will be true if in a dialog that takes away player control.
pub fn in_portrait_dialog() -> impl FnMut(
    Option<Res<PortraitDialog>>,
    Option<Res<StartDialogWhenLoaded>>,
) -> bool {
    move |dialog, loading| {
        dialog.is_some()
            || loading.is_some_and(|loading| {
                matches!(loading.fe, DialogFrontend::Portrait)
            })
    }
}

impl Dialog {
    /// Spawns the dialog UI and inserts all necessary resources.
    pub(crate) fn spawn_with_portrait_fe(
        self,
        cmd: &mut Commands,
        asset_server: &AssetServer,
    ) {
        let speaker = self.current_node_info().who;
        PortraitDialog::spawn(cmd, asset_server, speaker);

        self.spawn(cmd);
    }
}

/// If inserted, then the game is in the dialog UI.
#[derive(Resource)]
pub struct PortraitDialog {
    /// We force a small delay between frames to prevent the player from
    /// skipping through the dialog way too fast.
    /// The tiny delay lets the brain to at least get the gist of what's
    /// being said.
    last_frame_shown_at: Instant,
    last_rendered_node: Option<NodeName>,
    /// Holds the whole dialog UI.
    root: Entity,
    /// The dialog camera entity.
    camera: Entity,
    /// Contains the list of dialog choices (those are going to be the
    /// children).
    /// The node is always present, but can have no children.
    choices_box: Entity,
}

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
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
    /// Sometimes the vocative line to render is short.
    /// If there come choices after the vocative line, don't require the player
    /// to press the interact key to see the choices.
    /// Show them straight away.
    ///
    /// This state transitions to [`PortraitDialogState::PlayerControl`]
    /// unless any choice still loading.
    /// It renders the choices only if
    /// - no more text to render (short line)
    /// - there are choices to render
    /// - no choices have been rendered yet
    RenderChoicesIfNoMoreTextToRender,
}

/// Emitted when the player clicks the interact key or uses the mouse to confirm
/// their selection.
#[derive(Event)]
struct PlayerAdvancesDialogEvent;

/// Marks the dialog camera.
#[derive(Component)]
struct DialogCamera;
/// The root entity of the dialog UI.
#[derive(Component)]
struct DialogUiRoot;
/// A child of the dialog box entity that contains the text.
#[derive(Component)]
struct DialogText;
/// A child of the dialog box entity that contains list of choices if any.
#[derive(Component)]
struct DialogChoicesBox;
/// A child of the root entity that contains the portrait image.
#[derive(Component)]
struct DialogPortrait;

/// Entities that render choices in dialogs.
/// When advancing the dialog, the selected choice will be used to determine
/// the next sequence.
#[derive(Component, Clone, Debug)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
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
        app.init_state::<PortraitDialogState>()
            .add_event::<PlayerAdvancesDialogEvent>();

        app.add_systems(
            First,
            await_portrait_async_ops
                .run_if(in_state(PortraitDialogState::WaitingForAsync)),
        )
        .add_systems(
            Last,
            player_wishes_to_continue
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(on_event::<PlayerAdvancesDialogEvent>()),
        )
        .add_systems(
            Update,
            render_choices_if_no_more_text_to_render.run_if(in_state(
                PortraitDialogState::RenderChoicesIfNoMoreTextToRender,
            )),
        )
        .add_systems(
            Update,
            confirm_selection
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(common_action::interaction_just_pressed()),
        )
        .add_systems(
            Update,
            change_selection_with_arrows
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(common_action::move_action_just_pressed()),
        )
        .add_systems(
            Update,
            change_selection_with_numbers
                .run_if(in_state(PortraitDialogState::PlayerControl))
                .run_if(common_action::numeric_key_pressed()),
        )
        .add_systems(
            Update,
            handle_mouse_input
                .run_if(in_state(PortraitDialogState::PlayerControl)),
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
fn player_wishes_to_continue(
    mut cmd: Commands,
    mut next_dialog_state: ResMut<NextState<PortraitDialogState>>,
    mut dialog_fe: ResMut<PortraitDialog>,
    mut dialog_be: ResMut<Dialog>,
    asset_server: Res<AssetServer>,
    store: Res<GlobalStore>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    mut text: Query<(&mut Text, &TextLayoutInfo), With<DialogText>>,
    mut portrait: Query<&mut UiImage, With<DialogPortrait>>,
    choices: Query<(Entity, &DialogChoice)>,
) {
    if dialog_fe.last_frame_shown_at.elapsed() < MIN_TEXT_FRAME_TIME {
        return;
    }

    let (mut text, layout) = text.single_mut();

    if let Some(remaining_text) = get_more_text_to_render(&text, layout) {
        trace!("Rendering remaining text");

        // if there's more text to render, set the remaining text to
        // the text component value and wait for the player to continue
        text.sections[0].value = remaining_text.to_string();
        dialog_fe.last_frame_shown_at = Instant::now();

        return;
    }

    let next_state = advance_dialog(
        &mut cmd,
        &store,
        &mut dialog_be,
        &mut dialog_fe,
        &asset_server,
        &mut controls,
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
fn await_portrait_async_ops(
    mut cmd: Commands,
    mut next_dialog_state: ResMut<NextState<PortraitDialogState>>,
    mut dialog_fe: ResMut<PortraitDialog>,
    mut dialog_be: ResMut<Dialog>,
    asset_server: Res<AssetServer>,
    store: Res<GlobalStore>,
    mut controls: ResMut<ActionState<GlobalAction>>,

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

fn render_choices_if_no_more_text_to_render(
    mut cmd: Commands,
    mut next_dialog_state: ResMut<NextState<PortraitDialogState>>,
    dialog_be: ResMut<Dialog>,
    dialog_fe: Res<PortraitDialog>,
    asset_server: Res<AssetServer>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    text: Query<(&Text, &TextLayoutInfo), With<DialogText>>,
    choices: Query<(Entity, &DialogChoice)>,
) {
    if !choices.is_empty() {
        warn!(
            "render_choices_if_no_more_text_to_render called but \
            there are already choices displayed"
        );
        next_dialog_state.set(PortraitDialogState::PlayerControl);
        return;
    }

    let (text, layout) = text.single();
    if get_more_text_to_render(text, layout).is_some() {
        next_dialog_state.set(PortraitDialogState::PlayerControl);
        return;
    }

    match dialog_be.get_choices() {
        None => {
            next_dialog_state.set(PortraitDialogState::PlayerControl);
        }
        Some(Err(_)) => {
            // choices are still loading, try again next tick
        }
        Some(Ok(branches)) => {
            show_player_choices(&mut cmd, &asset_server, &dialog_fe, &branches);

            controls.consume_all();
            next_dialog_state.set(PortraitDialogState::PlayerControl);
        }
    }
}

fn get_more_text_to_render(
    text: &Text,
    layout: &TextLayoutInfo,
) -> Option<String> {
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

                return Some(remaining_text.to_string());
            }
        }
    }

    None
}

fn advance_dialog(
    cmd: &mut Commands,
    store: &GlobalStore,
    dialog_be: &mut Dialog,
    dialog_fe: &mut PortraitDialog,
    asset_server: &AssetServer,
    controls: &mut ActionState<GlobalAction>,
    text: &mut Text,
    portrait: &mut UiImage,
    choices: Query<(Entity, &DialogChoice)>,
) -> PortraitDialogState {
    loop {
        let last_matches_be_current = dialog_fe
            .last_rendered_node
            .as_ref()
            .is_some_and(|n| n == &dialog_be.current_node);

        if !last_matches_be_current {
            dialog_fe.last_rendered_node = Some(dialog_be.current_node.clone());

            let node = dialog_be.current_node_info();
            if let NodeKind::Vocative { line } = &node.kind {
                trace!("Rendering vocative {:?}: '{line:?}'", node.who);

                text.sections[0].value.clone_from(line);
                portrait.texture =
                    asset_server.load(node.who.portrait_asset_path());

                // let the player read the text and perhaps show the choices
                // if the text does not need the player to press the interact
                // to scroll through it
                break PortraitDialogState::RenderChoicesIfNoMoreTextToRender;
            }
        }

        match dialog_be.advance(cmd, store) {
            AdvanceOutcome::Transition => {
                // run `advance` again
            }
            AdvanceOutcome::ScheduledDespawn => {
                trace!("Despawning portrait dialog FE");

                dialog_fe.despawn(cmd, controls);

                // the dialog is over
                break PortraitDialogState::NotInDialog;
            }
            AdvanceOutcome::AwaitingPlayerChoice => {
                if choices.is_empty() {
                    trace!("Displaying player choices");

                    show_player_choices(
                        cmd,
                        asset_server,
                        dialog_fe,
                        &dialog_be
                            .get_choices()
                            .expect("choices present on AwaitingPlayerChoice")
                            .expect("choices loaded on AwaitingPlayerChoice"),
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

                    if let Branching::Single(next_node_name) =
                        &dialog_be.branching
                    {
                        // If the next node has no choices, run transition_to.
                        // This is because all t he text has been rendered as
                        // option, so no need to repeat it.

                        let next_node =
                            dialog_be.graph.nodes.get(next_node_name).unwrap();
                        if matches!(
                            &next_node.kind,
                            NodeKind::Vocative { .. } | NodeKind::Blank
                        ) {
                            portrait.texture = asset_server
                                .load(next_node.who.portrait_asset_path());
                            dialog_be.transition_to(
                                cmd,
                                store,
                                next_node_name.clone(),
                            );
                        }
                    }

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
) {
    trace!("Cancelling dialog");
    dialog_be.despawn(&mut cmd);
    dialog_fe.despawn(&mut cmd, &mut controls);

    next_dialog_state.set(PortraitDialogState::NotInDialog);
}

/// The player can hover over a choice to select it and/or click to confirm.
fn handle_mouse_input(
    mut events: EventWriter<PlayerAdvancesDialogEvent>,

    mut choices: Query<(
        &Interaction,
        &Children,
        &mut DialogChoice,
        &mut BackgroundColor,
    )>,
    mut texts: Query<&mut Text>,
) {
    let any_pressed = choices.iter().any(|(interaction, ..)| {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    });

    if !any_pressed {
        return;
    }

    // if true then we fire a confirm event
    let mut was_pressed = false;
    // there's always at least one active choice
    let mut old_active_order = 0;
    // there's always going to be a new active choice because we just
    // checked that the interaction is either hovered or pressed
    let mut new_active_order = 0;
    let mut choice_map: BTreeMap<_, _> = {
        choices
            .iter_mut()
            .map(|(interaction, children, choice, bg_color)| {
                if choice.is_selected {
                    old_active_order = choice.order;
                }

                match interaction {
                    Interaction::Pressed => {
                        new_active_order = choice.order;
                        was_pressed = true;
                    }
                    Interaction::Hovered => {
                        new_active_order = choice.order;
                    }
                    _ => {}
                };

                (choice.order, (children, choice, bg_color))
            })
            .collect()
    };

    if was_pressed {
        events.send(PlayerAdvancesDialogEvent);
    }

    if old_active_order == new_active_order {
        return;
    }

    // highlight new choice
    set_choice_highlight(true, new_active_order, &mut choice_map, &mut texts);
    // unhighlight old choice
    set_choice_highlight(false, old_active_order, &mut choice_map, &mut texts);
}

/// Runs when the user presses the interact key.
fn confirm_selection(mut events: EventWriter<PlayerAdvancesDialogEvent>) {
    events.send(PlayerAdvancesDialogEvent);
}

/// Run if pressed some movement key.
/// If there are choices, then the selection will be changed if the movement
/// was either up or down.
fn change_selection_with_arrows(
    controls: Res<ActionState<GlobalAction>>,

    mut choices: Query<(&Children, &mut DialogChoice, &mut BackgroundColor)>,
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

    let (old_active_order, mut choice_map) = {
        let mut active = 0;
        let choice_map: BTreeMap<_, _> = choices
            .iter_mut()
            .map(|(children, choice, bg_color)| {
                if choice.is_selected {
                    active = choice.order;
                }
                (choice.order, (children, choice, bg_color))
            })
            .collect();

        (active, choice_map)
    };

    let new_active_order = if up {
        // previous
        if old_active_order == 0 {
            choice_map.len() - 1
        } else {
            old_active_order - 1
        }
    } else {
        // next
        if old_active_order == choice_map.len() - 1 {
            0
        } else {
            old_active_order + 1
        }
    };

    // highlight new choice
    set_choice_highlight(true, new_active_order, &mut choice_map, &mut texts);
    // unhighlight old choice
    set_choice_highlight(false, old_active_order, &mut choice_map, &mut texts);
}

fn change_selection_with_numbers(
    controls: Res<ActionState<GlobalAction>>,

    mut choices: Query<(&Children, &mut DialogChoice, &mut BackgroundColor)>,
    mut texts: Query<&mut Text>,
) {
    if choices.is_empty() {
        return;
    }

    let mut new_active_order = None;
    // get all numerical actions (ordered) from 1 to 9
    for (i, action) in GlobalAction::numerical().into_iter().enumerate().skip(1)
    {
        if controls.pressed(&action) {
            new_active_order = Some(i - 1);
            break;
        }
    }

    if let Some(new_active_order) = new_active_order {
        let (old_active_order, mut choice_map) = {
            let mut active = 0;
            let choice_map: BTreeMap<_, _> = choices
                .iter_mut()
                .map(|(children, choice, bg_color)| {
                    if choice.is_selected {
                        active = choice.order;
                    }
                    (choice.order, (children, choice, bg_color))
                })
                .collect();

            (active, choice_map)
        };

        if choice_map.len() > new_active_order
            && old_active_order != new_active_order
        {
            // highlight new choice
            set_choice_highlight(
                true,
                new_active_order,
                &mut choice_map,
                &mut texts,
            );
            // unhighlight old choice
            set_choice_highlight(
                false,
                old_active_order,
                &mut choice_map,
                &mut texts,
            );
        }
    }
}

/// Either highlights or unhighlights a choice.
fn set_choice_highlight(
    highlighted: bool,
    order: usize,
    choice_map: &mut BTreeMap<
        usize,
        (&Children, Mut<DialogChoice>, Mut<BackgroundColor>),
    >,
    texts: &mut Query<&mut Text>,
) {
    // highlight new choice
    if let Some((children, new_choice, bg_color)) = choice_map.get_mut(&order) {
        new_choice.is_selected = highlighted;

        let (new_text_color, new_bg_color) = if highlighted {
            (Color::BLACK, CHOICE_HIGHLIGHT_COLOR)
        } else {
            (Color::WHITE, Color::NONE)
        };

        debug_assert_eq!(1, children.len());
        if let Ok(mut text) = texts.get_mut(children[0]) {
            text.sections[0].style.color = new_text_color;
            **bg_color = new_bg_color.into();
        } else {
            error!("Cannot find text for choice with order {order}");
        }
    } else {
        error!("Cannot find choice with order {order}");
    }
}

impl PortraitDialog {
    fn despawn(
        &mut self,
        cmd: &mut Commands,
        controls: &mut ActionState<GlobalAction>,
    ) {
        cmd.entity(self.camera).despawn_recursive();
        cmd.entity(self.root).despawn_recursive();
        cmd.remove_resource::<PortraitDialog>();

        controls.consume_all();
    }

    /// Spawns [`PortraitDialog`] resource and all the necessary UI components.
    fn spawn(
        cmd: &mut Commands,
        asset_server: &AssetServer,
        speaker: Character,
    ) {
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
                color: Color::WHITE,
            },
        );

        let root = cmd
            .spawn((
                Name::new("Portrait dialog root"),
                DialogUiRoot,
                TargetCamera(camera),
                NodeBundle {
                    // centers the content
                    style: Style {
                        width: Val::Vw(100.0),
                        bottom: Val::Px(0.0),
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::RowReverse,

                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        // written into with the with_children command
        let mut choices_box = Entity::PLACEHOLDER;
        cmd.entity(root).with_children(|parent| {
            parent
                .spawn((
                    Name::new("Dialog Box"),
                    RenderLayers::layer(render_layer::DIALOG),
                    UiImage::new(asset_server.load(DIALOG_BOX)),
                    NodeBundle {
                        style: Style {
                            width: Val::Px(350.0 * PIXEL_ZOOM as f32),
                            height: Val::Px(107.0 * PIXEL_ZOOM as f32),
                            margin: UiRect {
                                left: Val::Px(0.0),
                                right: Val::Auto,
                                top: Val::Auto,
                                bottom: Val::Auto,
                            },
                            justify_content: JustifyContent::Center,
                            justify_items: JustifyItems::Center,
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
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
                    // of the whole box, 70% is usable space for
                    // text and choices
                    const TEXT_HEIGHT_PERCENT: f32 = 23.0;
                    const TEXT_MARGIN_TOP_PERCENT: f32 = 2.0;
                    const CHOICES_HEIGHT_PERCENT: f32 = 45.0;

                    parent.spawn((
                        DialogText,
                        Name::new("Dialog text"),
                        RenderLayers::layer(render_layer::DIALOG),
                        TextBundle {
                            text,
                            style: Style {
                                width: Val::Percent(90.0),
                                height: Val::Percent(TEXT_HEIGHT_PERCENT),
                                margin: UiRect {
                                    top: Val::Percent(TEXT_MARGIN_TOP_PERCENT),
                                    bottom: Val::Px(10.0),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        },
                    ));
                    choices_box = parent
                        .spawn((
                            DialogChoicesBox,
                            Name::new("Dialog choices"),
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(85.0),
                                    height: Val::Percent(
                                        CHOICES_HEIGHT_PERCENT,
                                    ),
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                },
                                ..default()
                            },
                        ))
                        .id();
                });

            parent.spawn((
                DialogPortrait,
                Name::new("Portrait"),
                RenderLayers::layer(render_layer::DIALOG),
                NodeBundle {
                    style: Style {
                        width: Val::Px(common_assets::portraits::SIZE_PX.x),
                        height: Val::Px(common_assets::portraits::SIZE_PX.y),
                        margin: UiRect {
                            right: Val::Px(0.0),
                            left: Val::Auto,
                            top: Val::Auto,
                            bottom: Val::Auto,
                        },
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

        cmd.insert_resource(Self {
            camera,
            choices_box,
            last_frame_shown_at: Instant::now(),
            last_rendered_node: default(),
            root,
        });
    }
}

/// Renders UI for player choices.
fn show_player_choices(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    dialog_fe: &PortraitDialog,
    between: &[(&NodeName, &str)],
) {
    for (order, (node_name, choice_text)) in between.iter().enumerate() {
        let choice =
            spawn_choice(cmd, asset_server, order, node_name, choice_text);
        cmd.entity(dialog_fe.choices_box).add_child(choice);
    }
}

fn spawn_choice(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    order: usize,
    node_name: &NodeName,
    choice_text: &str,
) -> Entity {
    let is_first = order == 0;

    let choice = DialogChoice {
        of: node_name.clone(),
        order,
        is_selected: is_first,
    };

    cmd.spawn((
        Name::new(format!("Choice {order}: {node_name:?}")),
        RenderLayers::layer(render_layer::DIALOG),
        choice,
        Interaction::default(),
        NodeBundle {
            background_color: if is_first {
                CHOICE_HIGHLIGHT_COLOR.into()
            } else {
                Color::NONE.into()
            },
            style: Style {
                margin: if is_first {
                    default()
                } else {
                    UiRect::top(Val::Px(2.5))
                },
                padding: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(7.5),
                    bottom: Val::Px(10.0),
                },
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("Dialog choice text"),
            RenderLayers::layer(render_layer::DIALOG),
            TextBundle {
                text: Text::from_section(
                    format!("{}. {choice_text}", order + 1),
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: CHOICE_FONT_SIZE,
                        color: if is_first {
                            Color::BLACK
                        } else {
                            Color::WHITE
                        },
                    },
                ),
                ..default()
            },
        ));
    })
    .id()
}
