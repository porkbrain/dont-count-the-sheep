use bevy::math::Vec2;
use common_visuals::camera::{PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH};

pub trait Vec2Ext {
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
