use crate::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    // ↑
    Top,
    // ↓
    Bottom,
    // ←
    Left,
    // →
    Right,
    // ↖
    TopLeft,
    // ↗
    TopRight,
    // ↙
    BottomLeft,
    // ↘
    BottomRight,
}

impl Square {
    pub fn neighbor(self, direction: Direction) -> Self {
        match direction {
            Direction::Top => Self::new(self.x, self.y + 1),
            Direction::Bottom => Self::new(self.x, self.y - 1),
            Direction::Left => Self::new(self.x - 1, self.y),
            Direction::Right => Self::new(self.x + 1, self.y),
            Direction::TopLeft => Self::new(self.x - 1, self.y + 1),
            Direction::TopRight => Self::new(self.x + 1, self.y + 1),
            Direction::BottomLeft => Self::new(self.x - 1, self.y - 1),
            Direction::BottomRight => Self::new(self.x + 1, self.y - 1),
        }
    }
}
