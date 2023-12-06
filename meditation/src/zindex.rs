//! Zindex is a magical number determines stacking of elements.
//! Let's keep all that magic to this module otherwise we'll need a wizard to
//! maintain it.

pub(crate) const MAIN_BACKGROUND: f32 = -3.0;
pub(crate) const TWINKLES: f32 = -2.0;
pub(crate) const SHOOTING_STARS: f32 = -1.0;

pub(crate) const SPARK_EFFECT: f32 = 1.0;

pub(crate) const MENU: f32 = 2.0;

pub(crate) const WEATHER: f32 = 3.0;
pub(crate) const WEATHER_IN_MENU: f32 = WEATHER;
pub(crate) const WEATHER_ARROW: f32 = WEATHER;
