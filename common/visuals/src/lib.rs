#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![feature(trivial_bounds)]

pub mod camera;
mod fps;
pub mod systems;
mod types;

use bevy::{
    app::{App, FixedUpdate, Last, Startup, Update},
    color::Color,
    diagnostic::FrameTimeDiagnosticsPlugin,
    math::{cubic_splines::CubicSegment, Vec2},
};
use lazy_static::lazy_static;
pub use types::*;

/// `#0d0e1f`
pub const PRIMARY_COLOR: Color =
    Color::srgb(0.050980393, 0.05490196, 0.12156863);

lazy_static! {
    /// The ubiquitous "ease-in-out" animation curve.
    pub static ref EASE_IN_OUT: CubicSegment<Vec2> =
        CubicSegment::new_bezier((0.25, 0.1), (0.25, 1.0));
}

/// Only registers FPS counter if the `fps` feature is enabled.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // FPS is always displayed
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, fps::spawn)
            .add_systems(Update, fps::update);

        app.add_event::<BeginInterpolationEvent>();

        app.add_systems(
            FixedUpdate,
            (systems::advance_atlas_animation, systems::interpolate),
        )
        .add_systems(Last, systems::recv_begin_interpolation_events)
        .add_systems(
            Update,
            (systems::begin_atlas_animation_at_random, systems::flicker),
        );

        #[cfg(feature = "devtools")]
        {
            app.register_type::<AtlasAnimation>()
                .register_type::<AtlasAnimationEnd>()
                .register_type::<AtlasAnimationTimer>()
                .register_type::<TranslationInterpolation>()
                .register_type::<ColorInterpolation>()
                .register_type::<BeginAtlasAnimation>()
                .register_type::<Flicker>();
        }
    }
}
