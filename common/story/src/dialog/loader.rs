use bevy::asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use thiserror::Error;

use super::{list::Namespace, DialogGraph};

#[derive(Default)]
pub(crate) struct Loader;

/// Errors that can occur when loading assets
#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum LoaderError {
    /// The file could not be loaded, most likely not found.
    #[error("Could load toml file: {0}")]
    Io(#[from] std::io::Error),
    /// The string must be parsable into the `T` type.
    #[error("Could not parse toml file: {0}")]
    Toml(#[from] toml::de::Error),
}

impl AssetLoader for Loader {
    type Asset = DialogGraph;
    type Settings = ();
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut s = String::new();
            reader.read_to_string(&mut s).await?;
            let toml = toml::de::from_str(&s)?;

            let namespace = Namespace::from(load_context.asset_path());
            Ok(DialogGraph::subgraph_from_toml(namespace, toml))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}
