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
use common_action::{ActionState, GlobalAction};
use common_ext::QueryExt;
use common_store::{GlobalStore, InspectAbilityStore};
use common_visuals::{
    camera::PIXEL_ZOOM, BeginInterpolationEvent, ColorInterpolation,
};
use lazy_static::lazy_static;
use strum::EnumString;

use super::actor::player::TakeAwayPlayerControl;
use crate::top_down::{ActorMovementEvent, Player, TileKind, TopDownScene};

/// Useful for error labels.
pub const LIGHT_RED: Color = Color::rgb(1.0, 0.7, 0.7);

/// The label's bg is a rect with a half transparent color.
const BG_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.65);
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

/// Present in those entities with [`InspectLabel`] that have their label
/// currently displayed.
#[derive(Component, Reflect)]
pub(crate) struct InspectLabelDisplayed {
    bg: Entity,
    text: Entity,
    category_color: Color,
    /// It's a fade out animation
    being_hidden: bool,
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

/// What entity with [`ReadyForInteraction`] component is the one that would
/// be interacted with if the player pressed the interact button.
///
/// Only one entity can be highlighted at a time.
#[derive(Component)]
pub struct HighlightedForInteraction;

/// Different categories can have different radius of visibility based on the
/// player's experience.
#[derive(Default, Reflect, Clone, Copy, Debug, EnumString)]
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
///
/// In godot, this is done by using InspectLabel node with specific metadata.
/// See the wiki for more information.
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

/// System in this set consumes [`ChangeHighlightedInspectLabelEvent`]s.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChangeHighlightedInspectLabelEventConsumer;

/// Enables changing of the label's appearance.
/// This is only relevant for highlighted labels.
/// Useful to give the player some extra information why the interaction
/// is not actually possible due to some other condition, such as time (e.g.
/// shop after hours).
///
/// This change of appearance is not permanent and resets on first opportunity.
#[derive(Event)]
pub struct ChangeHighlightedInspectLabelEvent {
    /// The entity that has [`InspectLabel`] component
    pub entity: Entity,
    /// Edit options
    pub spawn_params: SpawnLabelBgAndTextParams,
}

/// Customize the appearance of a label.
#[derive(Default)]
pub struct SpawnLabelBgAndTextParams {
    /// Highlight the label visually.
    /// (does not overwrite interaction precedence)
    pub highlighted: bool,
    /// Overwrite the label font color that's by default given by its category.
    pub overwrite_font_color: Option<Color>,
    /// Change the text that's displayed.
    pub overwrite_display_text: Option<String>,
}

/// Respawns the label with provided appearance options.
pub(crate) fn change_highlighted_label(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<ChangeHighlightedInspectLabelEvent>,

    highlighted: Query<
        (&InspectLabel, &InspectLabelDisplayed),
        With<HighlightedForInteraction>,
    >,
) {
    let ChangeHighlightedInspectLabelEvent {
        entity,
        spawn_params,
    } = events.read().last().expect("At least one event present");

    let Some((label, displayed)) = highlighted.get_single_or_none() else {
        return;
    };

    cmd.entity(displayed.bg).despawn_recursive();
    cmd.entity(displayed.text).despawn_recursive();

    let displayed =
        spawn_label_bg_and_text(&mut cmd, &asset_server, label, spawn_params);
    cmd.entity(*entity)
        .add_child(displayed.bg)
        .add_child(displayed.text)
        .insert(displayed);
}

/// We want the player to know what would be interacted with if they clicked
/// the interact button.
///
/// 1. Find the closest entity with [`InspectLabel`]
/// 2. Set that entity as highlighted
pub(crate) fn highlight_what_would_be_interacted_with(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut begin_interpolation: EventWriter<BeginInterpolationEvent>,
    controls: Res<ActionState<GlobalAction>>,

    player: Query<
        &GlobalTransform,
        (With<Player>, Without<TakeAwayPlayerControl>),
    >,
    highlighted: Query<
        (Entity, &InspectLabel, &InspectLabelDisplayed),
        With<HighlightedForInteraction>,
    >,
    inspectable: Query<
        (
            Entity,
            &InspectLabel,
            Option<&InspectLabelDisplayed>,
            &GlobalTransform,
        ),
        With<ReadyForInteraction>,
    >,
) {
    let mut remove_old_highlight_if_present = || {
        if let Some((highlighted, label, old_displayed)) =
            highlighted.get_single_or_none()
        {
            cmd.entity(highlighted)
                .remove::<HighlightedForInteraction>();

            cmd.entity(old_displayed.bg).despawn_recursive();
            cmd.entity(old_displayed.text).despawn_recursive();

            let mut new_displayed = spawn_label_bg_and_text(
                &mut cmd,
                &asset_server,
                label,
                &default(),
            );
            if !controls.pressed(&GlobalAction::Inspect) {
                new_displayed
                    .schedule_hide(&mut begin_interpolation, highlighted);
            }
            cmd.entity(highlighted)
                .add_child(new_displayed.bg)
                .add_child(new_displayed.text)
                .insert(new_displayed);
        }
    };

    //
    // 1.
    //
    let Some(player) = player.get_single_or_none() else {
        return;
    };
    let player = player.translation().truncate();

    let Some((closest, label, displayed, _)) = inspectable
        .iter()
        // important to filter out entities without an event because those can
        // never be interacted with
        //
        // the system [`interact`] assumes on this condition
        .filter(|(_, label, ..)| label.emit_event_on_interacted.is_some())
        .map(|(entity, label, displayed, transform)| {
            let distance = transform.translation().truncate().distance(player);
            (entity, label, displayed, distance)
        })
        .min_by(|(_, _, _, a), (_, _, _, b)| {
            a.partial_cmp(b).expect("distance is always a number")
        })
    else {
        remove_old_highlight_if_present();
        return;
    };

    //
    // 2.
    //

    let highlighted_matches_closest = highlighted
        .get_single_or_none()
        .is_some_and(|(highlighted_entity, ..)| highlighted_entity == closest);
    if highlighted_matches_closest {
        // nothing to do, already in the state we want
        return;
    }

    remove_old_highlight_if_present();

    if let Some(InspectLabelDisplayed { bg, text, .. }) = displayed {
        cmd.entity(*bg).despawn_recursive();
        cmd.entity(*text).despawn_recursive();
    }

    let displayed = spawn_label_bg_and_text(
        &mut cmd,
        &asset_server,
        label,
        &SpawnLabelBgAndTextParams {
            highlighted: true,
            ..default()
        },
    );
    cmd.entity(closest)
        // Q: What if interpolation just finished in this frame and removed this
        // component?
        // Don't we need to order this system after the interpolation system?
        // A: No, because interpolation runs on FixedUpdate schedule.
        .insert(HighlightedForInteraction)
        .add_child(displayed.bg)
        .add_child(displayed.text)
        .insert(displayed);
}

/// This is registered in [`crate::top_down::default_setup_for_scene`].
///
/// Any logic that listens to [`ActionEvent`]s should be ordered _after_ this.
pub fn interact(
    mut cmd: Commands,

    label: Query<&InspectLabel, With<HighlightedForInteraction>>,
) {
    let Some(InspectLabel {
        // this will always be Some because we only insert the component
        // HighlightedForInteraction to inspect labels with an event
        emit_event_on_interacted: Some(event),
        ..
    }) = label.get_single_or_none()
    else {
        return;
    };

    event.send_deferred(&mut cmd);
}

/// Run this when action [`GlobalAction::Inspect`] is pressed.
/// It updates labels that come into the vicinity of the player.
pub(crate) fn show_all_in_vicinity(
    mut cmd: Commands,
    store: Res<GlobalStore>,
    asset_server: Res<AssetServer>,
    mut begin_interpolation: EventWriter<BeginInterpolationEvent>,

    player: Query<&GlobalTransform, With<Player>>,
    mut inspectable_objects: Query<(
        Entity,
        &InspectLabel,
        &GlobalTransform,
        Option<&mut InspectLabelDisplayed>,
        Option<&ReadyForInteraction>,
    )>,
    mut texts: Query<&mut Text, With<InspectLabelText>>,
    mut bgs: Query<&mut Sprite, With<InspectLabelBg>>,
) {
    let Some(player) = player.get_single_or_none() else {
        return;
    };
    let player = player.translation().truncate();

    for (entity, label, position, displayed, ready_for_interaction) in
        inspectable_objects.iter_mut()
    {
        store.mark_as_seen(&label.display);

        let distance = player.distance(position.translation().truncate());
        let should_be_shown = distance <= label.category.max_distance()
            || ready_for_interaction.is_some();

        match (should_be_shown, displayed) {
            // should not be shown and it's not, do nothing
            (false, None) => {}

            // should be shown, but is scheduled to be hidden, cancel that
            (true, Some(mut displayed)) if displayed.being_hidden => {
                displayed.cancel_hide(&mut cmd, &mut texts, &mut bgs);
            }
            // should be shown and is, we don't have to do anything here
            (true, Some(_)) => {}

            // should not be shown and it is scheduled to be hidden, do nothing
            (false, Some(displayed)) if displayed.being_hidden => {}
            // should not be shown and it is, hide it
            (false, Some(mut displayed)) => {
                displayed.schedule_hide(&mut begin_interpolation, entity);
            }

            // should be shown and it's not, show it
            (true, None) => {
                let displayed = spawn_label_bg_and_text(
                    &mut cmd,
                    &asset_server,
                    label,
                    &default(),
                );
                cmd.entity(entity)
                    .add_child(displayed.bg)
                    .add_child(displayed.text)
                    .insert(displayed);
            }
        }
    }
}

/// Attach the result as a component to the label's entity and the bg and text
/// children to the labels' entity.
fn spawn_label_bg_and_text(
    cmd: &mut Commands,
    asset_server: &Res<AssetServer>,
    label: &InspectLabel,
    SpawnLabelBgAndTextParams {
        highlighted,
        overwrite_font_color,
        overwrite_display_text,
    }: &SpawnLabelBgAndTextParams,
) -> InspectLabelDisplayed {
    trace!("Displaying label {}", label.display);

    let font_size =
        label.category.font_zone() + if *highlighted { 3.0 } else { 0.0 };

    let text_to_display = if let Some(text) = overwrite_display_text {
        text.as_str()
    } else {
        label.display.as_ref()
    };

    // We set this to be the zindex of the bg and text.
    // This is a dirty hack that puts the label always in front of everything.
    const Z_INDEX: f32 = 10.0;

    // bit of padding and then a few pixels per character
    // this is easier than waiting for the text to be rendered and
    // then using the logical size, and the impression doesn't
    // matter for such a short text
    let bg_box_width =
        font_size + font_size / 7.0 * text_to_display.len() as f32;
    let bg = cmd
        .spawn(InspectLabelBg)
        .insert(Name::new("InspectLabelBg"))
        .insert(SpriteBundle {
            transform: Transform::from_translation(Vec3::Z * Z_INDEX),
            sprite: Sprite {
                color: BG_COLOR * if *highlighted { 1.5 } else { 1.0 },
                custom_size: Some(Vec2::new(bg_box_width, font_size / 2.0)),
                ..default()
            },
            ..default()
        })
        .id();

    // make it stand above others with zindex
    let text = cmd
        .spawn(InspectLabelText)
        .insert(Name::new("InspectLabelText"))
        .insert(Text2dBundle {
            // We invert the pixel camera zoom, otherwise we'd end
            // up with pixelated text.
            // We end up using larger font size instead.
            transform: Transform::from_translation(Vec3::Z * Z_INDEX * 1.01)
                .with_scale(Vec3::splat(1.0 / PIXEL_ZOOM as f32)),
            text: Text {
                sections: vec![TextSection::new(
                    text_to_display,
                    TextStyle {
                        font: asset_server
                            .load(common_assets::fonts::TINY_PIXEL1),
                        font_size,
                        color: overwrite_font_color
                            .unwrap_or_else(|| label.category.color()),
                    },
                )],
                linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                ..default()
            },
            ..default()
        })
        .id();

    InspectLabelDisplayed {
        bg,
        text,
        category_color: label.category.color(),
        being_hidden: false,
    }
}

/// Run this when action [`GlobalAction::Inspect`] was just released.
/// It schedules removal of all labels by interpolating their color to none.
pub(crate) fn schedule_hide_all(
    mut begin_interpolation: EventWriter<BeginInterpolationEvent>,

    mut inspectable_objects: Query<
        (Entity, &mut InspectLabelDisplayed),
        Without<HighlightedForInteraction>,
    >,
) {
    for (entity, mut displayed) in inspectable_objects.iter_mut() {
        displayed.schedule_hide(&mut begin_interpolation, entity);
    }
}

impl InspectLabelDisplayed {
    fn schedule_hide(
        &mut self,
        begin_interpolation: &mut EventWriter<BeginInterpolationEvent>,
        label_entity: Entity,
    ) {
        self.being_hidden = true;
        let bg = self.bg;
        let text = self.text;
        let to_color = {
            let mut c = self.category_color;
            c.set_a(0.0);
            c
        };

        // looks better when the text fades out faster than the bg
        lazy_static! {
            static ref TEXT_ANIMATION_CURVE: CubicSegment<Vec2> =
                CubicSegment::new_bezier((0.9, 0.05), (0.9, 1.0));
            static ref BG_ANIMATION_CURVE: CubicSegment<Vec2> =
                CubicSegment::new_bezier((0.95, 0.01), (0.95, 1.0));
        }

        begin_interpolation.send(
            BeginInterpolationEvent::of_color(text, None, to_color)
                .over(FADE_OUT_IN)
                .with_animation_curve(TEXT_ANIMATION_CURVE.clone())
                .when_finished_do(move |cmd| {
                    cmd.entity(label_entity).remove::<Self>();
                    cmd.entity(text).despawn_recursive();
                }),
        );

        begin_interpolation.send(
            BeginInterpolationEvent::of_color(bg, None, Color::NONE)
                .over(FADE_OUT_IN)
                .with_animation_curve(BG_ANIMATION_CURVE.clone())
                .when_finished_despawn_recursive_itself(),
        );
    }

    fn cancel_hide(
        &mut self,
        cmd: &mut Commands,
        texts: &mut Query<&mut Text, With<InspectLabelText>>,
        bgs: &mut Query<&mut Sprite, With<InspectLabelBg>>,
    ) {
        cmd.entity(self.bg).remove::<ColorInterpolation>();
        bgs.get_mut(self.bg)
            .expect("BG must exist if display exists")
            .color = BG_COLOR;

        cmd.entity(self.text).remove::<ColorInterpolation>();
        texts
            .get_mut(self.text)
            .expect("Text must exist if display exists")
            .sections[0]
            .style
            .color = self.category_color;

        self.being_hidden = false;
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
    pub fn with_emit_event_on_interacted(
        mut self,
        event: impl ActionEvent,
    ) -> Self {
        self.emit_event_on_interacted = Some(Box::new(event));
        self
    }

    /// Set an event to be emitted when the label is interacted with.
    pub fn set_emit_event_on_interacted(&mut self, event: impl ActionEvent) {
        self.emit_event_on_interacted = Some(Box::new(event));
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
