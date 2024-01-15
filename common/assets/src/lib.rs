//! Exports paths to the assets used by the game in [`paths`].
//! Also exports a [`RonLoader`] for loading assets from .ron files.
//! We store e.g. level layouts this way.

mod paths;
pub mod store;

use std::marker::PhantomData;

use bevy::asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext};
pub use paths::*;
use serde::de::DeserializeOwned;
pub use store::AssetStore;
use thiserror::Error;

/// Loads assets from .ron files.
/// The specific type of asset is determined by the type parameter `T`.
#[derive(Debug)]

pub struct RonLoader<T>(PhantomData<T>);

/// Errors that can occur when loading assets from .ron files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    /// The file could not be loaded, most likely not found.
    #[error("Could load ron file: {0}")]
    Io(#[from] std::io::Error),
    /// We convert the file bytes into a string, which can fail.
    #[error("Non-utf8 string in ron file: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    /// The string must be parsable into the `T` type.
    #[error("Could not parse ron file: {0}")]
    Ron(#[from] ron::de::SpannedError),
}

impl<T: Asset + DeserializeOwned> AssetLoader for RonLoader<T> {
    type Asset = T;
    type Settings = ();
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            Ok(ron::from_str(std::str::from_utf8(&bytes)?)?)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl<T> Default for RonLoader<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
