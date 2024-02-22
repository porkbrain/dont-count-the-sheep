//! Zindex is a magical number determines stacking of elements.
//! Let's keep all that magic to this module otherwise we'll need a wizard to
//! maintain it.
//!
//! Most entities will have their z-index set with
//! [`common_top_down::layout::TopDownScene::extend_z`].

pub(crate) const BG_BATHROOM_BEDROOM_AND_KITCHEN: f32 = -3.0;
pub(crate) const BG_HALLWAY: f32 = -2.0;

pub(crate) const CLOUD_ATLAS: f32 = -1.5;

pub(crate) const BACKWALL_FURNITURE: f32 = -1.0;
pub(crate) const ELEVATOR: f32 = -1.0;
