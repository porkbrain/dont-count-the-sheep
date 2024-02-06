use bevy::{
    math::Vec2,
    reflect::{std_traits::ReflectDefault, Reflect},
};

use crate::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect, Default)]
#[reflect(Default)]
pub enum Direction {
    /// ↑
    Top,
    /// ↓
    ///
    /// Set as default arbitrarily to allow reflection and default value.
    #[default]
    Bottom,
    /// ←
    Left,
    /// →
    Right,
    /// ↖
    TopLeft,
    /// ↗
    TopRight,
    /// ↙
    BottomLeft,
    /// ↘
    BottomRight,
}

impl Square {
    #[inline]
    pub fn neighbor(self, direction: Direction) -> Self {
        let (x, y) = match direction {
            Direction::Top => (self.x, self.y + 1),
            Direction::Bottom => (self.x, self.y - 1),
            Direction::Left => (self.x - 1, self.y),
            Direction::Right => (self.x + 1, self.y),
            Direction::TopLeft => (self.x - 1, self.y + 1),
            Direction::TopRight => (self.x + 1, self.y + 1),
            Direction::BottomLeft => (self.x - 1, self.y - 1),
            Direction::BottomRight => (self.x + 1, self.y - 1),
        };

        Self::new(x, y)
    }

    #[inline]
    pub fn neighbors(self) -> impl Iterator<Item = Self> {
        use Direction::*;

        [
            Top,
            Bottom,
            Left,
            Right,
            TopLeft,
            TopRight,
            BottomLeft,
            BottomRight,
        ]
        .iter()
        .copied()
        .map(move |direction| self.neighbor(direction))
    }

    /// Given a square, returns the direction to the other square.
    /// They don't have to be neighbors, works at arbitrary distance.
    /// If they are the same square then returns `None`.
    pub fn direction_to(self, other: Self) -> Option<Direction> {
        use Direction::*;

        if self == other {
            return None;
        }

        // angle between the diff and the top vector
        #[allow(clippy::approx_constant)]
        let direction = match (Vec2::from(other) - Vec2::from(self))
            .angle_between(Vec2::new(0.0, 1.0))
        {
            // 15 deg = pi/12 ~= 0.2618

            // +- PI/12 is top
            -0.2618..=0.2618 => Top,
            // PI +- PI/12 is bottom
            -3.1416..=-2.8798 | 2.8798..=3.1416 => Bottom,
            // PI/2 +- PI/12 is right
            1.3090..=1.8326 => Right,
            // -PI/2 +- PI/12 is left
            -1.8326..=-1.3090 => Left,
            // PI/12 to PI/2 is top right
            0.2618..=1.3090 => TopRight,
            // -PI/2 to -PI/12 is top left
            -1.3090..=-0.2618 => TopLeft,
            // -PI to -PI/2 is bottom left
            -3.1416..=-1.8326 => BottomLeft,
            // PI/2 to PI is bottom right
            1.8326..=3.1416 => BottomRight,

            _ => return None,
        };

        Some(direction)
    }
}
