#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_trait_impl)]
#![feature(effects)]

pub mod direction;
pub mod shapes;

use bevy::prelude::*;
pub use direction::Direction as GridDirection;

#[derive(Debug, Clone, Copy, Default, PartialEq, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SquareLayout {
    pub square_size: f32,
    pub origin: Vec2,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Square {
    pub x: i32,
    pub y: i32,
}

#[inline]
pub const fn square(x: i32, y: i32) -> Square {
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
