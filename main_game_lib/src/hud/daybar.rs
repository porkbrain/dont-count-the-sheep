//! Daybar is a HUD element that shows the player's progress through the day.
//! It attaches itself to the [`MainCamera`].
//!
//! See wiki for more information about day progress.

use bevy::ui::RelativeCursorPosition;
use common_ext::QueryExt;
use common_visuals::camera::MainCamera;

use crate::prelude::*;

/// Manages in-game time progression.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DayBar {
    /// The current progress through the day, from 0 to 1.
    pub(crate) progress: f32,
}

/// Increases the day bar based on the situation that's happening.
#[derive(Event)]
pub enum IncreaseDayBarEvent {
    /// Non trivial scene transition, such as leaving a building.
    ChangedScene,
    /// Finished meditating.
    Meditated,
    /// Sets the daybar progress to 0.
    Reset,
    /// Custom amount of increase in the day bar.
    /// The final progress is clamped between 0 and 1.
    Custom(f32),
}

/// What sort of things are dependent on status of the daybar.
#[derive(Debug)]
pub enum DayBarDependent {
    /// The span of time when the mall is open.
    MallOpenHours,
    /// The span of time when the clinic is open.
    ClinicOpenHours,
}

#[derive(Component)]
pub(crate) struct DayBarRoot;
#[derive(Component)]
pub(crate) struct DayBarProgress;

pub(crate) fn spawn(
    mut cmd: Commands,
    daybar: Res<DayBar>,

    camera: Query<Entity, With<MainCamera>>,
) {
    cmd.spawn((
        Name::new("DayBar"),
        DayBarRoot,
        TargetCamera(camera.single()),
        Interaction::default(),
        RelativeCursorPosition::default(),
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                width: Val::Px(200.0),
                min_width: Val::Percent(10.0),
                aspect_ratio: Some(7.5),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::GREEN.with_a(0.75)),
            border_color: BorderColor(Color::WHITE.with_a(0.75)),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("ProgressBar"),
            DayBarProgress,
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(daybar.progress * 100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::RED.with_a(0.75)),
                ..default()
            },
        ));
    });
}

pub(crate) fn despawn(
    mut cmd: Commands,
    root: Query<Entity, With<DayBarRoot>>,
) {
    cmd.entity(root.single()).despawn_recursive();
}

pub(crate) fn increase(
    mut events: EventReader<IncreaseDayBarEvent>,
    mut daybar: ResMut<DayBar>,

    mut progress: Query<&mut Style, With<DayBarProgress>>,
) {
    for event in events.read() {
        let amount = match event {
            IncreaseDayBarEvent::ChangedScene => 0.01,
            IncreaseDayBarEvent::Meditated => 0.05,
            IncreaseDayBarEvent::Custom(amount) => *amount,
            IncreaseDayBarEvent::Reset => -daybar.progress,
        };

        daybar.progress = (daybar.progress + amount).clamp(0.0, 1.0);
    }

    if let Some(mut progress) = progress.get_single_mut_or_none() {
        progress.width = Val::Percent(daybar.progress * 100.0);
    }
}

#[cfg(feature = "devtools")]
pub(crate) fn change_progress(
    mut events: EventWriter<IncreaseDayBarEvent>,

    root: Query<
        (&Interaction, &RelativeCursorPosition),
        (Changed<Interaction>, With<DayBarRoot>),
    >,
) {
    for (interaction, cursor_position) in root.iter() {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        if let Some(position) = cursor_position.normalized {
            events.send(IncreaseDayBarEvent::Reset);
            events.send(IncreaseDayBarEvent::Custom(position.x));
        }
    }
}

impl Default for DayBar {
    fn default() -> Self {
        Self { progress: 0.0 }
    }
}

impl DayBar {
    /// Returns `true` if the day is over.
    pub fn is_depleted(&self) -> bool {
        self.progress >= 1.0
    }

    /// Whether it's time for something to happen.
    pub fn is_it_time_for(&self, what: DayBarDependent) -> bool {
        let range = match what {
            DayBarDependent::MallOpenHours => ..0.75,
            DayBarDependent::ClinicOpenHours => ..0.75,
        };

        range.contains(&self.progress)
    }
}
