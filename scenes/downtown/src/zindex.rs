//! Zindex is a magical number determines stacking of elements.
//! Let's keep all that magic to this module otherwise we'll need a wizard to
//! maintain it.
//!
//! Most entities will have their z-index set with
//! [`common_top_down::layout::TopDownScene::extend_z`].

pub(crate) const BG: f32 = -3.0;
