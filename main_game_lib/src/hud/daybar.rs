//! Daybar is a HUD element that shows the player's progress through the day.
//! It attaches itself to the [`MainCamera`].
//!
//! See wiki for more information about day progress.

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
        };

        daybar.progress = (daybar.progress + amount).clamp(0.0, 1.0);

        if let Some(mut progress) = progress.get_single_mut_or_none() {
            progress.width = Val::Percent(daybar.progress * 100.0);
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
}
