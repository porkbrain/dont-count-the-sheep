//! When a dialog is spawned, it's already loaded as it should look and does not
//! require any additional actions.
//!
//! # Systems
//! - [`crate::portrait_dialog::advance`] that advances the dialog one step
//!   further, presumably fire it when the player presses the interact key
//! - [`crate::portrait_dialog::change_selection`] that changes the selected
//!   choice based on whether the player **just** pressed up or down, run it if
//!   the player pressed some movement key

pub mod apartment_elevator;

mod aaatargets;

use std::{collections::BTreeMap, time::Duration};

pub use aaatargets::DialogRoot;
use aaatargets::{DialogTargetChoice, DialogTargetGoto};
use bevy::{
    math::vec2,
    prelude::*,
    render::view::RenderLayers,
    text::{Text2dBounds, TextLayoutInfo},
    utils::Instant,
};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use common_action::{ActionState, GlobalAction};
use common_store::{DialogStore, GlobalStore};
use common_visuals::camera::render_layer;
use itertools::Itertools;

use crate::Character;

const FONT_SIZE: f32 = 21.0;
const CHOICE_FONT_SIZE: f32 = 17.0;
const FONT: &str = common_assets::fonts::PENCIL1;
const PUSH_BUBBLE_TOP: f32 = 290.0;
const ROOT_POS: Vec2 = vec2(-640.0, -360.0);
const TEXT_BOUNDS: Vec2 = vec2(250.0, 120.0);
const OPTION_TEXT_BOUNDS: Vec2 = vec2(250.0, 80.0);
const MIN_TEXT_FRAME_TIME: Duration = Duration::from_millis(200);

/// Will be true if in a dialog that takes away player control.
pub fn in_portrait_dialog() -> impl FnMut(Option<Res<PortraitDialog>>) -> bool {
    move |dialog| dialog.is_some()
}

/// Will be false if in a dialog that takes away player control.
pub fn not_in_portrait_dialog(
) -> impl FnMut(Option<Res<PortraitDialog>>) -> bool {
    move |dialog| dialog.is_none()
}

/// If inserted, then the game is in the dialog UI. TODO
#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct PortraitDialog {
    /// We force a small delay between frames to prevent the player from
    /// skipping through the dialog way too fast.
    /// The tiny delay lets the brain to at least get the gist of what's
    /// being said.
    last_frame_shown_at: Instant,
    /// Currently iterated steps.
    /// Can either end, display choices, or go to another story point.
    sequence: Vec<Step>,
    /// Index of the current step.
    sequence_index: usize,
    /// Determines the portrait used
    speaker: Option<Character>,
}

/// The root entity of the dialog UI.
#[derive(Component)]
pub struct DialogUiRoot;

/// A child of the root entity that contains the text.
#[derive(Component)]
pub struct DialogText;

/// A child of the root entity that contains the portrait image.
#[derive(Component)]
pub struct DialogPortrait;

/// Entities that render choices in dialogs.
/// When advancing the dialog, the selected choice will be used to determine
/// the next sequence.
#[derive(Component, Clone, Debug)]
pub struct DialogChoice {
    of: DialogTargetChoice,
    /// Starts at 0.
    order: usize,
    /// Is selected either if it's the first choice or if the player changed
    /// selection to this.
    is_selected: bool,
}

/// Next step in the dialog can take various forms.
enum Step {
    Text {
        speaker: Character,
        content: &'static str,
    },
    Choice {
        speaker: Character,
        content: &'static str,
        between: Vec<DialogTargetChoice>,
    },
    GoTo {
        story_point: DialogTargetGoto,
    },
}

trait AsSequence {
    fn sequence() -> Vec<Step>;
}

trait AsChoice: AsSequence {
    fn choice() -> &'static str;
}

/// Call this to load the next step in the dialog.
/// A step could be some text, or a player choice, etc.
#[allow(clippy::too_many_arguments)]
pub fn advance(
    mut cmd: Commands,
    mut dialog: ResMut<PortraitDialog>,
    asset_server: Res<AssetServer>,
    global_store: Res<GlobalStore>,
    mut controls: ResMut<ActionState<GlobalAction>>,

    root: Query<Entity, With<DialogUiRoot>>,
    mut text: Query<(&mut Text, &TextLayoutInfo), With<DialogText>>,
    mut portrait: Query<&mut Handle<Image>, With<DialogPortrait>>,
    choices: Query<(Entity, &DialogChoice)>,
) {
    if dialog.last_frame_shown_at.elapsed() < MIN_TEXT_FRAME_TIME {
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
                dialog.last_frame_shown_at = Instant::now();
                return;
            }
        }
    }

    let root = root.single();

    let choices = choices
        .iter()
        .map(|(entity, choice)| (entity, choice.clone()))
        .collect_vec();
    let outcome = advance_sequence(
        &mut cmd,
        &asset_server,
        &global_store,
        &mut dialog,
        &mut text,
        root,
        &choices,
        |speaker| {
            *portrait.single_mut() =
                asset_server.load(speaker.portrait_asset_path());
        },
    );

    if let SequenceFinished::Yes = outcome {
        trace!("Despawning dialog");

        cmd.remove_resource::<PortraitDialog>();
        cmd.entity(root).despawn_recursive();

        controls.consume_all();
    }
}

