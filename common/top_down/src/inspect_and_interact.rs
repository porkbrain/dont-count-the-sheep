//! The player has an ability to inspect the world around them.
//! When inspecting, we show labels on anything the player can interact with.
//!
//! Search the wiki for inspect ability.
//!
//! # Event system for interaction
//! Possible interactions in the world announce themselves to the [`interact`]
//! system.
//! They do that by inserting [`ReadyForInteraction`] component to their
//! relevant [`InspectLabel`] entity.
//! There is a property on the [`InspectLabel`] component that defines what
//! event should be emitted from the [`interact`] system when the player
//! decides to interact with that label.
//!
//! There is a common pattern: player enters a zone, hence they can interact
//! with something in that zone.
//! The zone is represented by a tile kind.
//! See the [`ZoneToInspectLabelEntity`] resource that simplifies this pattern.

use std::{borrow::Cow, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use common_ext::QueryExt;
use common_store::{GlobalStore, InspectAbilityStore};
use common_visuals::{
    camera::PIXEL_ZOOM, BeginInterpolationEvent, ColorInterpolation,
};

use crate::{ActorMovementEvent, Player, TileKind, TopDownScene};

/// The label's bg is a rect with a half transparent color.
const HALF_TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.5);
/// When the player releases the inspect button, the labels fade out in this
/// duration.
const FADE_OUT_IN: Duration = Duration::from_millis(5000);

/// We don't want to use a generic with [`InspectLabel`] because we need to
/// browse all labels at once.
/// That specific event they actually emit is not important for any logic here.
///
/// To avoid the need for a generic, we use a trait object.
///
/// You should not need to implement this manually as long as your type
/// implements [`Event`] and [`Clone`].
pub trait ActionEvent: Event {
    /// To keep the trait object safe, we cannot use a generic here.
    /// The solution to type erasure is to use commands.
    ///
    /// Take sure that the systems that listen to this event are running in at
    /// least the [`Update`] schedule or later.
    fn send_deferred(&self, cmd: &mut Commands);
}

/// Implement this for all events.
impl<T: Event + Clone> ActionEvent for T {
    fn send_deferred(&self, cmd: &mut Commands) {
        let cloned = self.clone();
        cmd.add(move |w: &mut World| {
            w.send_event(cloned);
        });
    }
}

/// When the inspect mode is active and the player is in a vicinity of an object
/// this label is shown on the object.
///
/// Use [`InspectLabelCategory::into_label`] to create a new label.
///
/// This works only if the entity has also [`Transform`] component.
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
#[derive(Default, Reflect, Clone, Copy, Debug)]
pub enum InspectLabelCategory {
    /// Default category, nothing special
    #[default]
    Default,
    /// NPCs have a larger radius of visibility.
    Npc,
}

/// The text of the label.
#[derive(Component, Reflect)]
pub(crate) struct InspectLabelText;
/// The half transparent background of the label.
#[derive(Component, Reflect)]
pub(crate) struct InspectLabelBg;

/// A helper resource that maps local tile kinds to entities that have
/// [`InspectLabel`] component.
///
/// When entities are mapped this way, they are assigned the
/// [`ReadyForInteraction`] component when the player enters the given zone.
#[derive(Resource, Reflect, Default)]
pub struct ZoneToInspectLabelEntity<L> {
    /// The key is the local tile kind, the value is some entity that has
    /// [`InspectLabel`] component.
    pub map: HashMap<L, Entity>,
}

pub(crate) fn match_interact_label_with_action_event<T: TopDownScene>(
    mut cmd: Commands,
    mut events: EventReader<ActorMovementEvent<T::LocalTileKind>>,
    zone_to_inspect_label_entity: Res<
        ZoneToInspectLabelEntity<T::LocalTileKind>,
    >,
) {
    for event in events.read().filter(|event| event.is_player()) {
        match event {
            ActorMovementEvent::ZoneEntered {
                zone: TileKind::Local(local_zone),
                ..
            } => {
                zone_to_inspect_label_entity.map.get(local_zone).inspect(
                    |entity| {
                        cmd.entity(**entity).insert(ReadyForInteraction);
                    },
                );
            }
            ActorMovementEvent::ZoneLeft {
                zone: TileKind::Local(local_zone),
                ..
            } => {
                zone_to_inspect_label_entity.map.get(local_zone).inspect(
                    |entity| {
                        cmd.entity(**entity).remove::<ReadyForInteraction>();
                    },
                );
            }
            _ => {}
        };
    }
}

/// This is registered in [`crate::default_setup_for_scene`].
///
/// Any logic that listens to [`ActionEvent`]s should be ordered _after_ this.
pub fn interact(
    mut cmd: Commands,

    player: Query<&GlobalTransform, With<Player>>,
    inspectable_objects: Query<
        (&InspectLabel, &GlobalTransform),
        With<ReadyForInteraction>,
    >,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };
    let player = player.translation().truncate();

    let closest = inspectable_objects
        .iter()
        .filter(|(label, _)| label.emit_event_on_interacted.is_some()) // !
        .map(|(label, transform)| {
            let distance = transform.translation().truncate().distance(player);
            (label, distance)
        })
        .min_by(|(_, a), (_, b)| {
            a.partial_cmp(b).expect("distance is always a number")
        })
        .map(|(label, _)| label);

    if let Some(closest) = closest {
        let event = closest.emit_event_on_interacted.as_ref().unwrap(); // !

        event.send_deferred(&mut cmd);
    }
}

