//! Extensions to the `Color` type.

use bevy::color::{Color, Srgba};

/// Some useful extensions to the `Color` type.
pub trait ColorExt {
    /// <https://github.com/bevyengine/bevy/issues/1402>
    fn lerp(self, other: Self, t: f32) -> Self;

    /// <https://stackoverflow.com/a/6961743/5093093>
    fn invert(self) -> Self;
}

impl ColorExt for Color {
    fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);

        let Srgba {
            red: self_r,
            green: self_g,
            blue: self_b,
            alpha: self_a,
        } = self.to_srgba();
        let Srgba {
            red: other_r,
            green: other_g,
            blue: other_b,
            alpha: other_a,
        } = other.to_srgba();

        Color::srgba(
            self_r + (other_r - self_r) * t,
            self_g + (other_g - self_g) * t,
            self_b + (other_b - self_b) * t,
            self_a + (other_a - self_a) * t,
        )
    }

    fn invert(self) -> Self {
        let Srgba {
            red: self_r,
            green: self_g,
            blue: self_b,
            alpha: self_a,
        } = self.to_srgba();

        Color::srgba(1.0 - self_r, 1.0 - self_g, 1.0 - self_b, self_a)
    }
}
