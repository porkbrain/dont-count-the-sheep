#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub mod camera;
#[cfg(feature = "fps")]
mod fps;
pub mod systems;
mod types;

use bevy::{
    app::{App, Startup, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    math::{cubic_splines::CubicSegment, Vec2},
    render::color::Color,
};
use lazy_static::lazy_static;
pub use types::*;

/// `#0d0e1f`
pub const PRIMARY_COLOR: Color =
    Color::rgb(0.050980393, 0.05490196, 0.12156863);

lazy_static! {
    /// The ubiquitous "ease-in-out" animation curve.
    pub static ref EASE_IN_OUT: CubicSegment<Vec2> =
        CubicSegment::new_bezier((0.25, 0.1), (0.25, 1.0));
}

/// Only registers FPS counter if the `fps` feature is enabled.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "fps")]
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, fps::spawn)
            .add_systems(Update, (fps::update, fps::toggle));
    }
}

/// Some useful extensions to the `Color` type.
pub trait ColorExt {
    /// <https://github.com/bevyengine/bevy/issues/1402>
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl ColorExt for Color {
    fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Color::rgb(
            self.r() + (other.r() - self.r()) * t,
            self.g() + (other.g() - self.g()) * t,
            self.b() + (other.b() - self.b()) * t,
        )
    }
}