/// Run if pressed some movement key.
/// If there are choices, then the selection will be changed if the movement
/// was either up or down.
pub fn change_selection(
    controls: Res<ActionState<GlobalAction>>,
    asset_server: Res<AssetServer>,

    mut choices: Query<(&Children, &mut DialogChoice, &mut Handle<Image>)>,
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

        **image =
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

        **image = asset_server.load(common_assets::dialog::DIALOG_CHOICE);

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

/// Spawns [`PortraitDialog`] resource and all the necessary UI components.
fn spawn(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    global_store: &GlobalStore,
    sequence: Vec<Step>,
) {
    let mut dialog = PortraitDialog::new(sequence);
    let mut text = Text::from_section(
        "",
        TextStyle {
            font: asset_server.load(FONT),
            font_size: FONT_SIZE,
            color: Color::BLACK,
        },
    );

    let root = cmd
        .spawn((
            Name::new("Dialog root"),
            DialogUiRoot,
            SpatialBundle {
                transform: Transform::from_translation(ROOT_POS.extend(0.0)),
                ..default()
            },
        ))
        .id();

    let mut initial_speaker = None;
    let outcome = advance_sequence(
        cmd,
        asset_server,
        global_store,
        &mut dialog,
        &mut text,
        root,
        &[],
        |speaker| {
            initial_speaker = Some(speaker);
        },
    );
    if let SequenceFinished::Yes = outcome {
        debug!("Dialog sequence finished before spawning");
        cmd.entity(root).despawn();
        return;
    }

    cmd.insert_resource(dialog);
    cmd.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new("Dialog bubble"),
            RenderLayers::layer(render_layer::DIALOG),
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    PUSH_BUBBLE_TOP,
                    -1.0,
                )),
                texture: asset_server
                    .load(common_assets::dialog::DIALOG_BUBBLE),
                ..default()
            },
        ));

        parent.spawn((
            DialogPortrait,
            Name::new("Dialog portrait"),
            RenderLayers::layer(render_layer::DIALOG),
            SpriteBundle {
                texture: if let Some(speaker) = initial_speaker {
                    asset_server.load(speaker.portrait_asset_path())
                } else {
                    default()
                },
                ..default()
            },
        ));

        parent.spawn((
            DialogText,
            Name::new("Dialog text"),
            RenderLayers::layer(render_layer::DIALOG),
            Text2dBundle {
                text,
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    PUSH_BUBBLE_TOP - 10.0,
                    1.0,
                )),
                text_2d_bounds: Text2dBounds { size: TEXT_BOUNDS },
                ..default()
            },
        ));
    });
}

#[must_use]
enum SequenceFinished {
    /// Can be despawned or spawning skipped altogether.
    Yes,
    /// More things to do.
    No,
}

/// Executes each dialog step until it reaches a step that requires player
/// input such as text or choice.
#[allow(clippy::too_many_arguments)]
fn advance_sequence(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    global_store: &GlobalStore,
    dialog: &mut PortraitDialog,
    text: &mut Text,
    root: Entity,
    choices: &[(Entity, DialogChoice)],
    mut set_portrait_image: impl FnMut(Character),
) -> SequenceFinished {
    loop {
        let current_index = dialog.sequence_index;
        if current_index >= dialog.sequence.len() {
            break SequenceFinished::Yes;
        }

        debug_assert!(!dialog.sequence.is_empty());

        match &dialog.sequence[current_index] {
            Step::Text { speaker, content } => {
                text.sections[0].value = content.to_string();

                if dialog.speaker != Some(*speaker) {
                    set_portrait_image(*speaker);
                    dialog.speaker = Some(*speaker);
                }

                dialog.sequence_index += 1;
                dialog.last_frame_shown_at = Instant::now();
                break SequenceFinished::No;
            }
            Step::GoTo { story_point } => {
                // next sequence

                global_store.insert_dialog_type_path(story_point.type_path());
                dialog.sequence = story_point.sequence();
                dialog.sequence_index = 0;
            }
            Step::Choice {
                speaker,
                content,
                between,
            } => {
                if let Some((_, choice)) =
                    choices.iter().find(|(_, c)| c.is_selected)
                {
                    // choice made, next sequence

                    global_store.insert_dialog_type_path(choice.of.type_path());

                    choices.iter().for_each(|(entity, _)| {
                        cmd.entity(*entity).despawn_recursive()
                    });

                    dialog.sequence = choice.of.sequence();
                    dialog.sequence_index = 0;
                } else {
                    text.sections[0].value = content.to_string();
                    if dialog.speaker != Some(*speaker) {
                        set_portrait_image(*speaker);
                        dialog.speaker = Some(*speaker);
                    }

                    // spawn choices

                    let total = between.len();
                    debug_assert_ne!(0, total);

                    let transform_manager = dialog
                        .speaker
                        .map(|c| c.choice_transform_manager(total))
                        .unwrap_or_else(|| {
                            ChoiceTransformManager::no_portrait(total)
                        });

                    let children = spawn_choices(
                        cmd,
                        asset_server,
                        transform_manager,
                        between,
                    );

                    cmd.entity(root).push_children(&children);

                    break SequenceFinished::No;
                }
            }
        }
    }
}

