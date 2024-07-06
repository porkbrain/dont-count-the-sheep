use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    utils::ConditionalSendFuture,
};
use thiserror::Error;

use crate::rscn::{Config, TscnTree};

/// Loads .tscn files into [`TscnTree`] representation.
#[derive(Default)]
pub struct TscnLoader;

/// Errors that can occur when loading assets from .tscn files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LoaderError {
    /// The file could not be loaded, most likely not found.
    #[error("Could load .tscn file: {0}")]
    Io(#[from] std::io::Error),
    /// We convert the file bytes into a string, which can fail.
    #[error("Non-utf8 string in .tscn file: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

impl AssetLoader for TscnLoader {
    type Asset = TscnTree;
    type Settings = Config;
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<
        Output = Result<
            <Self as AssetLoader>::Asset,
            <Self as AssetLoader>::Error,
        >,
    > {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let tscn = std::str::from_utf8(&bytes)?;

            Ok(crate::rscn::parse(tscn, settings))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tscn"]
    }
}
