use std::ops::AddAssign;

use crate::prelude::*;

use super::consts::*;

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Score(usize);

pub(crate) fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(25.0),
                top: Val::Px(25.0),
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

pub(crate) fn update(mut score: Query<(&Score, &mut Text)>) {
    let Ok((score, mut text)) = score.get_single_mut() else {
        return;
    };

    text.sections[0].value = score.to_string();
}

impl AddAssign<usize> for Score {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}
