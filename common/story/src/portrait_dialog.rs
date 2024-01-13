pub mod example;

mod aaatargets;

use std::time::Duration;

use aaatargets::DialogTarget;
use bevy::{
    math::vec2,
    prelude::*,
    render::view::RenderLayers,
    text::{Text2dBounds, TextLayoutInfo},
    utils::Instant,
};

use crate::Character;

/// TODO: organize render layers
const RENDER_LAYER: u8 = 25;
const FONT_SIZE: f32 = 21.0;
const FONT: &str = common_assets::paths::fonts::PENCIL1;
const PUSH_BUBBLE_TOP: f32 = 290.0;
const ROOT_POS: Vec2 = vec2(-640.0, -360.0);
const TEXT_BOUNDS: Vec2 = vec2(250.0, 120.0);
const MIN_TEXT_FRAME_TIME: Duration = Duration::from_millis(200);

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

#[derive(Component)]
pub struct DialogRoot;

#[derive(Component)]
pub struct DialogText;

#[derive(Component)]
pub struct DialogPortrait;

/// Next step in the dialog can take various forms.
enum Step {
    Text {
        speaker: Character,
        content: &'static str,
    },
    Choice {
        between: Vec<DialogTarget>,
    },
    GoTo {
        story_point: DialogTarget,
    },
}

trait DialogFragment {
    fn sequence() -> Vec<Step>;

    fn choice() -> &'static str {
        unreachable!(
            "Dialog {:?} cannot be made into a choice",
            core::any::type_name::<Self>()
        )
    }
}

pub fn advance(
    mut cmd: Commands,
    mut dialog: ResMut<PortraitDialog>,
    asset_server: Res<AssetServer>,

    root: Query<Entity, With<DialogRoot>>,
    mut text: Query<(&mut Text, &TextLayoutInfo), With<DialogText>>,
    mut portrait: Query<&mut Handle<Image>, With<DialogPortrait>>,
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

    let outcome = advance_sequence(&mut dialog, &mut text, |speaker| {
        *portrait.single_mut() =
            asset_server.load(speaker.portrait_asset_path());
    });

    if let SequenceFinished::Yes = outcome {
        trace!("Despawning dialog");

        cmd.remove_resource::<PortraitDialog>();
        cmd.entity(root.single()).despawn_recursive();
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

    let mut initial_speaker = None;
    let outcome = advance_sequence(&mut dialog, &mut text, |speaker| {
        initial_speaker = Some(speaker);
    });
    if let SequenceFinished::Yes = outcome {
        debug!("Dialog sequence finished before spawning");
        return;
    }

    cmd.insert_resource(dialog);

    cmd.spawn((
        Name::new("Dialog root"),
        SpatialBundle {
            transform: Transform::from_translation(ROOT_POS.extend(0.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("Dialog bubble"),
            RenderLayers::layer(RENDER_LAYER),
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
            RenderLayers::layer(RENDER_LAYER),
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
            RenderLayers::layer(RENDER_LAYER),
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
    /// Can be despawned or skipped spawning altogether.
    Yes,
    No,
}

/// Executes each dialog step until it reaches a step that requires player
/// input such as text or choice.
fn advance_sequence(
    dialog: &mut PortraitDialog,
    text: &mut Text,
    mut set_portrait_image: impl FnMut(Character),
) -> SequenceFinished {
    loop {
        let current_index = dialog.sequence_index;
        if current_index >= dialog.sequence.len() {
            break SequenceFinished::Yes;
        }

        dialog.sequence_index += 1;

        debug_assert!(!dialog.sequence.is_empty());

        match &dialog.sequence[current_index] {
            Step::Text { speaker, content } => {
                text.sections[0].value = content.to_string();

                if dialog.speaker != Some(*speaker) {
                    set_portrait_image(*speaker);
                    dialog.speaker = Some(*speaker);
                }

                dialog.last_frame_shown_at = Instant::now();
                break SequenceFinished::No;
            }
            Step::GoTo { story_point } => {
                dialog.sequence = story_point.sequence();
                dialog.sequence_index = 0;
            }
            _ => {
                todo!()
            }
        }
    }
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
