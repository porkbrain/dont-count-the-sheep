pub mod example;

mod aaatargets;

use std::{cmp::Ordering, time::Duration};

use aaatargets::DialogTargetChoice;
use bevy::{
    math::vec2,
    prelude::*,
    render::view::RenderLayers,
    text::{Text2dBounds, TextLayoutInfo},
    utils::Instant,
};
use common_visuals::camera::render_layer;

use self::aaatargets::DialogTargetGoto;
use crate::Character;

const FONT_SIZE: f32 = 21.0;
const CHOICE_FONT_SIZE: f32 = 18.0;
const FONT: &str = common_assets::paths::fonts::PENCIL1;
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

/// If inserted, then the game is in the dialog UI.
#[derive(Resource, Reflect)]
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
pub struct DialogRoot;

/// A child of the root entity that contains the text.
#[derive(Component)]
pub struct DialogText;

/// A child of the root entity that contains the portrait image.
#[derive(Component)]
pub struct DialogPortrait;

/// Entities that render choices in dialogs.
/// When advancing the dialog, the selected choice will be used to determine
/// the next [`DialogTarget`].
#[derive(Component, Clone, Debug)]
pub struct DialogChoice {
    of: DialogTargetChoice,
    is_selected: bool,
}

/// Next step in the dialog can take various forms.
enum Step {
    Text {
        speaker: Character,
        content: &'static str,
    },
    Choice {
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
pub fn advance(
    mut cmd: Commands,
    mut dialog: ResMut<PortraitDialog>,
    asset_server: Res<AssetServer>,

    root: Query<Entity, With<DialogRoot>>,
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
        .map(|(entity, choice)| (entity, choice.clone()));
    let outcome = advance_sequence(
        &mut cmd,
        &asset_server,
        &mut dialog,
        &mut text,
        root,
        choices,
        |speaker| {
            *portrait.single_mut() =
                asset_server.load(speaker.portrait_asset_path());
        },
    );

    if let SequenceFinished::Yes = outcome {
        trace!("Despawning dialog");

        cmd.remove_resource::<PortraitDialog>();
        cmd.entity(root).despawn_recursive();
    }
}

/// Spawns [`PortraitDialog`] resource and all the necessary UI components.
fn spawn(cmd: &mut Commands, asset_server: &AssetServer, sequence: Vec<Step>) {
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
            DialogRoot,
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
        &mut dialog,
        &mut text,
        root,
        std::iter::empty(),
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
                    .load(common_assets::paths::ui::DIALOG_BUBBLE),
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
fn advance_sequence(
    cmd: &mut Commands,
    asset_server: &AssetServer,
    dialog: &mut PortraitDialog,
    text: &mut Text,
    root: Entity,
    mut choices: impl Iterator<Item = (Entity, DialogChoice)>,
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
                dialog.sequence = story_point.sequence();
                dialog.sequence_index = 0;
            }
            Step::Choice { between } => {
                if let Some((_, choice)) = choices.find(|(_, c)| c.is_selected)
                {
                    // choice made, next sequence

                    dialog.sequence = choice.of.sequence();
                    dialog.sequence_index = 0;
                } else {
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
    let total = transform_manager.positions.len();

    between
        .iter()
        .enumerate()
        .map(move |(i, of)| {
            // This will be [`Less`] for the first choice,
            // [`Greater`] for the last choice, and [`Equal`]
            // for all the choices
            // in between.
            //
            // However, if there's only one choice, then it will
            // be [`Equal`].
            let ordering = match (total, i) {
                (1, _) => Ordering::Equal,
                (_, 0) => Ordering::Less,
                (_, i) if i == total - 1 => Ordering::Greater,
                _ => Ordering::Equal,
            };
            let asset = match ordering {
                Ordering::Less => "characters/dialog_option.png",
                Ordering::Greater => "characters/dialog_option.png",
                Ordering::Equal => "characters/dialog_option.png",
            }; // TODO

            let choice = DialogChoice {
                of: *of,
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
                                color: Color::BLACK,
                            },
                        )
                        .with_alignment(TextAlignment::Left),
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
