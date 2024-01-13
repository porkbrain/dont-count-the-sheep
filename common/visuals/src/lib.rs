#![doc = include_str!("../README.md")]

pub mod camera;
#[cfg(feature = "fps")]
mod fps;
pub mod systems;
mod types;

use bevy::{
    app::{App, Startup, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    render::color::Color,
};
pub use types::*;

/// `#0d0e1f`
pub const PRIMARY_COLOR: Color =
    Color::rgb(0.050980393, 0.05490196, 0.12156863);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "fps")]
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, fps::spawn)
            .add_systems(Update, (fps::update, fps::toggle));
    }
}

pub trait ColorExt {
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
