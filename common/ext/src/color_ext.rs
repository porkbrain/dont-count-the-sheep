//! Extensions to the `Color` type.

use bevy::render::color::Color;

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
        Color::rgba(
            self.r() + (other.r() - self.r()) * t,
            self.g() + (other.g() - self.g()) * t,
            self.b() + (other.b() - self.b()) * t,
            self.a() + (other.a() - self.a()) * t,
        )
    }

    fn invert(self) -> Self {
        Color::rgba(1.0 - self.r(), 1.0 - self.g(), 1.0 - self.b(), self.a())
    }
}
