#![doc = include_str!("../README.md")]

#[cfg(feature = "fps")]
mod fps;
mod systems;
mod types;

use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    diagnostic::FrameTimeDiagnosticsPlugin,
    render::color::Color,
};
pub use types::*;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, systems::advance_animation)
            .add_systems(
                Update,
                (systems::begin_animation_at_random, systems::flicker),
            );

        #[cfg(feature = "fps")]
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, fps::spawn)
            .add_systems(Update, (fps::update, fps::toggle));
    }

    fn finish(&self, _app: &mut App) {
        //
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
