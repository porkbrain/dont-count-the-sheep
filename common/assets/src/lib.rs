//! Exports paths to the assets used by the game.
//! Also exports a [`ron_loader::Loader`] for loading assets from .ron files.
//! We store e.g. level layouts this way.

pub mod ignore_loader;
mod paths;
pub mod ron_loader;
pub mod store;

pub use paths::*;
pub use store::AssetStore;
