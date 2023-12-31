use std::marker::PhantomData;

use bevy::asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug)]

pub struct RonLoader<T>(PhantomData<T>);

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Could load ron file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Non-utf8 string in ron file: {0}")]
    Utf8(#[from] std::str::Utf8Error),
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
