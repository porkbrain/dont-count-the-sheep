//! Zindex is a magical number determines stacking of elements.
//! Let's keep all that magic to this module otherwise we'll need a wizard to
//! maintain it.

pub(crate) const BG_ROOM_AND_KITCHEN: f32 = 0.0;

pub(crate) const BG_HALLWAY: f32 = BG_ROOM_AND_KITCHEN;

/// Must be behead the window.
pub(crate) const CLOUD_ATLAS: f32 = 0.5;

pub(crate) const BACKWALL_FURNITURE: f32 = 1.0;
pub(crate) const ELEVATOR: f32 = 1.0;
pub(crate) const KITCHEN_FURNITURE_MIDDLE: f32 = 2.0;
pub(crate) const KITCHEN_FURNITURE_CLOSEST: f32 = 3.0;
pub(crate) const BEDROOM_FURNITURE_MIDDLE: f32 = 4.0;
pub(crate) const BEDROOM_FURNITURE_CLOSEST: f32 = 6.0;
pub(crate) const HALLWAY_DOORS: f32 = 7.0;
