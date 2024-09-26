use std::{
    fmt::{self, Display},
    ops::AddAssign,
};

use main_game_lib::common_ext::QueryExt;

use super::consts::*;
use crate::prelude::*;

#[derive(Component, Default)]
pub(crate) struct Score {
    total: usize,
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
                    color: Srgba::hex(HIGHLIGHT_COLOR).unwrap().into(),
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

pub(super) fn update(mut score: Query<(&mut Score, &mut Text)>) {
    if let Some((score, mut text)) = score.get_single_mut_or_none() {
        text.sections[0].value = score.to_string();
    }
}

impl AddAssign<usize> for Score {
    fn add_assign(&mut self, rhs: usize) {
        self.total += rhs;
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.total)
    }
}
