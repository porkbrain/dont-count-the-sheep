//! The player has an ability to inspect the world around them.
//! When inspecting, we show labels on anything the player can interact with.
//!
//! Search the wiki for inspect ability.
//!
//! TODO: explain the indirection with events

use std::borrow::Cow;

use bevy::prelude::*;
use common_ext::QueryExt;
use common_store::{GlobalStore, InspectAbilityStore};

use crate::Player;

/// We don't want to use a generic with [`InspectLabel`] because we need to
/// browse all labels at once.
/// That specific event they actually emit is not important for any logic here.
///
/// To avoid the need for a generic, we use a trait object.
pub trait ActionEvent: Event + Send + Sync + 'static {}

/// Implement this for all events.
impl<T: Event + Send + Sync + 'static> ActionEvent for T {}

/// When the inspect mode is active and the player is in a vicinity of an object
/// this label is shown on the object.
///
/// Use [`InspectLabelCategory::into_label`] to create a new label.
#[derive(Component, Reflect)]
pub struct InspectLabel {
    display: Cow<'static, str>,
    category: InspectLabelCategory,
    #[reflect(ignore)]
    emit_event_on_interacted: Option<Box<dyn ActionEvent>>,
}

/// Entities with [`InspectLabel`] and this component are considered when the
/// player hits the interact button.
/// The closest entity is chosen by default, but the player can change their
/// selection.
///
/// Therefore, scenes should signalize to this module that some action is
/// available by assigning this component to the entity that represents that
/// action.
#[derive(Component)]
pub struct ReadyForInteraction;

/// Different categories can have different radius of visibility based on the
/// player's experience.
#[derive(Default, Reflect, Clone, Copy)]
pub enum InspectLabelCategory {
    /// Default category, nothing special
    #[default]
    Default,
    /// NPCs have a larger radius of visibility.
    Npc,
}

#[derive(Component, Reflect)]
pub(crate) struct InspectLabelText;

/// Run this when action [`GlobalAction::Inspect`] was just pressed.
pub(crate) fn show_all_in_vicinity(
    mut cmd: Commands,
    store: Res<GlobalStore>,
    asset_server: Res<AssetServer>,

    player: Query<&GlobalTransform, With<Player>>,
    inspectable_object: Query<(Entity, &InspectLabel, &GlobalTransform)>,
) {
    trace!("Showing objects in vicinity of the player");

    let Some(player) = player.get_single_or_none() else {
        return;
    };
    let player = player.translation().truncate();

    for (entity, label, position) in inspectable_object.iter() {
        store.mark_as_seen(&label.display);

        let distance = player.distance(position.translation().truncate());
        if distance >= label.category.max_distance() {
            continue;
        }

        cmd.entity(entity).with_children(|parent| {
            parent.spawn(InspectLabelText).insert(Text2dBundle {
                // make it stand above others
                transform: Transform::from_translation(Vec3::Z),
                text: Text {
                    sections: vec![TextSection::new(
                        label.display.clone(),
                        TextStyle {
                            font: asset_server
                                .load(common_assets::fonts::TINY_PIXEL1),
                            font_size: 22.0, // TODO: buggy camera zoom
                            color: label.category.color(),
                        },
                    )],
                    linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    ..default()
                },
                ..default()
            });
        });
    }
}

/// Run this when action [`GlobalAction::Inspect`] was just released.
pub(crate) fn hide_all(
    mut cmd: Commands,
    text: Query<Entity, With<InspectLabelText>>,
) {
    for entity in text.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

impl InspectLabelCategory {
    /// Create a new label.
    pub fn into_label(
        self,
        label: impl Into<Cow<'static, str>>,
    ) -> InspectLabel {
        InspectLabel {
            category: self,
            display: label.into(),
            emit_event_on_interacted: None,
        }
    }
}

impl InspectLabel {
    /// Set an event to be emitted when the label is interacted with.
    pub fn emit_event_on_interacted(mut self, event: impl ActionEvent) -> Self {
        self.emit_event_on_interacted = Some(Box::new(event));
        self
    }
}

impl InspectLabelCategory {
    fn max_distance(self) -> f32 {
        match self {
            InspectLabelCategory::Default => 125.0,
            InspectLabelCategory::Npc => 175.0,
        }
    }

    fn color(self) -> Color {
        match self {
            InspectLabelCategory::Default => Color::WHITE,
            InspectLabelCategory::Npc => Color::ORANGE,
        }
    }
}