/// Run this when action [`GlobalAction::Inspect`] is pressed.
/// It updates labels that come into the vicinity of the player.
pub(crate) fn show_all_in_vicinity(
    mut cmd: Commands,
    store: Res<GlobalStore>,
    asset_server: Res<AssetServer>,

    player: Query<&GlobalTransform, With<Player>>,
    inspectable_objects: Query<(
        Entity,
        &InspectLabel,
        &GlobalTransform,
        Option<&Children>,
    )>,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };
    let player = player.translation().truncate();

    for (entity, label, position, children) in inspectable_objects.iter() {
        store.mark_as_seen(&label.display);

        let distance = player.distance(position.translation().truncate());
        let should_be_shown = distance <= label.category.max_distance();

        match (should_be_shown, children) {
            // should not be shown and it's not, do nothing
            (false, None) => {}

            // should be shown and is, we don't have to do anything here because
            // `cancel_hide_all` got us covered
            (true, Some(_)) => {}

            // should not be shown and it is, hide it
            (false, Some(children)) => {
                trace!("Label {} going out of the view", label.display);

                cmd.entity(entity).remove::<Children>();
                for child in children {
                    cmd.entity(*child).despawn();
                }
            }

            // should be shown and it's not, show it
            (true, None) => {
                trace!("Displaying label {}", label.display);

                let font_size = label.category.font_zone();

                // bit of padding and then a few pixels per character
                // this is easier than waiting for the text to be rendered and
                // then using the logical size, and the impression doesn't
                // matter for such a short text
                let bg_box_width =
                    font_size + font_size / 7.0 * label.display.len() as f32;
                let bg = cmd
                    .spawn(InspectLabelBg)
                    .insert(SpriteBundle {
                        transform: Transform::from_translation(Vec3::Z),
                        sprite: Sprite {
                            color: HALF_TRANSPARENT,
                            custom_size: Some(Vec2::new(
                                bg_box_width,
                                font_size / 2.0,
                            )),
                            ..default()
                        },
                        ..default()
                    })
                    .id();

                // make it stand above others with zindex
                let txt = cmd
                    .spawn(InspectLabelText)
                    .insert(Text2dBundle {
                        // We invert the pixel camera zoom, otherwise we'd end
                        // up with pixelated text.
                        // We end up using larger font size instead.
                        transform: Transform::from_translation(Vec3::Z * 2.0)
                            .with_scale(Vec3::splat(1.0 / PIXEL_ZOOM as f32)),
                        text: Text {
                            sections: vec![TextSection::new(
                                label.display.clone(),
                                TextStyle {
                                    font: asset_server.load(
                                        common_assets::fonts::TINY_PIXEL1,
                                    ),
                                    font_size,
                                    color: label.category.color(),
                                },
                            )],
                            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                            ..default()
                        },
                        ..default()
                    })
                    .id();

                cmd.entity(entity).insert_children(0, &[bg, txt]);
            }
        }

        if distance >= label.category.max_distance() {
            continue;
        }
    }
}

/// Run this when action [`GlobalAction::Inspect`] is just pressed.
/// It cancels eventual [`schedule_hide_all`] call that scheduled the fade out
/// and removal of the box.
pub(crate) fn cancel_hide_all(
    mut cmd: Commands,

    inspectable_objects: Query<&InspectLabel>,
    mut text: Query<(Entity, &Parent, &mut Text), With<InspectLabelText>>,
    mut bg: Query<(Entity, &mut Sprite), With<InspectLabelBg>>,
) {
    for (entity, parent, mut text) in text.iter_mut() {
        let parent = parent.get();
        let color = inspectable_objects.get(parent).unwrap().category.color();
        text.sections[0].style.color = color;

        cmd.entity(entity).remove::<ColorInterpolation>();
    }

    for (entity, mut sprite) in bg.iter_mut() {
        sprite.color = HALF_TRANSPARENT;
        cmd.entity(entity).remove::<ColorInterpolation>();
    }
}

/// Run this when action [`GlobalAction::Inspect`] was just released.
/// It schedules removal of all labels by interpolating their color to none.
pub(crate) fn schedule_hide_all(
    mut begin_interpolation: EventWriter<BeginInterpolationEvent>,

    inspectable_objects: Query<&InspectLabel>,
    text: Query<(Entity, &Parent), With<InspectLabelText>>,
    bg: Query<Entity, With<InspectLabelBg>>,
) {
    // looks better when the text fades out faster than the bg
    let text_animation_curve =
        CubicSegment::new_bezier((0.9, 0.05), (0.9, 1.0));
    let bg_animation_curve =
        CubicSegment::new_bezier((0.95, 0.01), (0.95, 1.0));

    for (entity, parent) in text.iter() {
        let parent = parent.get();
        let to_color = {
            let mut c =
                inspectable_objects.get(parent).unwrap().category.color();
            c.set_a(0.0);
            c
        };

        begin_interpolation.send(
            BeginInterpolationEvent::of_color(entity, None, to_color)
                .over(FADE_OUT_IN)
                .with_animation_curve(text_animation_curve.clone())
                .when_finished_do(move |cmd| {
                    cmd.entity(parent).remove::<Children>();
                    cmd.entity(entity).despawn();
                }),
        );
    }

    for entity in bg.iter() {
        begin_interpolation.send(
            BeginInterpolationEvent::of_color(entity, None, Color::NONE)
                .over(FADE_OUT_IN)
                .with_animation_curve(bg_animation_curve.clone())
                .when_finished_despawn_itself(),
        );
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

    /// The font size of the label text that shows up when inspecting.
    fn font_zone(self) -> f32 {
        match self {
            InspectLabelCategory::Default => 12.0,
            InspectLabelCategory::Npc => 16.0,
        }
    }
}
