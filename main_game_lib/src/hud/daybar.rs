//! Daybar is a HUD element that shows the player's progress through the day.
//! It attaches itself to the [`MainCamera`].
//!
//! See wiki for more information about day progress.

use std::ops::{Add, Neg};

use bevy::ui::RelativeCursorPosition;
use common_assets::fonts;
use common_ext::QueryExt;
use common_visuals::camera::MainCamera;

use crate::prelude::*;

/// Unit of time.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect, Default,
)]
pub struct Beats(pub isize);

/// Manages in-game time progression.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DayBar {
    /// The current progress through the day, from 0 to [`DAY_LENGTH`].
    pub(crate) progress: Beats,
    /// If the tooltip is shown, it's some with the entity that can then be
    /// despawned to hide it again.
    pub(crate) tooltip: Option<Entity>,
}

/// Increases the day bar based on the situation that's happening.
#[derive(Event)]
pub enum UpdateDayBarEvent {
    /// Non trivial scene transition, such as leaving a building.
    ChangedScene,
    /// Finished meditating.
    Meditated,
    /// Sets the daybar progress to 0.
    Reset,
    /// Custom amount of increase in the day bar.
    /// The final progress is clamped between 0 and 1.
    Custom(Beats),
}

/// What sort of things are dependent on status of the daybar.
#[derive(Debug)]
pub enum DayBarDependent {
    /// The span of time when the mall is open.
    MallOpenHours,
    /// The span of time when the clinic is open.
    ClinicOpenHours,
    /// The span of time when the plant shop is open.
    PlantShopOpenHours,
}

#[derive(Component)]
pub(crate) struct DayBarRoot;

pub(crate) fn spawn(
    mut cmd: Commands,

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

                border: UiRect::all(Val::Px(2.0)),

                width: Val::Px(160.0),
                min_width: Val::Percent(8.0),
                height: Val::Px(160.0),
                min_height: Val::Percent(8.0),

                ..default()
            },
            background_color: BackgroundColor(Color::RED.with_a(0.75)),
            border_color: BorderColor(Color::WHITE.with_a(0.75)),
            ..default()
        },
    ));
}

pub(crate) fn despawn(
    mut cmd: Commands,
    root: Query<Entity, With<DayBarRoot>>,
) {
    cmd.entity(root.single()).despawn_recursive();
}

pub(crate) fn update(
    mut events: EventReader<UpdateDayBarEvent>,
    mut daybar: ResMut<DayBar>,
) {
    for event in events.read() {
        let amount = match event {
            UpdateDayBarEvent::ChangedScene => Beats::TEN_MINUTES,
            UpdateDayBarEvent::Meditated => Beats::FIFTEEN_MINUTES,
            UpdateDayBarEvent::Custom(amount) => *amount,
            UpdateDayBarEvent::Reset => -daybar.progress,
        };

        daybar.progress =
            (daybar.progress + amount).clamp(Beats(0), Beats::DAY);
    }

    // TODO: if tooltip is shown, update it
}

pub(crate) fn interact(
    mut cmd: Commands,
    mut daybar: ResMut<DayBar>,
    asset_server: Res<AssetServer>,

    root: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<DayBarRoot>),
    >,
) {
    let Some((entity, interaction)) = root.get_single_or_none() else {
        return;
    };

    if let Some(entity) = daybar.tooltip {
        if cmd.get_entity(entity).is_none() {
            daybar.tooltip = None;
        }
    }

    match interaction {
        Interaction::Hovered if daybar.tooltip.is_none() => {
            let tooltip = cmd
                .spawn((
                    Name::new("DayBarTooltip"),
                    NodeBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(12.5)),
                            position_type: PositionType::Absolute,
                            left: Val::Percent(75.0),
                            top: Val::Percent(105.0),
                            aspect_ratio: Some(10.0),
                            border: UiRect::all(Val::Px(2.5)),
                            ..default()
                        },
                        background_color: BackgroundColor(
                            Color::BLACK.with_a(0.85),
                        ),
                        border_color: BorderColor(Color::BLACK),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            // LOCALIZATION
                            format!(
                                "{} / {} beats\n({})",
                                daybar.progress.0,
                                Beats::DAY.0,
                                daybar.progress.time_of_day(),
                            ),
                            TextStyle {
                                color: Color::WHITE,
                                // TODO
                                font_size: 18.0,
                                font: asset_server.load(fonts::PIXEL1),
                            },
                        )
                        .with_justify(JustifyText::Center),
                        ..default()
                    });
                })
                .id();

            daybar.tooltip = Some(tooltip);
            cmd.entity(entity).add_child(tooltip);
        }
        Interaction::None => {
            if let Some(tooltip) = daybar.tooltip.take() {
                cmd.entity(tooltip).despawn_recursive();
            }
        }
        Interaction::Pressed => {
            if let Some(tooltip) = daybar.tooltip.take() {
                cmd.entity(tooltip).despawn_recursive();
            }

            // TODO: open character sheet
        }

        _ => {
            // nothing to do
        }
    }
}

impl Default for DayBar {
    fn default() -> Self {
        Self {
            progress: Beats(0),
            tooltip: None,
        }
    }
}

impl DayBar {
    /// Returns `true` if the day is over.
    pub fn is_depleted(&self) -> bool {
        self.progress >= Beats::DAY
    }

    /// Whether it's time for something to happen.
    pub fn is_it_time_for(&self, what: DayBarDependent) -> bool {
        let range = match what {
            DayBarDependent::MallOpenHours => ..Beats::EVENING,
            DayBarDependent::ClinicOpenHours => ..Beats::EVENING,
            DayBarDependent::PlantShopOpenHours => ..Beats::EVENING,
        };

        range.contains(&self.progress)
    }
}

impl Beats {
    /// How long waking hours last.
    pub const DAY: Self = Self(80_000);

    /// How many beats is approximately one hour.
    pub const ONE_HOUR: Self = Self(5_000);
    /// How many beats is approximately 15 minutes.
    pub const FIFTEEN_MINUTES: Self = Self(1_250);
    /// How many beats is approximately 10 minutes.
    pub const TEN_MINUTES: Self = Self(833);

    /// When does the evening start.
    pub const EVENING: Self = Self(55_000);

    /// When does the morning end.
    pub const NOON: Self = Self(25_000);

    /// LOCALIZATION
    fn time_of_day(self) -> String {
        if self < Beats::NOON {
            "morning".to_string()
        } else if self < Beats::EVENING {
            "midday".to_string()
        } else {
            "evening".to_string()
        }
    }
}

impl Neg for Beats {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for Beats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
