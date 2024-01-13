use bevy::reflect::Reflect;

use crate::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum Direction {
    /// ↑
    Top,
    /// ↓
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
}
