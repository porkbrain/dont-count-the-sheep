#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(trivial_bounds)]

pub mod camera;
#[cfg(feature = "fps")]
mod fps;
pub mod systems;
mod types;

use bevy::{
    app::{App, Last, Startup, Update},
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
        app.add_event::<BeginInterpolationEvent>()
            .register_type::<TranslationInterpolation>()
            .register_type::<ColorInterpolation>()
            .register_type::<BeginAtlasAnimationAtRandom>()
            .register_type::<Flicker>();

        app.add_systems(Last, systems::recv_begin_interpolation_events);

        #[cfg(feature = "fps")]
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, fps::spawn)
            .add_systems(Update, (fps::update, fps::toggle));
    }
}