/// Each choice is spawned as a separate entity.
fn spawn_choices(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    transform_manager: ChoiceTransformManager,
    between: &[DialogTargetChoice],
) -> Vec<Entity> {
    between
        .iter()
        .enumerate()
        .map(move |(i, of)| {
            let (asset, color) = if i == 0 {
                (
                    common_assets::dialog::DIALOG_CHOICE_HIGHLIGHTED,
                    Color::WHITE,
                )
            } else {
                (common_assets::dialog::DIALOG_CHOICE, Color::BLACK)
            };

            let choice = DialogChoice {
                of: *of,
                order: i,
                is_selected: i == 0,
            };
            let sprite = SpriteBundle {
                transform: transform_manager.get(i),
                texture: asset_server.load(asset),
                ..default()
            };

            cmd.spawn((
                Name::new(format!("Dialog choice {i}: {of:?}")),
                RenderLayers::layer(render_layer::DIALOG),
                choice,
                sprite,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Dialog choice text"),
                    RenderLayers::layer(render_layer::DIALOG),
                    Text2dBundle {
                        text_2d_bounds: Text2dBounds {
                            size: OPTION_TEXT_BOUNDS,
                        },
                        text: Text::from_section(
                            of.choice(),
                            TextStyle {
                                font: asset_server.load(FONT),
                                font_size: CHOICE_FONT_SIZE,
                                color,
                            },
                        )
                        .with_justify(JustifyText::Left),
                        ..default()
                    },
                ));
            })
            .id()
        })
        .collect()
}

impl PortraitDialog {
    fn new(sequence: Vec<Step>) -> Self {
        Self {
            sequence,
            sequence_index: 0,
            speaker: None,
            last_frame_shown_at: Instant::now(),
        }
    }
}

pub(super) struct ChoiceTransformManager {
    positions: Vec<Vec2>,
}

impl ChoiceTransformManager {
    pub(super) fn no_portrait(total_choices: usize) -> Self {
        Self {
            positions: match total_choices {
                1 => vec![vec2(0.0, -75.0)],
                2 => vec![vec2(0.0, -75.0), vec2(0.0, -140.0)],
                3 => vec![vec2(0.0, -5.0), vec2(0.0, -75.0), vec2(0.0, -145.0)],
                total => todo!("Cannot handle {total} choices"),
            },
        }
    }

    fn get(&self, index: usize) -> Transform {
        debug_assert!(
            index < self.positions.len(),
            "Cannot get position index for index {index}"
        );
        Transform::from_translation(self.positions[index].extend(index as f32))
    }
}

impl Character {
    fn choice_transform_manager(
        self,
        total_choices: usize,
    ) -> ChoiceTransformManager {
        #[allow(clippy::match_single_binding)]
        let positions = match total_choices {
            1 => match self {
                _ => vec![vec2(240.0, -75.0)],
            },
            2 => match self {
                _ => vec![vec2(240.0, -75.0), vec2(260.0, -140.0)],
            },
            3 => match self {
                _ => vec![
                    vec2(227.0, -5.0),
                    vec2(240.0, -75.0),
                    vec2(260.0, -145.0),
                ],
            },
            total => todo!("Cannot handle {total} choices"),
        };
        debug_assert_eq!(total_choices, positions.len());

        ChoiceTransformManager { positions }
    }
}

impl Step {
    fn text(character: Character, content: &'static str) -> Self {
        Self::Text {
            speaker: character,
            content,
        }
    }
}

impl Default for PortraitDialog {
    fn default() -> Self {
        Self {
            last_frame_shown_at: Instant::now(),
            sequence: vec![],
            sequence_index: 0,
            speaker: None,
        }
    }
}
