//! Some useful extensions for [`Vec2`].

use bevy::math::Vec2;
use common_visuals::camera::{PIXEL_VISIBLE_HEIGHT, PIXEL_VISIBLE_WIDTH};

/// Some useful extensions for [`Vec2`].
pub trait Vec2Ext {
    /// Sometimes we work in graphic coordinates, where the top left is (0, 0).
    fn as_top_left_into_centered(&self) -> Self;

    /// Helper function that exports z coordinate given y coordinate.
    ///
    /// It's domain in pixels is from -100_000 to 100_000.
    ///
    /// It's range is from -0.1 to 1.1.
    fn ysort(self) -> f32;

    /// Square root of each element of the vector.
    fn sqrt(self) -> Self;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn as_top_left_into_centered(&self) -> Self {
        Self::new(
            self.x - PIXEL_VISIBLE_WIDTH / 2.0,
            -self.y + PIXEL_VISIBLE_HEIGHT / 2.0,
        )
    }

    /// Helper function that exports z coordinate given y coordinate.
    ///
    /// It's domain in pixels is from -100_000 to 100_000.
    ///
    /// It's range is from -0.1 to 1.1.
    #[inline]
    fn ysort(self) -> f32 {
        // it's easier to just hardcode the range than pass around values
        //
        // this gives us sufficient buffer for maps of all sizes
        let (min, max) = (-100_000.0, 100_000.0);
        let size = max - min;

        // we allow for a tiny leeway for positions outside of the bounding box
        ((max - self.y) / size).clamp(-0.1, 1.1)
    }

    #[inline]
    fn sqrt(self) -> Self {
        Self::new(self.x.sqrt(), self.y.sqrt())
    }
}
