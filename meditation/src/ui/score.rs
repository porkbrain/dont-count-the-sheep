use std::{
    fmt::{self, Display},
    ops::AddAssign,
};

use bevy::time::Stopwatch;

use crate::prelude::*;

use super::consts::*;

const HOT_DEDUCTION: usize = 80;
const HOT_DEDUCTION_INTERVAL: Duration = from_millis(5_000);
const COLD_DEDUCTION: usize = 100;
const COLD_DEDUCTION_INTERVAL: Duration = from_millis(10_000);

#[derive(Component)]
pub(crate) struct Score {
    total: usize,
    last_deduction: Stopwatch,
    deduction_interval: Duration,
    deduction_per_interval: usize,
}

pub(super) fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(SCORE_EDGE_OFFSET),
                top: Val::Px(SCORE_EDGE_OFFSET),
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Score::default(),
                TextBundle::from_section(
                    "0",
                    TextStyle {
                        font: asset_server.load(FONT),
                        font_size: SMALL_FONT_SIZE,
                        color: Color::hex(HIGHLIGHT_COLOR).unwrap(),
                    },
                ),
            ));
        });
}

pub(super) fn update(
    mut score: Query<(&mut Score, &mut Text)>,
    time: Res<Time>,
) {
    let Ok((mut score, mut text)) = score.get_single_mut() else {
        return;
    };

    score.last_deduction.tick(time.delta());

    if score.last_deduction.elapsed() > score.deduction_interval {
        score.total = score.total.saturating_sub(score.deduction_per_interval);
        score.last_deduction.reset();
    }

    text.sections[0].value = score.to_string();
}

impl AddAssign<usize> for Score {
    fn add_assign(&mut self, rhs: usize) {
        self.total += rhs;
    }
}

impl Default for Score {
    fn default() -> Self {
        Self {
            total: 0,
            last_deduction: stopwatch_at(from_millis(0)),
            deduction_per_interval: COLD_DEDUCTION,
            deduction_interval: COLD_DEDUCTION_INTERVAL,
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.total)
    }
}

impl Score {
    pub(crate) fn set_hot(&mut self) {
        self.deduction_per_interval = HOT_DEDUCTION;
        self.deduction_interval = HOT_DEDUCTION_INTERVAL;
    }

    pub(crate) fn set_cold(&mut self) {
        self.deduction_per_interval = COLD_DEDUCTION;
        self.deduction_interval = COLD_DEDUCTION_INTERVAL;
    }
}
