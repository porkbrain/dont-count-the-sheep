use std::marker::PhantomData;

use bevy::asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext};
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Loads assets from .ron files.
/// The specific type of asset is determined by the type parameter `T`.
#[derive(Debug)]
pub struct Loader<T>(PhantomData<T>);

/// Errors that can occur when loading assets from .ron files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    /// The file could not be loaded, most likely not found.
    #[error("Could load ron file: {0}")]
    Io(#[from] std::io::Error),
    /// The string must be parsable into the `T` type.
    #[error("Could not parse ron file: {0}")]
    Ron(#[from] ron::de::SpannedError),
}

impl<T: Asset + DeserializeOwned> AssetLoader for Loader<T> {
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

            bevy::log::trace!("Loading RON for {}", T::type_path(),);
            Ok(ron::de::from_bytes(&bytes)?)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl<T> Default for Loader<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
