//! Informs the player about things that are happening using a simple list of
//! notifications.

use bevy::ui::RelativeCursorPosition;
use common_assets::ui::HEARTBEAT_ATLAS_SIZE;
use common_visuals::{
    camera::{MainCamera, PIXEL_ZOOM},
    BeginInterpolationEvent,
};

use super::{MARGIN_LEFT_PX, MARGIN_TOP_PX};
use crate::prelude::*;

const FONT_SIZE: f32 = 18.0;
const MAX_DISPLAYED_NOTIFICATIONS: usize = 5;
const NOTIFICATION_DISPLAY_TIME: Duration = from_millis(5_000);

/// A notification to display to the user.
#[derive(Debug)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[cfg_attr(feature = "devtools", reflect(Default))]
pub enum Notification {
    /// Simple text notification.
    PlainText(String),
}

/// To display notifications to the user, push them into this resource.
#[derive(Resource, Debug, Default)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[cfg_attr(feature = "devtools", reflect(Resource))]
pub struct NotificationFifo(Vec<FifoElement>);

#[derive(Debug)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[cfg_attr(feature = "devtools", reflect(Default))]
pub(crate) struct FifoElement {
    notification: Notification,
    displayed_for: Stopwatch,
    entity: Option<Entity>,
}

impl NotificationFifo {
    /// Push a notification to the end of the queue.
    pub fn push(&mut self, notification: Notification) {
        self.0.push(FifoElement {
            notification,
            displayed_for: Stopwatch::new(),
            entity: None,
        });
    }
}

impl Notification {
    /// Create a new notification for when the player discovers a new location.
    ///
    /// LOCALIZATION
    pub fn new_location_discovered(location: &str) -> Self {
        Notification::PlainText(format!("New location discovered: {location}"))
    }
}

#[derive(Component)]
pub(crate) struct NotificationRoot;

pub(crate) fn spawn(
    mut cmd: Commands,

    camera: Query<Entity, With<MainCamera>>,
) {
    cmd.spawn((
        Name::new("Notifications"),
        NotificationRoot,
        TargetCamera(camera.single()),
        Interaction::default(),
        RelativeCursorPosition::default(),
        NodeBundle {
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.9).into(),
            focus_policy: bevy::ui::FocusPolicy::Block,
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                flex_wrap: FlexWrap::Wrap,

                top: Val::Px(
                    // this is where heartbeat daybar ends
                    MARGIN_TOP_PX + HEARTBEAT_ATLAS_SIZE.y * PIXEL_ZOOM as f32
                    +
                    // this is a space between daybar and notifications
                    MARGIN_TOP_PX,
                ),
                left: Val::Px(MARGIN_LEFT_PX),

                padding: UiRect {
                    left: Val::Px(MARGIN_LEFT_PX),
                    right: Val::Px(MARGIN_LEFT_PX),
                    ..default()
                },

                ..default()
            },
            ..default()
        },
    ));
}

pub(crate) fn despawn(
    mut cmd: Commands,

    root: Query<Entity, With<NotificationRoot>>,
) {
    cmd.entity(root.single()).despawn_recursive();
}

pub(crate) fn update(
    mut cmd: Commands,
    mut notifications: ResMut<NotificationFifo>,
    mut begin_interpolation: EventWriter<BeginInterpolationEvent>,
    time: Res<Time>,

    root: Query<Entity, With<NotificationRoot>>,
) {
    let mut displayed_notifications = notifications
        .0
        .iter()
        .filter(|n| n.entity.is_some())
        .count();

    notifications.0.retain_mut(|el| {
        if let Some(entity) = el.entity {
            el.displayed_for.tick(time.delta());

            if el.displayed_for.elapsed() > NOTIFICATION_DISPLAY_TIME {
                // we get rid of this notification

                displayed_notifications -= 1;
                begin_interpolation.send(
                    BeginInterpolationEvent::of_color(
                        entity,
                        None,
                        Color::NONE,
                    )
                    .when_finished_despawn_recursive_itself(),
                );

                return false;
            }
        } else if displayed_notifications < MAX_DISPLAYED_NOTIFICATIONS {
            // we display this notification

            let node_id = el.notification.spawn(&mut cmd);
            el.entity = Some(node_id);

            displayed_notifications += 1;
            cmd.entity(root.single()).add_child(node_id);
        }

        true
    });
}

impl Notification {
    fn spawn(&self, cmd: &mut Commands) -> Entity {
        match self {
            Notification::PlainText(text) => cmd
                .spawn((
                    Name::from("Displayed notification"),
                    TextBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font_size: FONT_SIZE,
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::vertical(Val::Px(
                                2.0 * MARGIN_TOP_PX,
                            )),

                            min_width: Val::Px(
                                HEARTBEAT_ATLAS_SIZE.x * PIXEL_ZOOM as f32,
                            ),

                            ..default()
                        },
                        ..default()
                    },
                ))
                .id(),
        }
    }
}

#[cfg(feature = "devtools")]
impl Default for Notification {
    fn default() -> Self {
        Notification::PlainText("Test notification".to_string())
    }
}

#[cfg(feature = "devtools")]
impl Default for FifoElement {
    fn default() -> Self {
        FifoElement {
            notification: Notification::default(),
            displayed_for: Stopwatch::new(),
            entity: None,
        }
    }
}
