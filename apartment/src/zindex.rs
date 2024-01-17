//! Zindex is a magical number determines stacking of elements.
//! Let's keep all that magic to this module otherwise we'll need a wizard to
//! maintain it.

pub(crate) const BG: f32 = 0.0;

/// Must be behead the window.
pub(crate) const CLOUD_ATLAS: f32 = 0.5;

pub(crate) const BEDROOM_FURNITURE_DISTANT: f32 = 1.0;
pub(crate) const KITCHEN_FURNITURE_DISTANT: f32 = 1.0;
pub(crate) const KITCHEN_FURNITURE_MIDDLE: f32 = 2.0;
pub(crate) const KITCHEN_FURNITURE_CLOSEST: f32 = 3.0;
pub(crate) const BEDROOM_FURNITURE_MIDDLE: f32 = 4.0;
pub(crate) const HALLWAY: f32 = 5.0;
pub(crate) const BEDROOM_FURNITURE_CLOSEST: f32 = 6.0;
pub(crate) const HALLWAY_DOORS: f32 = 7.0;
