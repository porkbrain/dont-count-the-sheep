#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_trait_impl)]
#![feature(effects)]

pub mod direction;
pub mod shapes;

use std::ops::{Add, Sub};

use bevy::prelude::*;
pub use direction::GridDirection;

#[derive(Debug, Clone, Copy, Default, PartialEq, Reflect)]
#[reflect(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SquareLayout {
    /// For example in pixels
    pub square_size: f32,
    /// Where in the world does this layout have square(0, 0).
    pub origin: Vec2,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect)]
#[reflect(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Square {
    pub x: i32,
    pub y: i32,
}

#[inline]
pub const fn sq(x: i32, y: i32) -> Square {
    Square::new(x, y)
}

impl Square {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn manhattan_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl SquareLayout {
    #[inline]
    pub const fn square_to_world_pos(&self, square: Square) -> Vec2 {
        let x = square.x as f32 * self.square_size;
        let y = square.y as f32 * self.square_size;
        self.origin + Vec2::new(x, y)
    }

    #[inline]
    pub fn world_pos_to_square(&self, pos: Vec2) -> Square {
        let pos = pos - self.origin;
        let x = (pos.x / self.square_size).round() as i32;
        let y = (pos.y / self.square_size).round() as i32;
        Square::new(x, y)
    }

    #[inline]
    pub const fn world_pos_to_square_const(&self, pos: Vec2) -> Square {
        let pos = pos - self.origin;
        let x = pos.x / self.square_size;
        let y = pos.y / self.square_size;

        const fn round(x: f32) -> i32 {
            (x + 0.5) as i32
        }

        Square::new(round(x), round(y))
    }

    #[inline]
    pub const fn square(&self) -> Vec2 {
        Vec2::splat(self.square_size)
    }
}

impl From<Square> for Vec2 {
    #[inline]
    fn from(square: Square) -> Self {
        Vec2::new(square.x as f32, square.y as f32)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl PartialOrd for Square {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Square {
    /// Square is ordered by x then y
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.x.cmp(&other.x) {
            std::cmp::Ordering::Equal => self.y.cmp(&other.y),
            ord => ord,
        }
    }
}

impl Add for Square {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        sq(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Add<&Square> for Square {
    type Output = Self;

    fn add(self, rhs: &Square) -> Self::Output {
        sq(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Add for &Square {
    type Output = Square;

    fn add(self, rhs: Self) -> Self::Output {
        sq(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Add<Square> for &Square {
    type Output = Square;

    fn add(self, rhs: Square) -> Self::Output {
        sq(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Square {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        sq(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Sub<&Square> for Square {
    type Output = Self;

    fn sub(self, rhs: &Square) -> Self::Output {
        sq(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Sub for &Square {
    type Output = Square;

    fn sub(self, rhs: Self) -> Self::Output {
        sq(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Sub<Square> for &Square {
    type Output = Square;

    fn sub(self, rhs: Square) -> Self::Output {
        sq(self.x - rhs.x, self.y - rhs.y)
    }
}
