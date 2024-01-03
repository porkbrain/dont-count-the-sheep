use bevy::math::Vec2;

use crate::{VISIBLE_HEIGHT, VISIBLE_WIDTH};

pub trait Vec2Ext {
    fn as_top_left_into_centered(&self) -> Self;
}

impl Vec2Ext for Vec2 {
    fn as_top_left_into_centered(&self) -> Self {
        Self::new(self.x - VISIBLE_WIDTH / 2.0, -self.y + VISIBLE_HEIGHT / 2.0)
    }
}
