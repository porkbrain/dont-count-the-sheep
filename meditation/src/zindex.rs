//! Zindex is a magical number determines stacking of elements.
//! Let's keep all that magic to this module otherwise we'll need a wizard to
//! maintain it.

pub(crate) const MAIN_BACKGROUND: f32 = -3.0;
pub(crate) const TWINKLES: f32 = -2.0;
pub(crate) const SHOOTING_STARS: f32 = -1.0;

pub(crate) const BLACK_HOLE: f32 = 0.0;
pub(crate) const BLACK_HOLE_TWINKLE: f32 = BLACK_HOLE + 0.1;

pub(crate) const SPARK_EFFECT: f32 = 1.0;

pub(crate) const CLIMATE: f32 = 2.0;

pub(crate) const DISTRACTION_BASE: f32 = 3.0;
pub(crate) const DISTRACTION_VIDEO: f32 = -0.2; // children so start at 0
pub(crate) const DISTRACTION_STATIC: f32 = -0.1; // children so start at 0
pub(crate) const DISTRACTION_CRACK: f32 = DISTRACTION_BASE;
pub(crate) const DISTRACTION_FRAME: f32 = 0.1; // children so start at 0
pub(crate) const DISTRACTION_BOLT: f32 = 0.2; // children so start at 0

pub(crate) const WEATHER: f32 = 4.0;
pub(crate) const WEATHER_ARROW: f32 = WEATHER;
