use std::{
    fmt::{self, Display},
    ops::AddAssign,
};

use bevy::time::Stopwatch;
use main_game_lib::common_ext::QueryExt;

use super::consts::*;
use crate::{climate::ClimateLightMode, prelude::*};

#[derive(Component)]
pub(crate) struct Score {
    total: usize,
    last_deduction: Stopwatch,
    deduction_interval: Duration,
    deduction_per_interval: usize,
}

#[derive(Component)]
pub(super) struct ScoreEntity;

pub(super) fn spawn(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn((
        ScoreEntity,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(SCORE_EDGE_OFFSET),
                top: Val::Px(SCORE_EDGE_OFFSET),
                ..default()
            },
            ..default()
        },
    ))
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

pub(super) fn despawn(
    mut cmd: Commands,

    entities: Query<Entity, With<ScoreEntity>>,
) {
    for entity in entities.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

pub(super) fn update(
    mut score: Query<(&mut Score, &mut Text)>,
    time: Res<Time>,
) {
    let Some((mut score, mut text)) = score.get_single_mut_or_none() else {
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
            deduction_per_interval: ClimateLightMode::default().deduction(),
            deduction_interval: ClimateLightMode::default()
                .deduction_interval(),
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.total)
    }
}

impl Score {
    pub(crate) fn set_deduction_interval(&mut self, interval: Duration) {
        self.deduction_interval = interval;
    }

    pub(crate) fn set_deduction(&mut self, deduction: usize) {
        self.deduction_per_interval = deduction;
    }
}
