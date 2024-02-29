//! Extensions to the `Color` type.

use bevy::render::color::Color;

/// Some useful extensions to the `Color` type.
pub trait ColorExt {
    /// <https://github.com/bevyengine/bevy/issues/1402>
    fn lerp(self, other: Self, t: f32) -> Self;
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
}
