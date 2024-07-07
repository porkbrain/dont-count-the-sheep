//! Daybar is a HUD element that shows the player's progress through the day.
//! It attaches itself to the [`MainCamera`].
//!
//! See wiki for more information about day progress.

use std::ops::{Add, AddAssign, Neg, Sub};

use bevy::ui::RelativeCursorPosition;
use common_assets::{
    fonts,
    ui::{HEARTBEAT_ATLAS, HEARTBEAT_ATLAS_SIZE, TIME_ATLAS},
};
use common_ext::QueryExt;
use common_visuals::camera::{MainCamera, PIXEL_ZOOM};

use super::{MARGIN_LEFT_PX, MARGIN_TOP_PX};
use crate::{player_stats::PlayerStats, prelude::*};

const TOOLTIP_FONT: &str = fonts::PIXEL1;
const TOOLTIP_FONT_SIZE: f32 = 18.0;

/// Unit of time.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect, Default,
)]
pub struct Beats(pub isize);

/// Manages in-game time progression.
#[derive(Resource)]
#[cfg_attr(feature = "devtools", derive(Reflect))]
#[cfg_attr(feature = "devtools", reflect(Resource))]
pub struct DayBar {
    /// The current progress through the day, from 0 to [`Beats::DAY`].
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
    NewDay,
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
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
    daybar: Res<DayBar>,

    camera: Query<Entity, With<MainCamera>>,
) {
    cmd.spawn((
        Name::new("DayBar"),
        DayBarRoot,
        TargetCamera(camera.single()),
        Interaction::default(),
        RelativeCursorPosition::default(),
        ImageBundle {
            focus_policy: bevy::ui::FocusPolicy::Block,
            style: Style {
                position_type: PositionType::Absolute,

                top: Val::Px(MARGIN_TOP_PX),
                left: Val::Px(MARGIN_LEFT_PX),

                width: Val::Px(
                    HEARTBEAT_ATLAS_SIZE.x as f32 * PIXEL_ZOOM as f32,
                ),
                height: Val::Px(
                    HEARTBEAT_ATLAS_SIZE.y as f32 * PIXEL_ZOOM as f32,
                ),

                ..default()
            },
            image: UiImage::new(asset_server.load(TIME_ATLAS)),
            ..default()
        },
        TextureAtlas {
            index: daybar.time_atlas_sprite_index(),
            layout: atlases.add(TextureAtlasLayout::from_grid(
                HEARTBEAT_ATLAS_SIZE,
                13,
                1,
                default(),
                default(),
            )),
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,

                    width: Val::Px(
                        HEARTBEAT_ATLAS_SIZE.x as f32 * PIXEL_ZOOM as f32,
                    ),
                    height: Val::Px(
                        HEARTBEAT_ATLAS_SIZE.y as f32 * PIXEL_ZOOM as f32,
                    ),

                    ..default()
                },
                image: UiImage::new(asset_server.load(HEARTBEAT_ATLAS)),
                ..default()
            },
            TextureAtlas {
                index: 0,
                layout: atlases.add(TextureAtlasLayout::from_grid(
                    HEARTBEAT_ATLAS_SIZE,
                    3,
                    1,
                    default(),
                    default(),
                )),
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

pub(crate) fn update(
    mut events: EventReader<UpdateDayBarEvent>,
    mut daybar: ResMut<DayBar>,
    mut stats: ResMut<PlayerStats>,

    mut daybar_ui: Query<&mut TextureAtlas, With<DayBarRoot>>,
) {
    for event in events.read() {
        // beats never go backwards except on a new day
        let full_cost = match event {
            // !
            // unlike the other, this one resets the daybar and so it's not
            // considered an activity that would need discounting
            UpdateDayBarEvent::NewDay => {
                daybar.progress = Beats(0);
                stats.traits.early_bird.extra_beats_today = Beats(0);
                stats.traits.night_owl.extra_beats_today = Beats(0);
                stats.days_passed += 1;

                continue;
            }

            UpdateDayBarEvent::ChangedScene => Beats::TEN_MINUTES,
            UpdateDayBarEvent::Meditated => Beats::FIFTEEN_MINUTES,
            UpdateDayBarEvent::Custom(Beats(amount)) => {
                debug_assert!(*amount > 0, "Beats can only go forward");
                Beats(*amount)
            }
        };

        let progress = daybar.progress;
        let progress_after_full_cost =
            (progress + full_cost).clamp(Beats(0), Beats::DAY);
        let real_cost = progress_after_full_cost - progress;

        daybar.progress += if real_cost <= Beats(1) {
            // there's nothing that can be discounted
            real_cost
        } else {
            stats.discount_activity(progress, real_cost)
        };
    }

    if let Some(mut daybar_ui) = daybar_ui.get_single_mut_or_none() {
        daybar_ui.index = daybar.time_atlas_sprite_index();
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
                            left: Val::Percent(65.0),
                            top: Val::Percent(65.0),
                            aspect_ratio: Some(10.0),
                            border: UiRect::all(Val::Px(2.5)),
                            ..default()
                        },
                        background_color: BackgroundColor(
                            Color::BLACK.with_alpha(0.85),
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
                                font_size: TOOLTIP_FONT_SIZE,
                                font: asset_server.load(TOOLTIP_FONT),
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

    /// There are 13 sprite frames in the atlas representing these percentages:
    /// `100%, 94%, 87%, 75%, 63%, 50%, 38%, 25%, 17%, 13%, 10%, 5%, 0%`
    /// We want to change the sprite index when the fraction gets half way
    /// to the next percentage
    fn time_atlas_sprite_index(&self) -> usize {
        match self.progress.as_fraction_of_day() {
            0.0..=0.025 => 0,
            0.025..=0.075 => 1,
            0.075..=0.125 => 2,
            0.125..=0.16 => 3,
            0.16..=0.23 => 4,
            0.23..=0.33 => 5,
            0.33..=0.45 => 6,
            0.45..=0.58 => 7,
            0.58..=0.68 => 8,
            0.68..=0.80 => 9,
            0.80..=0.90 => 10,
            0.90..=0.96 => 11,
            _ => 12,
        }
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

    pub(crate) fn as_fraction_of_day(self) -> f32 {
        self.0 as f32 / Self::DAY.0 as f32
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

impl Sub for Beats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for Beats {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
