//! Some useful extensions for [`Vec2`].

use bevy::math::Vec2;
use common_visuals::camera::{PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH};

/// Some useful extensions for [`Vec2`].
pub trait Vec2Ext {
    /// Sometimes we work in graphic coordinates, where the top left is (0, 0).
    fn as_top_left_into_centered(&self) -> Self;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn as_top_left_into_centered(&self) -> Self {
        Self::new(
            self.x - PIXEL_VISIBLE_WIDTH / 2.0,
            -self.y + PIXEL_VISIBLE_HEIGHT / 2.0,
        )
    }
}
